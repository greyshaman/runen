use core::fmt;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use regex::Regex;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::utils::gen_id_by_spec_type;
use crate::rnn::neural::dendrite::InputCfg;
use crate::rnn::neural::neuron::{Neuron, NeuronConfig};

static mut ID_COUNTER: AtomicUsize = AtomicUsize::new(0_usize);
static CHANNEL_CAPACITY: usize = 5;

/// Network is a high level container to other containers (neurons)
#[derive(Debug)]
pub struct Network {
    id: String,
    neurons: Mutex<BTreeMap<String, Arc<Neuron>>>,
    input_interface: Arc<Mutex<BTreeMap<usize, Arc<Mutex<Sender<u8>>>>>>,
    output_interface: Arc<Mutex<BTreeMap<usize, Arc<Mutex<Receiver<u8>>>>>>,
    processing_registers: Arc<Mutex<HashMap<usize, JoinHandle<()>>>>,
    results: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl Network {
    pub fn new() -> Result<Network, Box<dyn Error>> {
        gen_id_by_spec_type(
            "",
            unsafe { ID_COUNTER.fetch_add(1, Ordering::Relaxed) },
            &SpecificationType::Network,
        )
        .map(|id| Network {
            id,
            neurons: Mutex::new(BTreeMap::new()),
            input_interface: Arc::new(Mutex::new(BTreeMap::new())),
            output_interface: Arc::new(Mutex::new(BTreeMap::new())),
            processing_registers: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub async fn get_neuron(&self, id: &str) -> Option<Arc<Neuron>> {
        let g_neurons = self.neurons.lock().await;
        g_neurons.get(id).map(|neuron| Arc::clone(neuron))
    }

    pub async fn create_neuron(
        &self,
        network: Arc<Network>,
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

        let neuron_config = NeuronConfig {
            id: new_id.clone(),
            input_configs,
        };
        let mut g_neurons = self.neurons.lock().await;
        match g_neurons.entry(new_id.clone()) {
            Entry::Vacant(entry) => Ok(Arc::clone(
                entry.insert(Neuron::build(neuron_config, Arc::clone(&network)).await),
            )),
            Entry::Occupied(_) => Err(Box::new(RnnError::IdBusy(format!(
                "Id {} already used",
                new_id
            )))),
        }
    }

    pub async fn get_available_neuron_id(&self) -> usize {
        let g_neurons = self.neurons.lock().await;
        g_neurons.keys().last().map_or(0, |id| {
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
        let mut g_neurons = self.neurons.lock().await;
        match g_neurons.remove(id) {
            Some(_) => Ok(()),
            None => Err(Box::new(RnnError::IdNotFound)),
        }
    }

    pub async fn has_neuron(&self, id: &str) -> bool {
        self.neurons.lock().await.contains_key(id)
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
        self.neurons.lock().await.len()
    }

    /// Send signal to port connected to synapse
    pub async fn input(&self, signal: u8, port: usize) -> Result<usize, Box<dyn Error>> {
        if let Some(synapse) = self.input_interface.lock().await.get(&port) {
            let g_synapse = synapse.lock().await;
            g_synapse
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
                .connect(&src_id, neuron_port, Arc::new(Mutex::new(rx)))
                .await?;
            let mut g_input_interface = self.input_interface.lock().await;
            match g_input_interface.entry(network_port) {
                Entry::Vacant(entry) => {
                    entry.insert(Arc::new(Mutex::new(tx)));
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

    pub async fn pop_results(&self) -> Vec<Vec<u8>> {
        let mut g_results = self.results.lock().await;
        let snapshot = g_results.clone();
        g_results.clear();
        snapshot
    }

    pub async fn setup_output(
        &self,
        network_port: usize,
        neuron_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut g_output_interface = self.output_interface.lock().await;
        if let Some(neuron) = self.get_neuron(neuron_id).await {
            let receiver = neuron.provide_output().await;
            // let c_receiver = receiver.clone();
            let c_receiver = neuron.provide_output().await;
            let c_results = self.results.clone();
            let c_output_interface = self.output_interface.clone();
            let c_port = network_port.clone();
            g_output_interface.insert(network_port.clone(), receiver);
            let jh = tokio::task::spawn(async move {
                let port = Arc::new(c_port);
                let mut g_port_result_receiver = c_receiver.lock().await;
                while let Ok(signal) = g_port_result_receiver.recv().await {
                    let output_width = c_output_interface.lock().await.len();
                    println!("output_width: {}", output_width);
                    println!("Check Point for {} port with signal {}", port, signal);
                    let mut result: Vec<u8> = vec![0; output_width];
                    // let mut result: Vec<u8> = Vec::with_capacity(output_width);
                    // result.fill(0);
                    result[*port] = signal;
                    c_results.lock().await.push(result);
                }
            });
            self.processing_registers
                .lock()
                .await
                .insert(network_port.clone(), jh);

            Ok(())
        } else {
            Err(Box::new(RnnError::IdNotFound))
        }
    }

    pub async fn get_output_receiver(&self, port: usize) -> Option<Arc<Mutex<Receiver<u8>>>> {
        self.output_interface
            .lock()
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
    use std::time::Duration;

    use tokio::time::sleep;

    use crate::rnn::tests::fixtures::{
        gen_neuron_input_config_fixture, new_network_fixture, new_neuron_fixture,
    };

    use super::*;

    #[test]
    fn should_create_two_unique_networks() {
        let n1 = Network::new().unwrap();
        let n2 = Network::new().unwrap();

        assert_ne!(n1.id, n2.id);
    }

    #[tokio::test]
    async fn should_create_two_neurons_in_same_network() {
        let net_orig = Arc::new(Network::new().unwrap());

        for _ in 0..=1 {
            let net = net_orig.clone();
            assert!(net.create_neuron(net.clone(), vec![]).await.is_ok());
        }

        let net = net_orig.clone();
        assert_eq!(net.len().await, 2);
    }

    #[tokio::test]
    async fn network_can_get_neuron_after_create() {
        let net = Arc::new(Network::new().unwrap());

        let neuron_rc = net.create_neuron(net.clone(), vec![]).await.unwrap();
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

        let neuron_rc = net.create_neuron(net.clone(), vec![]).await.unwrap();
        assert_eq!(net.len().await, 1);

        assert!(net.remove_neuron(&neuron_rc.get_id()).await.is_ok());
        assert_eq!(net.len().await, 0);
    }

    #[tokio::test]
    async fn network_should_return_error_if_remove_by_incorrect_id() {
        let net = Arc::new(Network::new().unwrap());

        let neuron = net.create_neuron(net.clone(), vec![]).await.unwrap();
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

        let neuron = net.create_neuron(net.clone(), vec![]).await.unwrap();

        assert!(net.has_neuron(neuron.get_id().as_str()).await);
        assert!(!net.has_neuron("missed").await);
    }

    #[test]
    fn network_returns_correct_id_by_get_id() {
        let net = Network::new().unwrap();
        assert_eq!(net.id, net.get_id());
    }

    #[tokio::test]
    async fn should_connect_one_neuron_to_available_port_of_another_one() {
        let net = Arc::new(new_network_fixture());
        let neuron1 = net.create_neuron(net.clone(), vec![]).await.unwrap();
        let src_id = neuron1.get_id();
        let neuron2 = net.create_neuron(net.clone(), vec![]).await.unwrap();
        let dst_id = neuron2.get_id();

        let res = net.connect_neurons(&src_id, &dst_id, 0).await;
        assert!(res.is_ok());

        let src_state = neuron1.get_state().await;

        assert_eq!(src_state.dendrite_count, 1);
        assert_eq!(src_state.dendrite_connected_count, 0);

        let dst_state = neuron2.get_state().await;
        assert_eq!(dst_state.dendrite_count, 1);
        assert_eq!(dst_state.dendrite_connected_count, 1);
    }

    #[tokio::test]
    async fn should_not_connect_one_neuron_to_busy_port_of_another_one() {
        let net = Arc::new(new_network_fixture());

        let neuron_alt = net.create_neuron(net.clone(), vec![]).await.unwrap();
        let alt_id = neuron_alt.get_id();
        let neuron1 = net.create_neuron(net.clone(), vec![]).await.unwrap();
        let src_id = neuron1.get_id();
        let neuron2 = net.create_neuron(net.clone(), vec![]).await.unwrap();
        let dst_id = neuron2.get_id();

        let res = net.connect_neurons(&src_id, &dst_id, 0).await;
        assert!(res.is_ok());

        let res = net.connect_neurons(&alt_id, &dst_id, 0).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_not_connect_one_neuron_to_missed_one() {
        let net = Arc::new(new_network_fixture());

        let neuron1 = net.create_neuron(net.clone(), vec![]).await.unwrap();
        let src_id = neuron1.get_id();
        let dst_id = "M0Z555";

        let res = net.connect_neurons(&src_id, &dst_id, 0).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_not_allow_to_connect_self_if_only_one_dendrite_exists() {
        let net = Arc::new(new_network_fixture());

        let neuron = net.create_neuron(net.clone(), vec![]).await.unwrap();
        let id = neuron.get_id();

        let res = net.connect_neurons(&id, &id, 0).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn should_self_connect_if_more_then_one_dendrite_exists() {
        let net = Arc::new(new_network_fixture());

        let neuron = net
            .create_neuron(net.clone(), gen_neuron_input_config_fixture(2))
            .await
            .unwrap();
        let id = neuron.get_id();

        let res = net.connect_neurons(&id, &id, 0).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn network_should_return_results_of_signal_sequence_processing() {
        let net = Arc::new(new_network_fixture());

        let config1 = vec![InputCfg::new(2, 2, -1), InputCfg::new(1, 1, 1)];

        let config2 = vec![InputCfg::new(1, 1, -2), InputCfg::new(2, 2, 1)];

        let neuron0 = new_neuron_fixture(net.clone(), vec![]).await;
        let id0 = neuron0.get_id();
        let neuron1 = new_neuron_fixture(net.clone(), config1).await;
        let id1 = neuron1.get_id();
        let neuron2 = new_neuron_fixture(net.clone(), config2).await;
        let id2 = neuron2.get_id();

        // create inter neuron links
        assert!(net.connect_neurons(&id0, &id1, 0).await.is_ok());
        assert!(net.connect_neurons(&id0, &id1, 1).await.is_ok());

        assert!(net.connect_neurons(&id0, &id2, 0).await.is_ok());
        assert!(net.connect_neurons(&id0, &id2, 1).await.is_ok());

        // link neuron's synapse with network's input port
        assert!(net.setup_input(0, &id0, 0).await.is_ok());

        // link neurons' axons with network's output ports
        assert!(net.setup_output(0, &id1).await.is_ok());
        assert!(net.setup_output(1, &id2).await.is_ok());

        // input signal 0 into 0 network's port
        assert!(net.input(0, 0).await.is_ok());
        sleep(Duration::from_millis(1)).await;

        // input signal 1 into 0 network's port
        assert!(net.input(1, 0).await.is_ok());
        sleep(Duration::from_millis(1)).await;

        let state0 = neuron0.get_state().await;
        assert_eq!(state0.hit_count, 2);

        let results = net.pop_results().await;
        assert_eq!(results.len(), 2);
    }
}
