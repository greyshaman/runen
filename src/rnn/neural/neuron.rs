//! The Neuron is model of biological neuron cell within organelles

use std::cmp::max;
use std::cmp::min;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::sync::Weak;

use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use tokio::sync::RwLock;
use tokio::sync::RwLockWriteGuard;
use tokio::task;

use super::dendrite::Dendrite;
use super::dendrite::InputCfg;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::layouts::network::Network;

#[derive(Debug)]
pub struct NeuronConfig {
    pub id: String,
    pub input_configs: Vec<InputCfg>,
}

/// Current neuron state.
pub struct NeuronState {
    /// The neuron id.
    pub id: String,

    /// Total number of dendrites.
    pub dendrite_count: usize,

    /// The number of dendrites have incoming connection.
    pub dendrite_connected_count: usize,

    /// The number of dendrites that receive signals after the last accumulator reset.
    pub dendrite_hit_count: usize,

    /// The number of neuron resets.
    pub reset_count: u64,

    /// The number of total signal to neuron hits.
    pub hit_count: u64,

    /// The accumulator value
    pub accumulator: i16,

    /// The number of active receivers.
    pub receiver_count: usize,

    /// The sum of dendrites weight
    pub total_weight: i16,
}

/// The neuron's core, which contains data that is shared between concurrent tasks.
#[derive(Debug)]
pub struct NeuronCore {
    /// The accumulator need to sum incoming signals.
    accumulator: i16,

    /// The counter of neuron resets.
    reset_counter: u64,

    /// The counter of signal hits.
    hit_counter: u64,

    /// Neurons input which are received the signals from outside.
    dendrites: HashMap<usize, Dendrite>,

    /// This structure records which dendrites receive signals from.
    /// If the signals are received on all inputs, or if the signal
    /// comes back to some input, the cumulative result is sent out
    /// and the signal processing starts in a different way.
    input_hits: HashSet<usize>,

    /// The accumulated result is transmitted through the axon using
    /// a broadcast channel and sent to other recipients.
    axon: Arc<Option<Arc<Sender<u8>>>>,
}

#[derive(Debug)]
pub struct Neuron {
    id: String,
    network: Weak<Network>,
    core: Arc<RwLock<NeuronCore>>,
}

impl Neuron {
    /// Create new empty neuron
    pub fn new(id: &str, network: Arc<Network>) -> Self {
        let core = NeuronCore {
            accumulator: 1,
            reset_counter: 0,
            hit_counter: 0,
            dendrites: HashMap::new(),
            input_hits: HashSet::new(),
            axon: Arc::new(None),
        };
        Neuron {
            id: String::from(id),
            network: Arc::downgrade(&network),
            core: Arc::new(RwLock::new(core)),
        }
    }

    pub async fn receive(core: &Arc<RwLock<NeuronCore>>, signal: u8, port: usize) {
        let mut w_core = core.write().await;
        {
            w_core.hit_counter += 1;
        }

        let input = w_core.dendrites.get_mut(&port).unwrap();

        let signal = Self::synapse_accept_signal(input, signal);

        let signal: i16 = Self::dendrite_weighting_signal(input, signal);
        // Neurosoma responsibility
        Self::neurosome_process_signal(w_core, signal, port);
    }

    /// Send only positive signal otherwise suppress transmission. Need to stop endless looping zero signals
    pub fn send(axon: Arc<Sender<u8>>, signal: u8) -> usize {
        let mut count: usize = 0;
        if signal > 0 {
            let sends = axon.send(signal).unwrap();
            count += sends;
        }
        count
    }

    /// Creates a new neuron with all the necessary components
    /// in the specified configuration.
    pub async fn build(config: NeuronConfig, network: Arc<Network>) -> Arc<Neuron> {
        let NeuronConfig { id, input_configs } = config;
        let neuron = Neuron::new(&id, network);
        neuron.config(input_configs).await;

        Arc::new(neuron)
    }

    // TODO add snapshot

    /// Config new neuron
    pub async fn config(&self, settings: Vec<InputCfg>) {
        let mut w_core = self.core.write().await;
        w_core.dendrites.clear();
        w_core.input_hits.clear();
        w_core.accumulator = 1;
        for (port, i_cfg) in settings.iter().enumerate() {
            let dendrite = Dendrite {
                config: InputCfg {
                    capacity_max: i_cfg.capacity_max,
                    regeneration: i_cfg.regeneration,
                    weight: i_cfg.weight,
                },
                syn_capacity: i_cfg.capacity_max,
                connected: None,
                input: None,
            };
            w_core.dendrites.insert(port, dendrite);
        }
    }

    /// Provides access to a channel (axon) for receiving signals from a given neuron.
    pub async fn provide_output(&self) -> Arc<RwLock<Receiver<u8>>> {
        let rx = {
            let mut w_core = self.core.write().await;
            w_core.axon.clone().as_deref().map_or_else(
                || {
                    let (tx, rx) = broadcast::channel::<u8>(5);
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
        receiver: Arc<RwLock<Receiver<u8>>>,
    ) -> Result<(), Box<dyn Error>> {
        {
            let mut w_core = self.core.write().await;
            if let Some(dendrite) = w_core.dendrites.get_mut(&port) {
                if dendrite.connected.is_none() {
                    dendrite.connected = Some(src_id.to_string());
                    dendrite.syn_capacity = dendrite.config.capacity_max;
                    // dendrite.input = Some(Arc::clone(&receiver));
                    dendrite.input = Some(receiver);
                } else {
                    return Err(Box::new(RnnError::IdBusy(format!(
                        "input port {} already connected",
                        port
                    ))));
                }
            } else {
                return Err(Box::new(RnnError::IdNotFound));
            }
        }

        {
            let core_cloned = Arc::clone(&self.core);
            let input_opt = {
                let r_core = self.core.read().await;
                r_core
                    .dendrites
                    .get(&port)
                    .and_then(|dendrite| dendrite.input.clone())
            };

            if let Some(synapse) = input_opt {
                let synapse_cloned = Arc::clone(&synapse);
                let _task_handler = task::spawn(async move {
                    let mut w_synapse = synapse_cloned.write().await;
                    while let Ok(signal) = w_synapse.recv().await {
                        Self::receive(&core_cloned, signal, port).await;
                    }
                });
                Ok(())
            } else {
                Err(Box::new(RnnError::IdNotFound))
            }
        }
    }

    pub fn get_network(&self) -> Option<Arc<Network>> {
        self.network.upgrade()
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub async fn get_input_ports_len(&self) -> usize {
        self.core.read().await.dendrites.len()
    }

    /// Request neuron state.
    pub async fn get_state(&self) -> NeuronState {
        let r_core = self.core.read().await;
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

        NeuronState {
            id: self.id.clone(),
            dendrite_count,
            dendrite_connected_count,
            dendrite_hit_count,
            accumulator,
            receiver_count,
            reset_count,
            hit_count,
            total_weight,
        }
    }

    fn get_connected_input_ports_len(dendrites: &HashMap<usize, Dendrite>) -> usize {
        dendrites
            .values()
            .filter(|dendrite| dendrite.connected.is_some())
            .count()
    }

    #[inline]
    fn synapse_accept_signal(input: &mut Dendrite, signal: u8) -> u8 {
        // Synapse responsibility
        let signal: u8 = min(signal, input.syn_capacity);
        input.syn_capacity -= signal;
        input.syn_capacity = min(
            input.syn_capacity + input.config.regeneration,
            input.config.capacity_max,
        );
        signal
    }

    #[inline]
    fn dendrite_weighting_signal(input: &Dendrite, signal: u8) -> i16 {
        (signal as i16) * input.config.weight
    }

    #[inline]
    fn neurosome_process_signal(
        mut w_core: RwLockWriteGuard<NeuronCore>,
        signal: i16,
        port: usize,
    ) {
        if w_core.input_hits.contains(&port) {
            let new_signal = max(w_core.accumulator, 0) as u8;
            w_core.accumulator = signal + 1;
            w_core.reset_counter += 1;
            w_core.input_hits.clear();
            w_core.input_hits.insert(port);
            w_core
                .axon
                .as_ref()
                .clone()
                .map(|axon| Self::send(Arc::clone(&axon), new_signal));
        } else {
            w_core.accumulator += signal as i16;
            w_core.input_hits.insert(port);
            if w_core.input_hits.len() >= Self::get_connected_input_ports_len(&w_core.dendrites) {
                let new_signal = max(w_core.accumulator, 0) as u8;
                w_core.accumulator = 1;
                w_core.reset_counter += 1;
                w_core.input_hits.clear();
                w_core
                    .axon
                    .as_ref()
                    .clone()
                    .map(|axon| Self::send(Arc::clone(&axon), new_signal));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod for_default_neuron {
        use crate::rnn::{
            common::spec_type::SpecificationType,
            tests::fixtures::{new_network_fixture, new_neuron_fixture},
        };

        use super::*;

        #[tokio::test]
        async fn status_fn_should_return_correct_neuron_state() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), vec![]).await;

            let state = neuron.get_state().await;

            assert_eq!(state.id, neuron.get_id());
            assert_eq!(state.dendrite_count, 1);
            assert_eq!(state.dendrite_connected_count, 0);
            assert_eq!(state.dendrite_hit_count, 0);
            assert_eq!(state.accumulator, 1);
            assert_eq!(state.receiver_count, 0);
            assert_eq!(state.reset_count, 0);
            assert_eq!(state.hit_count, 0);
            assert_eq!(state.total_weight, 1);
        }

        #[tokio::test]
        async fn get_input_ports_len_fn_should_return_one() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), vec![]).await;

            assert_eq!(neuron.get_input_ports_len().await, 1);
        }

        #[tokio::test]
        async fn get_connected_input_ports_len_fn_should_return_zero() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), vec![]).await;

            let r_core = neuron.core.read().await;

            assert_eq!(Neuron::get_connected_input_ports_len(&r_core.dendrites), 0);
        }

        #[tokio::test]
        async fn get_id_fn_should_return_correct_id() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), vec![]).await;

            let neuron_id = neuron.get_id();
            assert!(SpecificationType::Neuron.is_id_valid(neuron_id.as_str()))
        }

        #[tokio::test]
        async fn get_network_fn_should_return_network_with_correct_id() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), vec![]).await;

            assert!(neuron.get_network().is_some());
            assert_eq!(neuron.get_network().unwrap().get_id(), net.get_id());
        }

        #[tokio::test]
        async fn number_of_dendrites_should_be_the_same_as_in_config() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), vec![]).await;

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
        async fn provide_output_fn_should_return_receiver() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), vec![]).await;

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

            let state = neuron.get_state().await;
            assert_eq!(state.receiver_count, 1);
        }

        #[tokio::test]
        async fn connect_fn_should_perform_connection_to_input_port() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), vec![]).await;

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

            let state = neuron.get_state().await;
            assert_eq!(state.dendrite_connected_count, 1);
            assert_eq!(state.receiver_count, 1);
            assert_eq!(state.hit_count, 1);
            assert_eq!(state.reset_count, 1);
        }

        #[tokio::test]
        async fn link_to_fn_should_perform_link_to_another_neuron() {
            let net = Arc::new(new_network_fixture());
            let neuron1 = new_neuron_fixture(net.clone(), vec![]).await;
            let neuron2 = new_neuron_fixture(net.clone(), vec![]).await;

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

            let state = neuron1.get_state().await;
            assert_eq!(state.reset_count, 1);
            assert_eq!(state.hit_count, 1);
        }

        #[tokio::test]
        async fn get_network_fn_should_return_contained_network() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), vec![]).await;

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
        async fn get_input_ports_len_fn_should_return_configured_number_of_dendrites() {
            let net = Arc::new(new_network_fixture());
            let neuron = new_neuron_fixture(net.clone(), gen_neuron_input_config_fixture(3)).await;

            let state = neuron.get_state().await;

            assert_eq!(neuron.get_input_ports_len().await, 3);
            assert_eq!(state.dendrite_connected_count, 0);
            assert_eq!(state.total_weight, 6);
        }

        #[tokio::test]
        async fn config_fn_should_add_new_dendrites() {
            let net = Arc::new(new_network_fixture());
            let inputs_config1 = gen_neuron_input_config_fixture(3);
            let inputs_config2 = gen_neuron_input_config_fixture(2);
            let neuron = new_neuron_fixture(net.clone(), inputs_config1).await;
            assert_eq!(neuron.get_input_ports_len().await, 3);

            neuron.config(inputs_config2).await;
            assert_eq!(neuron.get_input_ports_len().await, 2);
        }
    }
}
