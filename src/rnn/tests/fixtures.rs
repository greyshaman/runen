use std::sync::Arc;

use crate::rnn::{
    common::{input_cfg::InputCfg, signal::Weight},
    layouts::network::Network,
    neural::neuron::Neuron,
};

pub fn new_network_fixture() -> Network {
    Network::new().unwrap()
}

pub async fn new_neuron_fixture(
    network: Arc<Network>,
    bias: Weight,
    input_config: Vec<InputCfg>,
) -> Arc<Neuron> {
    network
        .create_neuron(network.clone(), bias, input_config)
        .await
        .unwrap()
}

/// Generate neuron config with size param.
/// Each dendrite should have configuration with values plus one then before it.
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
