use core::fmt;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use regex::Regex;
use tokio::sync::broadcast::{self, Receiver};
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::rnn::common::command::NeuronCommand;
use crate::rnn::common::input_cfg::InputCfg;
use crate::rnn::common::network_cfg::NeuronCfg;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::signal::{Signal, Weight};
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::status::{PortInfo, Status};
use crate::rnn::common::utils::gen_id_by_spec_type;
use crate::rnn::neural::neuron::Neuron;

use super::signal_handler::SignalHandler;

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

/// Network spreads command via command channel to all neurons.
#[derive(Debug)]
struct CommandsCh {
    sender: Arc<broadcast::Sender<NeuronCommand>>,
}

/// Neurons and ports are sending status information to Network when MonitoringMode is enabled
#[derive(Debug)]
struct MonitoringCh {
    sender: Arc<mpsc::Sender<Status>>,
    store: Arc<RwLock<Vec<Status>>>,
}

/// The port proprties
#[derive(Debug)]
struct PortCore {
    /// Port id
    id: String,

    /// Signal hits counter
    signal_hits: u64,

    /// Sender or receiver handler dependent from port kind (Input or Output)
    signal_handler: SignalHandler,
}

/// Network is a high level container to other containers (neurons)
#[derive(Debug)]
pub struct NeuralNetwork {
    /// The network id
    id: String,

    /// Inner neurons
    neurons: RwLock<BTreeMap<String, Arc<Neuron>>>,

    /// Network's modes set
    modes: Arc<RwLock<Modes>>,

    /// Network's input ports
    input_interface: Arc<RwLock<BTreeMap<usize, Arc<RwLock<PortCore>>>>>,

    /// Network's output ports
    output_interface: Arc<RwLock<BTreeMap<usize, Arc<RwLock<PortCore>>>>>,

    /// The command channel stuff. Network sends command to all inner neurons
    /// through broadcast channel.
    commands_ch: CommandsCh,

    /// The monitoring channel stuff. Neurons are sending staus messages to
    /// the network when enabled monitoring mode
    monitoring_ch: MonitoringCh,

    /// all receivers are handling incoming signals and messages in separated tasks.
    /// These tasks are tracked by receivers_tracker.
    receivers_tracker: TaskTracker,
    cancel_token: CancellationToken,
}

impl NeuralNetwork {
    pub fn new() -> Result<NeuralNetwork, Box<dyn Error>> {
        let (monitoring_sender, monitoring_receiver) = mpsc::channel(CHANNEL_CAPACITY);
        let (commands_sender, _commands_receiver) = broadcast::channel(CHANNEL_CAPACITY);

        let net = gen_id_by_spec_type(
            "",
            unsafe { ID_COUNTER.fetch_add(1, Ordering::Relaxed) },
            &SpecificationType::Network,
        )
        .map(|id| NeuralNetwork {
            id,
            neurons: RwLock::new(BTreeMap::new()),
            modes: Arc::new(RwLock::new(Modes {
                monitoring_mode: MonitoringMode::None,
            })),
            input_interface: Arc::new(RwLock::new(BTreeMap::new())),
            output_interface: Arc::new(RwLock::new(BTreeMap::new())),
            commands_ch: CommandsCh {
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
                () = Self::monitoring_save_task(monitoring_store_cloned, monitoring_receiver) => {}
                () = cancel_token_cloned.cancelled() => {
                    println!("Waiting to shutdown....");
                    time::sleep(Duration::from_millis(GRACEFUL_SHUTDOWN_PERIOD)).await;
                    println!("Cleanup complete.");
                }
            }
        });

        Ok(net)
    }

    async fn monitoring_save_task(
        monitoring_store: Arc<RwLock<Vec<Status>>>,
        mut monitoring_receiver: mpsc::Receiver<Status>,
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

    pub fn get_monitoring_sender(&self) -> mpsc::WeakSender<Status> {
        self.monitoring_ch.sender.downgrade()
    }

    pub async fn create_neuron(
        &self,
        network: Arc<NeuralNetwork>,
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
            Entry::Occupied(_) => Err(Box::new(RnnError::NeuronAlreadyExists(new_id))),
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
            None => Err(Box::new(RnnError::NeuronNotFound(id.to_string()))),
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
        if src_neuron.is_none() {
            return Err(Box::new(RnnError::NeuronNotFound(src_id.to_string())));
        }

        let dst_neuron = self.get_neuron(dst_id).await;
        if dst_neuron.is_none() {
            return Err(Box::new(RnnError::NeuronNotFound(dst_id.to_string())));
        }

        let src_neuron = src_neuron.unwrap();
        let dst_neuron = dst_neuron.unwrap();
        src_neuron.link_to(dst_neuron, dst_port).await
    }

    pub async fn len(&self) -> usize {
        self.neurons.read().await.len()
    }

    /// Send signal to port connected to synapse
    pub async fn input(&self, signal: Signal, port: usize) -> Result<usize, Box<dyn Error>> {
        if let Some(port_core) = self.input_interface.write().await.get_mut(&port) {
            let mut w_port_core = port_core.write().await;
            w_port_core.signal_hits += 1;
            if let SignalHandler::Input(synapse) = &w_port_core.signal_handler {
                let w_synapse = synapse.read().await;
                let result = w_synapse
                    .send(signal)
                    .map_err(|error| Box::new(error) as Box<dyn Error>);

                if result.is_ok() && self.get_monitoring_mode().await == MonitoringMode::Monitoring
                {
                    Self::send_port_status(
                        self.monitoring_ch.store.clone(),
                        w_port_core.id.as_str(),
                        w_port_core.signal_hits,
                        signal,
                    )
                    .await;
                }

                result
            } else {
                Err(Box::new(RnnError::IncorrectPortType))
            }
        } else {
            Err(Box::new(RnnError::PortNotFound(port)))
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
                .connect(&src_id, None, neuron_port, Arc::new(RwLock::new(rx)))
                .await?;
            let mut w_input_interface = self.input_interface.write().await;
            match w_input_interface.entry(network_port) {
                Entry::Vacant(entry) => {
                    entry.insert(Arc::new(RwLock::new(PortCore {
                        id: src_id.clone(),
                        signal_hits: 0,
                        signal_handler: SignalHandler::Input(Arc::new(RwLock::new(tx))),
                    })));
                    Ok(())
                }
                Entry::Occupied(_) => Err(Box::new(RnnError::PortBusy(src_id))),
            }
        } else {
            Err(Box::new(RnnError::NeuronNotFound(neuron_id.to_string())) as Box<dyn Error>)
        }
    }

    pub async fn pop_monitoring_store(&self) -> Vec<Status> {
        let mut w_monitoring_store = self.monitoring_ch.store.write().await;
        let snapshot = w_monitoring_store.clone();
        w_monitoring_store.clear();
        snapshot
    }

    pub async fn get_current_neuron_status(
        &self,
        neuron_id: &str,
    ) -> Result<Status, Box<dyn Error>> {
        if let Some(target) = self.get_neuron(neuron_id).await {
            Ok(Neuron::prepare_status(&target.get_id(), &target.get_core()).await)
        } else {
            Err(Box::new(RnnError::NeuronNotFound(String::from(neuron_id))))
        }
    }

    pub async fn free_output(&self, network_port: usize) -> Result<(), Box<dyn Error>> {
        let mut w_output_interface = self.output_interface.write().await;
        match w_output_interface.entry(network_port) {
            Entry::Occupied(entity) => {
                entity.remove();
                Ok(())
            }
            Entry::Vacant(_) => Err(Box::new(RnnError::PortAlreadyFree)),
        }
    }

    pub async fn setup_output(
        &self,
        network_port: usize,
        neuron_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let port_id = format!("{}O{}", self.get_id(), network_port);
        let mut w_output_interface = self.output_interface.write().await;
        if let Some(neuron) = self.get_neuron(neuron_id).await {
            // connect axon to output port if port is free else return error
            match w_output_interface.entry(network_port.clone()) {
                Entry::Occupied(_) => Err(Box::new(RnnError::PortBusy(port_id.clone()))),
                Entry::Vacant(entry) => {
                    let receiver = neuron.provide_output().await;
                    let port_core = Arc::new(RwLock::new(PortCore {
                        id: port_id.clone(),
                        signal_hits: 0,
                        signal_handler: SignalHandler::Output(receiver.clone()),
                    }));
                    entry.insert(port_core.clone());
                    let monitoring_store_cloned = self.monitoring_ch.store.clone();

                    self.receivers_tracker.spawn(async move {
                        while let Ok(signal) = receiver.write().await.recv().await {
                            let port_core_cloned = port_core.clone();
                            let mut w_port_core = port_core_cloned.write().await;
                            w_port_core.signal_hits += 1;
                            Self::send_port_status(
                                monitoring_store_cloned.clone(),
                                &port_id,
                                w_port_core.signal_hits,
                                signal,
                            )
                            .await;
                        }
                    });

                    Ok(())
                }
            }
        } else {
            Err(Box::new(RnnError::NeuronNotFound(neuron_id.to_string())))
        }
    }

    pub async fn get_output_receiver(&self, port: usize) -> Option<Arc<RwLock<Receiver<Signal>>>> {
        if let Some(port_core) = self.output_interface.read().await.get(&port) {
            let r_port_core = port_core.read().await;
            if let SignalHandler::Output(receiver) = &r_port_core.signal_handler {
                Some(receiver.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    async fn send_port_status(
        monitoring_store: Arc<RwLock<Vec<Status>>>,
        port_id: &str,
        signal_hits: u64,
        recent_signal: Signal,
    ) {
        let mut w_monitoring_store = monitoring_store.write().await;
        let timestamp = Utc::now();
        w_monitoring_store.push(Status::Port(PortInfo {
            timestamp,
            id: port_id.to_string(),
            hit_count: signal_hits,
            recent_signal,
        }));
    }
}

impl fmt::Display for NeuralNetwork {
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
        let n1 = NeuralNetwork::new().unwrap();
        let n2 = NeuralNetwork::new().unwrap();

        assert_ne!(n1.id, n2.id);
    }

    #[tokio::test]
    async fn should_create_two_neurons_in_same_network() {
        let net_orig = Arc::new(NeuralNetwork::new().unwrap());

        for _ in 0..=1 {
            let net = net_orig.clone();
            assert!(net.create_neuron(net.clone(), 1, vec![]).await.is_ok());
        }

        let net = net_orig.clone();
        assert_eq!(net.len().await, 2);
    }

    #[tokio::test]
    async fn network_can_get_neuron_after_create() {
        let net = Arc::new(NeuralNetwork::new().unwrap());

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
        let net = Arc::new(NeuralNetwork::new().unwrap());

        let neuron_rc = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        assert_eq!(net.len().await, 1);

        assert!(net.remove_neuron(&neuron_rc.get_id()).await.is_ok());
        assert_eq!(net.len().await, 0);
    }

    #[tokio::test]
    async fn network_should_return_error_if_remove_by_incorrect_id() {
        let net = Arc::new(NeuralNetwork::new().unwrap());

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
        let net = Arc::new(NeuralNetwork::new().unwrap());

        let neuron = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();

        assert!(net.has_neuron(neuron.get_id().as_str()).await);
        assert!(!net.has_neuron("missed").await);
    }

    #[tokio::test]
    async fn network_returns_correct_id_by_get_id() {
        let net = NeuralNetwork::new().unwrap();
        assert_eq!(net.id, net.get_id());
    }

    #[tokio::test]
    async fn fn_get_current_neuron_statistics_should_return_some_value() {
        let net = Arc::new(new_network_fixture());
        let neuron = net.create_neuron(net.clone(), 1, vec![]).await.unwrap();
        let id = neuron.get_id();

        assert!(net.get_current_neuron_status(&id).await.is_ok());
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

        let src_stat = net.get_current_neuron_status(&src_id).await;
        assert!(src_stat.is_ok());
        let src_stat = match src_stat.unwrap() {
            Status::Neuron(info) => Some(info),
            _ => None,
        };
        assert!(src_stat.is_some());
        let src_stat = src_stat.unwrap();
        assert_eq!(src_stat.dendrite_count, 1);
        assert_eq!(src_stat.dendrite_connected_count, 0);

        let dst_stat = net.get_current_neuron_status(&dst_id).await;
        assert!(dst_stat.is_ok());
        let dst_stat = match dst_stat.unwrap() {
            Status::Neuron(info) => Some(info),
            _ => None,
        };
        assert!(dst_stat.is_some());
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

        let monitoring_log = net.pop_monitoring_store().await;

        assert_eq!(monitoring_log.len(), 3);
        for status in monitoring_log {
            match status {
                Status::Neuron(neuron_info) => {
                    assert!(neuron_info.timestamp.timestamp().is_positive());
                    assert_eq!(neuron_info.accumulator, 1);
                    assert_eq!(neuron_info.dendrite_connected_count, 1);
                    assert_eq!(neuron_info.dendrite_count, 1);
                    assert_eq!(neuron_info.dendrite_hit_count, 0);
                    assert_eq!(neuron_info.receiver_count, 1);
                }
                Status::Port(port_info) => {
                    assert!(port_info.timestamp.timestamp().is_positive());
                    assert_eq!(port_info.hit_count, 1);
                }
            }
        }
    }
}
