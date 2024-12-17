//! The Neuron is model of biological neuron cell within organelles

use std::cmp::max;
use std::cmp::min;
use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::sync::Weak;

use chrono::Utc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::sync::RwLockWriteGuard;
use tokio::task::JoinHandle;
use tokio_util::task::TaskTracker;

use super::dendrite::Dendrite;
use crate::rnn::common::command::NeuronCommand;
use crate::rnn::common::input_cfg::InputCfg;
use crate::rnn::common::network_cfg::NeuronCfg;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::signal::Signal;
use crate::rnn::common::signal::Weight;
use crate::rnn::common::status::NeuronInfo;
use crate::rnn::common::status::Status;
use crate::rnn::layouts::network::MonitoringMode;
use crate::rnn::layouts::network::Network;

/// The neuron's core, which contains data that is shared between concurrent tasks.
#[derive(Debug)]
pub struct NeuronCore {
    /// The neuron Bias
    bias: Weight,

    /// The accumulator need to sum incoming signals.
    accumulator: Weight,

    /// The counter of neuron resets.
    reset_counter: u64,

    /// The counter of signal hits.
    hit_counter: u64,

    /// Neurons input which are received the signals from outside.
    dendrites: BTreeMap<usize, Dendrite>,

    /// This structure records which dendrites receive signals from.
    /// If the signals are received on all inputs, or if the signal
    /// comes back to some input, the cumulative result is sent out
    /// and the signal processing starts in a different way.
    input_hits: HashSet<usize>,

    /// The accumulated result is transmitted through the axon using
    /// a broadcast channel and sent to other recipients.
    axon: Arc<Option<Arc<Sender<Signal>>>>,

    /// The task handlers
    synapse_connection_handlers: HashMap<usize, JoinHandle<()>>,

    /// Each synapse that has a connection is waiting for signals in a separate task.
    /// Tracker is a collection of these tasks
    receivers_task_tracker: TaskTracker,

    monitoring_sender: mpsc::WeakSender<Status>,

    monitoring_mode: MonitoringMode,
    // commands_receiver: broadcast::Receiver<NeuronCommands>,
}

#[derive(Debug)]
pub struct Neuron {
    id: String,
    network: Weak<Network>,
    core: Arc<RwLock<NeuronCore>>,
}

impl Neuron {
    /// Creates a new neuron.
    /// This method has been moved to a private view, as it only creates
    /// the essence of a neuron without any connection to the control command channels.
    async fn new(
        id: &str,
        bias: Weight,
        network: Arc<Network>,
        monitoring_sender: mpsc::WeakSender<Status>,
    ) -> Self {
        let core = NeuronCore {
            bias,
            accumulator: 0,
            reset_counter: 0,
            hit_counter: 0,
            dendrites: BTreeMap::new(),
            input_hits: HashSet::new(),
            axon: Arc::new(None),
            receivers_task_tracker: TaskTracker::new(),
            synapse_connection_handlers: HashMap::new(),
            monitoring_sender,
            monitoring_mode: network.get_monitoring_mode().await,
        };

        Neuron {
            id: String::from(id),
            network: Arc::downgrade(&network),
            core: Arc::new(RwLock::new(core)),
        }
    }

    /// Creates a new neuron with all the necessary components
    /// in the specified configuration.
    pub async fn build(network: Arc<Network>, config: NeuronCfg) -> Arc<Neuron> {
        let NeuronCfg {
            id,
            input_configs,
            bias,
        } = config;

        let mut commands_receiver = network.get_commands_receiver();
        let monitoring_sender = network.get_monitoring_sender();

        let neuron = Neuron::new(&id, bias, network, monitoring_sender).await;
        neuron.config(input_configs).await;

        let neuron = Arc::new(neuron);

        let neuron_cloned = neuron.clone();

        let _ = neuron
            .core
            .read()
            .await
            .receivers_task_tracker
            .spawn(async move {
                while let Ok(command) = commands_receiver.recv().await {
                    match command {
                        NeuronCommand::SwitchMonitoringMode(mode) => {
                            neuron_cloned.switch_monitoring_mode(mode).await;
                        }
                    }
                }
            });

        neuron
    }

    /// Receive signal by neuron through port
    pub async fn receive(
        id: &str,
        core: &Arc<RwLock<NeuronCore>>,
        signal: Signal,
        port: usize,
    ) -> Result<(), Box<dyn Error>> {
        let t_handler = {
            let mut w_core = core.write().await;
            {
                w_core.hit_counter += 1;
            }

            let monitoring_mode = w_core.monitoring_mode.clone();
            if let Some(input) = w_core.dendrites.get_mut(&port) {
                let signal = Self::synapse_accept_signal(input, signal);

                let signal: Weight = Self::dendrite_weighting_signal(input, signal);

                Self::process_signal(w_core, signal, port)?;

                if monitoring_mode == MonitoringMode::Monitoring {
                    let id = String::from(id);
                    let core_cloned = core.clone();
                    tokio::task::spawn(async move {
                        Self::send_monitoring_statistics(&id, &core_cloned).await
                    })
                } else {
                    tokio::task::spawn(async { Ok(()) })
                }
            } else {
                tokio::task::spawn(async move { Err(RnnError::DendriteNotFound(port)) })
            }
        };

        t_handler.await?.map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    /// Send only positive signal otherwise suppress transmission. Need to stop endless looping zero signals
    pub fn send(axon: Arc<Sender<Signal>>, signal: Signal) -> Result<usize, Box<dyn Error>> {
        if signal > 0 {
            axon.send(signal)
                .map_err(|_| Box::new(RnnError::SignalSendError) as Box<dyn Error>)
        } else {
            Err(Box::new(RnnError::SignalSuppressed))
        }
    }

    /// Config new neuron
    pub async fn config(&self, settings: Vec<InputCfg>) {
        let mut w_core = self.core.write().await;
        w_core.dendrites.clear();
        w_core.input_hits.clear();
        w_core.accumulator = 1;
        for (port, input_cfg) in settings.iter().enumerate() {
            let dendrite = Dendrite {
                config: InputCfg {
                    capacity_max: input_cfg.capacity_max,
                    regeneration: input_cfg.regeneration,
                    weight: input_cfg.weight,
                },
                synapse_capacity: input_cfg.capacity_max,
                connected: None,
                synapse: None,
            };
            w_core.dendrites.insert(port, dendrite);
        }
    }

    /// Get current neuron config
    pub async fn get_config(&self) -> NeuronCfg {
        let r_core = self.core.read().await;
        let input_configs = r_core
            .dendrites
            .iter()
            .map(|(_idx, dendrite)| dendrite.config.clone())
            .collect::<Vec<InputCfg>>();

        NeuronCfg {
            id: self.get_id(),
            bias: r_core.bias,
            input_configs,
        }
    }

    pub async fn switch_monitoring_mode(&self, mode: MonitoringMode) {
        let mut w_core = self.core.write().await;
        w_core.monitoring_mode = mode;
    }

    pub async fn get_monitoring_mode(&self) -> MonitoringMode {
        self.core.read().await.monitoring_mode.clone()
    }

    /// Provides access to a channel (axon) for receiving signals from a given neuron.
    pub async fn provide_output(&self) -> Arc<RwLock<Receiver<Signal>>> {
        let rx = {
            let mut w_core = self.core.write().await;
            w_core.axon.clone().as_deref().map_or_else(
                || {
                    let (tx, rx) = broadcast::channel::<Signal>(5);
                    w_core.axon = Arc::new(Some(Arc::new(tx)));
                    rx
                },
                |tx| tx.subscribe(),
            )
        };

        Arc::new(RwLock::new(rx))
    }

    /// Link to a specific input (synapse) of a neuron.
    /// A synapse can only have one connection.
    /// However, a neuron can have many synapses at the same time.
    pub async fn link_to(&self, party: Arc<Neuron>, port: usize) -> Result<(), Box<dyn Error>> {
        let out = self.provide_output().await;
        let party_id = party.get_id();
        if party_id == self.id {
            let r_core = self.core.read().await;
            let dendrites = &r_core.dendrites;
            let self_connected_dendrites_count = dendrites
                .iter()
                .filter(|(_, d)| {
                    d.connected
                        .as_ref()
                        .is_some_and(|connected| connected.to_string() == self.id)
                })
                .count();
            if r_core.dendrites.len() < 2 || self_connected_dendrites_count > 0_usize {
                return Err(Box::new(RnnError::ClosedLoop));
            }
        }
        party.connect(&self.id, port, out).await
    }

    pub async fn connect(
        &self,
        src_id: &str,
        port: usize,
        receiver: Arc<RwLock<Receiver<Signal>>>,
    ) -> Result<(), Box<dyn Error>> {
        {
            // exclusive lock core
            let mut w_core = self.core.write().await;

            // try get synapse by port number
            if let Some(dendrite) = w_core.dendrites.get_mut(&port) {
                // check if synapse not connected yet
                if dendrite.connected.is_none() {
                    // Store connection party id
                    dendrite.connected = Some(src_id.to_string());

                    // Set synapse current capacity
                    dendrite.synapse_capacity = dendrite.config.capacity_max;

                    // Set receiver half of connecting channel

                    dendrite.synapse = Some(receiver);

                    let synapse = dendrite.synapse.as_ref().unwrap().clone();
                    let core_cloned = self.core.clone();
                    let id_cloned = self.get_id();

                    // Check if already has task_handler at specified port number
                    if let Entry::Occupied(task_entry) =
                        w_core.synapse_connection_handlers.entry(port)
                    {
                        let task_handler = task_entry.get();
                        // abort listener task
                        task_handler.abort();

                        // clear entry
                        task_entry.remove();
                    }
                    let task_handler = w_core.receivers_task_tracker.spawn(async move {
                        let mut w_synapse = synapse.write().await;
                        while let Ok(signal) = w_synapse.recv().await {
                            let write_me_into_log =
                                Self::receive(&id_cloned, &core_cloned, signal, port).await;
                        }
                    });

                    w_core
                        .synapse_connection_handlers
                        .insert(port, task_handler);

                    Ok(())
                } else {
                    // Synapse already has connection then notify about it
                    Err(Box::new(RnnError::PortBusy(format!(
                        "input port {} already connected",
                        port
                    ))))
                }
            } else {
                // Neuron does not have input with specified port number
                Err(Box::new(RnnError::DendriteNotFound(port)))
            }
        }
    }

    /// Get network which contains this neuron
    pub fn get_network(&self) -> Option<Arc<Network>> {
        self.network.upgrade()
    }

    /// Get neuron's id
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    /// Get number of dendrites
    pub async fn get_input_ports_len(&self) -> usize {
        self.core.read().await.dendrites.len()
    }

    pub fn get_core(&self) -> Arc<RwLock<NeuronCore>> {
        self.core.clone()
    }

    /// send neuron state to monitoring network receiver.
    pub async fn send_monitoring_statistics(
        id: &str,
        core: &Arc<RwLock<NeuronCore>>,
    ) -> Result<(), RnnError> {
        let statistics = Self::prepare_status(id, core).await;

        let r_core = core.read().await;
        if let Some(sender) = r_core.monitoring_sender.upgrade() {
            sender.send(statistics).await.map_err(|e| {
                RnnError::MonitoringChannelClosed(format!(
                    "Statistics sending error due channel closed. Lost stat: {:?}",
                    e
                ))
            })
        } else {
            Ok(())
        }
    }

    pub async fn prepare_status(id: &str, core: &Arc<RwLock<NeuronCore>>) -> Status {
        let r_core = core.read().await;
        let dendrite_count = r_core.dendrites.len();
        let dendrite_connected_count = Self::get_connected_input_ports_len(&r_core.dendrites);
        let dendrite_hit_count = r_core.input_hits.len();
        let accumulator = r_core.accumulator;
        let receiver_count = if let Some(axon) = r_core.axon.as_ref() {
            axon.receiver_count()
        } else {
            0
        };
        let reset_count = r_core.reset_counter;
        let hit_count = r_core.hit_counter;
        let total_weight = r_core.dendrites.values().map(|d| d.config.weight).sum();
        let now = Utc::now();

        Status::Neuron(NeuronInfo {
            timestamp: now,
            id: id.to_string(),
            dendrite_count,
            dendrite_connected_count,
            dendrite_hit_count,
            accumulator,
            receiver_count,
            reset_count,
            hit_count,
            total_weight,
        })
    }

    fn get_connected_input_ports_len(dendrites: &BTreeMap<usize, Dendrite>) -> usize {
        dendrites
            .values()
            .filter(|dendrite| dendrite.connected.is_some())
            .count()
    }

    #[inline]
    fn synapse_accept_signal(input: &mut Dendrite, signal: Signal) -> Signal {
        // Synapse responsibility
        let signal: Signal = min(signal, input.synapse_capacity);
        input.synapse_capacity -= signal;
        input.synapse_capacity = min(
            input.synapse_capacity + input.config.regeneration,
            input.config.capacity_max,
        );
        signal
    }

    #[inline]
    fn dendrite_weighting_signal(input: &Dendrite, signal: Signal) -> Weight {
        (signal as Weight) * input.config.weight
    }

    #[inline]
    fn process_signal(
        mut w_core: RwLockWriteGuard<NeuronCore>,
        weighted_signal: Weight,
        port: usize,
    ) -> Result<(), Box<dyn Error>> {
        if w_core.input_hits.contains(&port) {
            // The Repeat signal case
            // A signal is being prepared for output through the axon
            let output_signal = max(w_core.accumulator, 0) as Signal;

            // Reset accumulator with new signal plus excitation level
            w_core.accumulator = weighted_signal + w_core.bias;

            // Increment neuron resets counter
            w_core.reset_counter += 1;

            // Reset hits register
            w_core.input_hits.clear();

            // Store fact of signal hit to current port
            w_core.input_hits.insert(port);

            // check if axon has connections
            if let Some(axon) = w_core.axon.as_ref().clone() {
                // send output signal through the axon
                Self::send(axon.clone(), output_signal).map(|_| ())
            } else {
                // Axon does not have any connections
                Err(Box::new(RnnError::DeadEndAxon))
            }
        } else {
            // Add signal value to accumulator
            w_core.accumulator += weighted_signal as i16;

            // Store fact of signal hit to current port
            w_core.input_hits.insert(port);

            // Check if all activated synapses had signal hits
            if w_core.input_hits.len() >= Self::get_connected_input_ports_len(&w_core.dendrites) {
                // All activated synapses received signals
                // A signal is being prepared for output through the axon
                let output_signal = max(w_core.accumulator, 0) as u8;

                // Reset accumulator
                w_core.accumulator = w_core.bias as Weight;

                // Increment neuron resets counter
                w_core.reset_counter += 1;

                // Reset hits register
                w_core.input_hits.clear();

                // check if axon has connections
                if let Some(axon) = w_core.axon.as_ref().clone() {
                    // send output signal through the axon
                    Self::send(axon.clone(), output_signal).map(|_| ())
                } else {
                    // Axon does not have any connections
                    Err(Box::new(RnnError::DeadEndAxon))
                }
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod for_default_neuron {
        use std::time::Duration;

        use crate::rnn::{
            common::spec_type::SpecificationType,
            layouts::network::MonitoringMode,
            tests::fixtures::{new_network_fixture, new_neuron_fixture},
        };

        use super::*;

        #[tokio::test]
        async fn fn_status_should_return_correct_neuron_state() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;

            let stat = net
                .get_current_neuron_status(&neuron.get_id())
                .await
                .unwrap();
            let stat = match stat {
                Status::Neuron(info) => Some(info),
                _ => None,
            };
            assert!(stat.is_some());
            let stat = stat.unwrap();

            assert!(stat.timestamp.timestamp().is_positive());
            assert_eq!(stat.id, neuron.get_id());
            assert_eq!(stat.dendrite_count, 1);
            assert_eq!(stat.dendrite_connected_count, 0);
            assert_eq!(stat.dendrite_hit_count, 0);
            assert_eq!(stat.accumulator, 1);
            assert_eq!(stat.receiver_count, 0);
            assert_eq!(stat.reset_count, 0);
            assert_eq!(stat.hit_count, 0);
            assert_eq!(stat.total_weight, 1);
        }

        #[tokio::test]
        async fn fn_switch_monitoring_mode_should_change_monitoring_mode() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;

            assert_eq!(neuron.get_monitoring_mode().await, MonitoringMode::None);

            neuron
                .switch_monitoring_mode(MonitoringMode::Monitoring)
                .await;
            assert_eq!(
                neuron.get_monitoring_mode().await,
                MonitoringMode::Monitoring
            );

            neuron.switch_monitoring_mode(MonitoringMode::None).await;
            assert_eq!(neuron.get_monitoring_mode().await, MonitoringMode::None);
        }

        #[tokio::test]
        async fn sending_switch_monitoring_mode_by_command_should_change_neuron_mode_to_same_mode()
        {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;

            assert_eq!(neuron.get_monitoring_mode().await, MonitoringMode::None);

            net.set_monitoring_mode(MonitoringMode::Monitoring).await;

            tokio::time::sleep(Duration::from_millis(1)).await; // Time required to apply changes in all neurons

            assert_eq!(
                neuron.get_monitoring_mode().await,
                MonitoringMode::Monitoring
            );
        }

        #[tokio::test]
        async fn fn_get_input_ports_len_should_return_one() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;

            assert_eq!(neuron.get_input_ports_len().await, 1);
        }

        #[tokio::test]
        async fn fn_get_connected_input_ports_len_should_return_zero() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;

            let r_core = neuron.core.read().await;

            assert_eq!(Neuron::get_connected_input_ports_len(&r_core.dendrites), 0);
        }

        #[tokio::test]
        async fn fn_get_id_should_return_correct_id() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;

            let neuron_id = neuron.get_id();
            assert!(SpecificationType::Neuron.is_id_valid(neuron_id.as_str()))
        }

        #[tokio::test]
        async fn fn_get_network_should_return_network_with_correct_id() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;

            assert!(neuron.get_network().is_some());
            assert_eq!(neuron.get_network().unwrap().get_id(), net.get_id());
        }

        #[tokio::test]
        async fn number_of_dendrites_should_be_the_same_as_in_config() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;

            neuron
                .config(vec![
                    InputCfg {
                        capacity_max: 1,
                        regeneration: 1,
                        weight: 1,
                    },
                    InputCfg {
                        capacity_max: 2,
                        regeneration: 1,
                        weight: 2,
                    },
                ])
                .await;

            assert_eq!(neuron.get_input_ports_len().await, 2);
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn fn_provide_output_should_return_receiver() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;
            let neuron_id = neuron.get_id();

            let receiver_orig = neuron.provide_output().await;

            {
                let r_core = neuron.core.read().await;
                let axon_opt = r_core.axon.as_ref().clone();

                assert!(axon_opt.is_some());

                let axon = axon_opt.unwrap();
                let receiver = receiver_orig.clone();
                let mut w_rx = receiver.write().await;
                let res = axon.send(7);
                if let Ok(rx_signal) = w_rx.recv().await {
                    assert_eq!(rx_signal, 7);
                }
                assert!(res.is_ok());
            }

            if let Status::Neuron(stat) = net.get_current_neuron_status(&neuron_id).await.unwrap() {
                assert_eq!(stat.receiver_count, 1);
            } else {
                assert!(false);
            }
        }

        #[tokio::test]
        async fn fn_connect_should_perform_connection_to_input_port() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;
            let neuron_id = neuron.get_id();

            let monitor = neuron.provide_output().await;
            let mut w_monitor = monitor.write().await;

            let (tx, rx) = broadcast::channel(2);
            let res = neuron.connect("M0I0", 0, Arc::new(RwLock::new(rx))).await;
            assert!(res.is_ok());

            let res = tx.send(1);
            if let Ok(rx_signal) = w_monitor.recv().await {
                assert_eq!(rx_signal, 2); // Signal is 2 because neuron add 1 as activity level
            }
            assert!(res.is_ok());

            if let Status::Neuron(stat) = net.get_current_neuron_status(&neuron_id).await.unwrap() {
                assert_eq!(stat.dendrite_connected_count, 1);
                assert_eq!(stat.receiver_count, 1);
                assert_eq!(stat.hit_count, 1);
                assert_eq!(stat.reset_count, 1);
            } else {
                assert!(false);
            }
        }

        #[tokio::test]
        async fn fn_link_to_should_perform_link_to_another_neuron() {
            let net = Arc::new(new_network_fixture());
            let neuron1 = new_neuron_fixture(net.clone(), 1, vec![]).await;
            let neuron2 = new_neuron_fixture(net.clone(), 1, vec![]).await;

            let monitor = neuron2.provide_output().await;
            let mut w_monitor = monitor.write().await;

            let (tx, rx) = broadcast::channel(1);
            assert!(neuron1
                .connect("M0I0", 0, Arc::new(RwLock::new(rx)))
                .await
                .is_ok());

            assert!(neuron1.link_to(neuron2.clone(), 0).await.is_ok());

            let res = tx.send(1);
            if let Ok(rx_signal) = w_monitor.recv().await {
                assert_eq!(rx_signal, 2);
            }
            assert!(res.is_ok());

            let stat = net
                .get_current_neuron_status(&neuron1.get_id())
                .await
                .unwrap();
            if let Status::Neuron(info) = stat {
                assert_eq!(info.reset_count, 1);
                assert_eq!(info.hit_count, 1);
            } else {
                assert!(false);
            }
        }

        #[tokio::test]
        async fn fn_get_network_should_return_contained_network() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), 1, vec![]).await;

            let network = neuron.get_network();
            assert!(network.is_some());
            assert_eq!(network.unwrap().get_id(), net.get_id());
        }
    }

    mod for_non_default_neuron {

        use crate::rnn::tests::fixtures::{
            gen_neuron_input_config_fixture, new_network_fixture, new_neuron_fixture,
        };

        use super::*;

        #[tokio::test]
        async fn fn_get_input_ports_len_should_return_configured_number_of_dendrites() {
            let net = Arc::new(new_network_fixture());
            let neuron =
                new_neuron_fixture(net.clone(), 1, gen_neuron_input_config_fixture(3)).await;

            let stat = net
                .get_current_neuron_status(&neuron.get_id())
                .await
                .unwrap();

            if let Status::Neuron(info) = stat {
                assert_eq!(neuron.get_input_ports_len().await, 3);
                assert_eq!(info.dendrite_connected_count, 0);
                assert_eq!(info.total_weight, 6);
            } else {
                assert!(false);
            }
        }

        #[tokio::test]
        async fn fn_config_should_add_new_dendrites() {
            let net = Arc::new(new_network_fixture());
            let inputs_config1 = gen_neuron_input_config_fixture(3);
            let inputs_config2 = gen_neuron_input_config_fixture(2);
            let neuron = new_neuron_fixture(net.clone(), 1, inputs_config1).await;
            assert_eq!(neuron.get_input_ports_len().await, 3);

            neuron.config(inputs_config2).await;
            assert_eq!(neuron.get_input_ports_len().await, 2);
        }

        #[tokio::test]
        async fn fn_get_config_should_return_correct_config() {
            let net = Arc::new(new_network_fixture());
            let input_config = gen_neuron_input_config_fixture(3);
            let neuron = new_neuron_fixture(net.clone(), 1, input_config).await;
            let expected_id = &neuron.get_id();
            let cfg = neuron.get_config().await;

            assert_eq!(&cfg.id, expected_id);
            assert_eq!(&cfg.input_configs.len(), &3);
            assert!(&cfg.input_configs.iter().enumerate().all(|(idx, cfg)| {
                let current_value = (idx + 1) as u8;
                cfg.capacity_max == current_value
                    && cfg.regeneration == current_value
                    && cfg.weight == current_value as i16
            }));
        }
    }
}
