use core::fmt;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use regex::Regex;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::rnn::common::command::NeuronCommand;
use crate::rnn::common::input_cfg::InputCfg;
use crate::rnn::common::network_cfg::NeuronCfg;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::signal::Weight;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::utils::gen_id_by_spec_type;
use crate::rnn::neural::neuron::{self, Neuron, NeuronStatistics};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0_usize);
static CHANNEL_CAPACITY: usize = 5;
static GRACEFUL_SHUTDOWN_PERIOD: u64 = 20;

/// The network tracing mode
#[derive(Debug, Clone, PartialEq)]
pub enum MonitoringMode {
    /// Default mode
    None,

    /// Enable monitoring mode and store monitoring data from neurons
    /// into self.monitoring_ch.store vector
    Monitoring,
}

/// Networks mode set like as monitoring mode.
#[derive(Debug)]
struct Modes {
    monitoring_mode: MonitoringMode,
}

#[derive(Debug)]
struct CommandsCh {
    sender: Arc<broadcast::Sender<NeuronCommand>>,
}

#[derive(Debug)]
struct MonitoringCh {
    sender: Arc<mpsc::Sender<neuron::NeuronStatistics>>,
    store: Arc<RwLock<Vec<NeuronStatistics>>>,
}

/// Network is a high level container to other containers (neurons)
#[derive(Debug)]
pub struct Network {
    id: String,
    neurons: RwLock<BTreeMap<String, Arc<Neuron>>>,
    modes: Arc<RwLock<Modes>>,
    input_interface: Arc<RwLock<BTreeMap<usize, Arc<RwLock<Sender<u8>>>>>>,
    output_interface: Arc<RwLock<BTreeMap<usize, Arc<RwLock<Receiver<u8>>>>>>,
    commands_ch: CommandsCh,
    monitoring_ch: MonitoringCh,
    receivers_tracker: TaskTracker,
    cancel_token: CancellationToken,
}

impl Network {
    pub fn new() -> Result<Network, Box<dyn Error>> {
        let (monitoring_sender, monitoring_receiver) = mpsc::channel(CHANNEL_CAPACITY);
        let (commands_sender, _commands_receiver) = broadcast::channel(CHANNEL_CAPACITY);

        let net = gen_id_by_spec_type(
            "",
            unsafe { ID_COUNTER.fetch_add(1, Ordering::Relaxed) },
            &SpecificationType::Network,
        )
        .map(|id| Network {
            id,
            neurons: RwLock::new(BTreeMap::new()),
            modes: Arc::new(RwLock::new(Modes {
                monitoring_mode: MonitoringMode::None,
            })),
            input_interface: Arc::new(RwLock::new(BTreeMap::new())),
            output_interface: Arc::new(RwLock::new(BTreeMap::new())),
            commands_ch: CommandsCh {
                // receiver: Arc::new(commands_receiver), // TODO: Try not keeping this because receivers created as subscriptions
                sender: Arc::new(commands_sender),
            },
            monitoring_ch: MonitoringCh {
                sender: Arc::new(monitoring_sender),
                store: Arc::new(RwLock::new(vec![])),
            },
            receivers_tracker: TaskTracker::new(),
            cancel_token: CancellationToken::new(),
        })?;

        let monitoring_store_cloned = net.monitoring_ch.store.clone();
        let cancel_token_cloned = net.cancel_token.clone();

        let _ = &net.receivers_tracker.spawn(async move {
            tokio::select! {
                () = Self::monitoring_receiver_task(monitoring_store_cloned, monitoring_receiver) => {}
                () = cancel_token_cloned.cancelled() => {
                    println!("Waiting to shutdown....");
                    time::sleep(Duration::from_millis(GRACEFUL_SHUTDOWN_PERIOD)).await;
                    println!("Cleanup complete.");
                }
            }
        });

        Ok(net)
    }

    async fn monitoring_receiver_task(
        monitoring_store: Arc<RwLock<Vec<NeuronStatistics>>>,
        mut monitoring_receiver: mpsc::Receiver<NeuronStatistics>,
    ) {
        while let Some(neuron_state) = monitoring_receiver.recv().await {
            let mut w_monitoring_store = monitoring_store.write().await;
            w_monitoring_store.push(neuron_state); // TODO review possibility write into stream
        }
    }

    pub async fn set_monitoring_mode(&self, mode: MonitoringMode) {
        let mut w_state = self.modes.write().await;
        w_state.monitoring_mode = mode.clone();
        let _send_command_result = self
            .commands_ch
            .sender
            .send(NeuronCommand::SwitchMonitoringMode(mode));
    }

    pub async fn get_monitoring_mode(&self) -> MonitoringMode {
        self.modes.read().await.monitoring_mode.clone()
    }

    pub async fn get_neuron(&self, id: &str) -> Option<Arc<Neuron>> {
        let r_neurons = self.neurons.read().await;
        r_neurons.get(id).map(|neuron| Arc::clone(neuron))
    }

    pub fn get_commands_receiver(&self) -> broadcast::Receiver<NeuronCommand> {
        self.commands_ch.sender.subscribe()
    }

    pub fn get_monitoring_sender(&self) -> mpsc::WeakSender<NeuronStatistics> {
        self.monitoring_ch.sender.downgrade()
    }

    pub async fn create_neuron(
        &self,
        network: Arc<Network>,
        bias: Weight,
        input_configs: Vec<InputCfg>,
    ) -> Result<Arc<Neuron>, Box<dyn std::error::Error>> {
        use std::collections::btree_map::Entry;

        let prefix = 'Z';
        let new_id = format!(
            "{}{prefix}{}",
            self.get_id(),
            self.get_available_neuron_id().await
        );
        let input_configs = if input_configs.is_empty() {
            vec![InputCfg {
                capacity_max: 1,
                regeneration: 1,
                weight: 1,
            }]
        } else {
            input_configs
        };

        let neuron_config = NeuronCfg {
            id: new_id.clone(),
            bias,
            input_configs,
        };
        let mut w_neurons = self.neurons.write().await;
        match w_neurons.entry(new_id.clone()) {
            Entry::Vacant(entry) => Ok(Arc::clone(
                entry.insert(Neuron::build(Arc::clone(&network), neuron_config).await),
            )),
            Entry::Occupied(_) => Err(Box::new(RnnError::IdBusy(format!(
                "Id {} already used",
                new_id
            )))),
        }
    }

    pub async fn get_available_neuron_id(&self) -> usize {
        let r_neurons = self.neurons.read().await;
        r_neurons.keys().last().map_or(0, |id| {
            if id.is_empty() {
                return 0;
            }

            let r_pattern = r"^M\d+Z(\d+)$";

            let rex = Regex::new(&r_pattern).unwrap();
            let captures = rex.captures(id).unwrap();
            if &captures.len() < &2 {
                return 0;
            }
            let id_num = captures[1].parse::<usize>().unwrap();
            id_num + 1
        })
    }

    pub async fn remove_neuron(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut w_neurons = self.neurons.write().await;
        match w_neurons.remove(id) {
            Some(_) => Ok(()),
            None => Err(Box::new(RnnError::IdNotFound)),
        }
    }

    pub async fn has_neuron(&self, id: &str) -> bool {
        self.neurons.read().await.contains_key(id)
    }

    pub async fn connect_neurons(
        &self,
        src_id: &str,
        dst_id: &str,
        dst_port: usize,
    ) -> Result<(), Box<dyn Error>> {
        let src_neuron = self.get_neuron(src_id).await;
        let dst_neuron = self.get_neuron(dst_id).await;
        if src_neuron.is_some() && dst_neuron.is_some() {
            let src_neuron = src_neuron.unwrap();
            let dst_neuron = dst_neuron.unwrap();
            src_neuron.link_to(dst_neuron, dst_port).await
        } else {
            Err(Box::new(RnnError::IdNotFound))
        }
    }

    pub async fn len(&self) -> usize {
        self.neurons.read().await.len()
    }

    /// Send signal to port connected to synapse
    pub async fn input(&self, signal: u8, port: usize) -> Result<usize, Box<dyn Error>> {
        if let Some(synapse) = self.input_interface.read().await.get(&port) {
            let r_synapse = synapse.read().await;
            r_synapse
                .send(signal)
                .map_err(|err| Box::new(err) as Box<dyn Error>)
        } else {
            Err(Box::new(RnnError::IdNotFound))
        }
    }

    /// Configuring the input interface of the network in such a way that an unambiguous
    /// mapping is established between input ports and neuron synapses.
    pub async fn setup_input(
        &self,
        network_port: usize,
        neuron_id: &str,
        neuron_port: usize,
    ) -> Result<(), Box<(dyn Error)>> {
        use std::collections::btree_map::Entry;

        if let Some(neuron) = self.get_neuron(neuron_id).await {
            let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
            let src_id = format!("{}I{}", self.get_id(), network_port);
            neuron
                .connect(&src_id, neuron_port, Arc::new(RwLock::new(rx)))
                .await?;
            let mut w_input_interface = self.input_interface.write().await;
            match w_input_interface.entry(network_port) {
                Entry::Vacant(entry) => {
                    entry.insert(Arc::new(RwLock::new(tx)));
                    Ok(())
                }
                Entry::Occupied(_) => Err(Box::new(RnnError::IdBusy(format!(
                    "network port {} already used",
                    network_port
                )))),
            }
        } else {
            Err(Box::new(RnnError::IdNotFound) as Box<dyn Error>)
        }
    }

    pub async fn pop_monitoring_store(&self) -> Vec<NeuronStatistics> {
        let mut w_monitoring_store = self.monitoring_ch.store.write().await;
        let snapshot = w_monitoring_store.clone();
        w_monitoring_store.clear();
        snapshot
    }

    pub async fn get_current_neuron_statistics(
        &self,
        neuron_id: &str,
    ) -> Result<NeuronStatistics, Box<dyn Error>> {
        if let Some(target) = self.get_neuron(neuron_id).await {
            Ok(Neuron::prepare_statistics(&target.get_id(), &target.get_core()).await)
        } else {
            Err(Box::new(RnnError::IdNotFound))
        }
    }

    pub async fn free_output(&self, network_port: usize) -> Result<(), Box<dyn Error>> {
        let mut w_output_interface = self.output_interface.write().await;
        match w_output_interface.entry(network_port) {
            Entry::Occupied(entity) => {
                entity.remove();
                Ok(())
            }
            Entry::Vacant(_) => Err(Box::new(RnnError::AlreadyFree)),
        }
    }

    pub async fn setup_output(
        &self,
        network_port: usize,
        neuron_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut w_output_interface = self.output_interface.write().await;
        if let Some(neuron) = self.get_neuron(neuron_id).await {
            // connect axon to output port if port is free else return error
            match w_output_interface.entry(network_port.clone()) {
                Entry::Occupied(_) => Err(Box::new(RnnError::IdBusy(format!(
                    "Port {} already used",
                    network_port
                )))),
                Entry::Vacant(entity) => {
                    entity.insert(neuron.provide_output().await);
                    Ok(())
                }
            }
            // if  self.get
            // let c_receiver = neuron.provide_output().await;
            // let c_results = self.trace_log.clone();
            // let c_output_interface = self.output_interface.clone();
            // let c_port = network_port.clone();
            // let jh = tokio::task::spawn(async move {
            //     let port = Arc::new(c_port);
            //     let mut w_port_result_receiver = c_receiver.write().await;
            //     while let Ok(signal) = w_port_result_receiver.recv().await {
            //         let output_width = c_output_interface.read().await.len();
            //         let mut result: Vec<u8> = vec![0; output_width];
            //         result[*port] = signal;
            //         let result: String = result.iter().fold(String::new(), |mut acc, item| {
            //             acc.push_str(item.to_string().as_str());
            //             acc
            //         });
            //         c_results.write().await.push(result);
            //     }
            // });
            // self.processing_registers
            //     .write()
            //     .await
            //     .insert(network_port.clone(), jh);
        } else {
            Err(Box::new(RnnError::IdNotFound))
        }
    }

    pub async fn get_output_receiver(&self, port: usize) -> Option<Arc<RwLock<Receiver<u8>>>> {
        self.output_interface
            .read()
            .await
            .get(&port)
            .map(|rx| rx.clone())
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The Network {} ", self.id)
    }
}

#[cfg(test)]
mod tests {
    use crate::rnn::tests::fixtures::{gen_neuron_input_config_fixture, new_network_fixture};

    use super::*;

    #[tokio::test]
    async fn should_create_two_unique_networks() {
        let n1 = Network::new().unwrap();
        let n2 = Network::new().unwrap();

        assert_ne!(n1.id, n2.id);
    }

    #[tokio::test]
    async fn should_create_two_neurons_in_same_network() {
        let net_orig = Arc::new(Network::new().unwrap());

        for _ in 0..=1 {
            let net = net_orig.clone();
            assert!(net.create_neuron(net.clone(), 1, vec![]).await.is_ok());
        }

        let net = net_orig.clone();
        assert_eq!(net.len().await, 2);
    }

    #[tokio::test]
    async fn network_can_get_neuron_after_create() {
        let net = Arc::new(Network::new().unwrap());

        let neuron_rc = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let neuron_id = neuron_rc.get_id();

        assert_eq!(net.len().await, 1);
        assert!(
            net.get_neuron(neuron_id.as_str()).await.is_some(),
            "Neuron not found"
        );
        assert!(
            net.get_neuron("missed").await.is_none(),
            "Should be nothing"
        );
    }

    #[tokio::test]
    async fn network_can_remove_neuron_after_create() {
        let net = Arc::new(Network::new().unwrap());

        let neuron_rc = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        assert_eq!(net.len().await, 1);

        assert!(net.remove_neuron(&neuron_rc.get_id()).await.is_ok());
        assert_eq!(net.len().await, 0);
    }

    #[tokio::test]
    async fn network_should_return_error_if_remove_by_incorrect_id() {
        let net = Arc::new(Network::new().unwrap());

        let neuron = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let neuron_id = neuron.get_id();
        assert_eq!(net.len().await, 1);

        assert!(
            net.remove_neuron("missed").await.is_err(),
            "Should return error"
        );

        let result = net.get_neuron(&neuron_id).await.unwrap();
        assert_eq!(result.get_id(), neuron_id);
    }

    #[tokio::test]
    async fn network_can_verify_if_contains_neuron_with_specified_id() {
        let net = Arc::new(Network::new().unwrap());

        let neuron = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();

        assert!(net.has_neuron(neuron.get_id().as_str()).await);
        assert!(!net.has_neuron("missed").await);
    }

    #[tokio::test]
    async fn network_returns_correct_id_by_get_id() {
        let net = Network::new().unwrap();
        assert_eq!(net.id, net.get_id());
    }

    #[tokio::test]
    async fn fn_get_current_neuron_statistics_should_return_some_value() {
        let net = Arc::new(new_network_fixture());
        let neuron = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let id = neuron.get_id();

        assert!(net.get_current_neuron_statistics(&id).await.is_ok());
    }

    #[tokio::test]
    async fn should_connect_one_neuron_to_available_port_of_another_one() {
        let net = Arc::new(new_network_fixture());
        let neuron1 = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let src_id = neuron1.get_id();
        let neuron2 = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let dst_id = neuron2.get_id();

        let res = net.connect_neurons(&src_id, &dst_id, 0).await;
        assert!(res.is_ok());

        let src_stat = net.get_current_neuron_statistics(&src_id).await;

        assert!(src_stat.is_ok());
        let src_stat = src_stat.unwrap();
        assert_eq!(src_stat.dendrite_count, 1);
        assert_eq!(src_stat.dendrite_connected_count, 0);

        let dst_stat = net.get_current_neuron_statistics(&dst_id).await;
        assert!(dst_stat.is_ok());
        let dst_stat = dst_stat.unwrap();
        assert_eq!(dst_stat.dendrite_count, 1);
        assert_eq!(dst_stat.dendrite_connected_count, 1);
    }

    #[tokio::test]
    async fn should_not_connect_one_neuron_to_busy_port_of_another_one() {
        let net = Arc::new(new_network_fixture());

        let neuron_alt = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let alt_id = neuron_alt.get_id();
        let neuron1 = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let src_id = neuron1.get_id();
        let neuron2 = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let dst_id = neuron2.get_id();

        let res = net.connect_neurons(&src_id, &dst_id, 0).await;
        assert!(res.is_ok());

        let res = net.connect_neurons(&alt_id, &dst_id, 0).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_not_connect_one_neuron_to_missed_one() {
        let net = Arc::new(new_network_fixture());

        let neuron1 = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let src_id = neuron1.get_id();
        let dst_id = "M0Z555";

        let res = net.connect_neurons(&src_id, &dst_id, 0).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_not_allow_to_connect_self_if_only_one_dendrite_exists() {
        let net = Arc::new(new_network_fixture());

        let neuron = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let id = neuron.get_id();

        let res = net.connect_neurons(&id, &id, 0).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_self_connect_if_more_then_one_dendrite_exists() {
        let net = Arc::new(new_network_fixture());

        let neuron = net
            .create_neuron(net.clone(), 1, gen_neuron_input_config_fixture(2))
            .await
            .unwrap();
        let id = neuron.get_id();

        let res = net.connect_neurons(&id, &id, 0).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn should_set_correct_monitoring_mode_for_new_added_neuron() {
        let net = Arc::new(new_network_fixture());
        let _n1 = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        net.set_monitoring_mode(MonitoringMode::Monitoring).await;

        let n2 = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        assert_eq!(n2.get_monitoring_mode().await, MonitoringMode::Monitoring);
    }

    #[tokio::test]
    async fn should_store_monitoring_records_on_signal_operation() {
        let net = Arc::new(new_network_fixture());
        net.set_monitoring_mode(MonitoringMode::Monitoring).await;
        let n = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        assert!(net.setup_input(0, &n.get_id(), 0).await.is_ok());
        assert!(net.setup_output(0, &n.get_id()).await.is_ok());

        assert_eq!(net.pop_monitoring_store().await.len(), 0);
        assert!(net.input(1, 0).await.is_ok());

        tokio::time::sleep(Duration::from_millis(1)).await;

        assert_eq!(net.pop_monitoring_store().await.len(), 1);
    }
}
