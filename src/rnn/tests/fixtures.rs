use std::sync::Arc;

use crate::rnn::{
    layouts::network::Network,
    neural::{dendrite::InputCfg, neuron::Neuron},
};

pub fn new_network_fixture() -> Network {
    Network::new().unwrap()
}

pub async fn new_neuron_fixture(network: Arc<Network>, input_config: Vec<InputCfg>) -> Arc<Neuron> {
    network
        .create_neuron(network.clone(), input_config)
        .await
        .unwrap()
}

pub fn gen_neuron_input_config_fixture(size: u8) -> Vec<InputCfg> {
    (1..=size)
        .into_iter()
        .map(|i| InputCfg {
            capacity_max: i,
            regeneration: i,
            weight: i as i16,
        })
        .collect()
}
