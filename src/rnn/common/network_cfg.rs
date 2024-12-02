use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use super::input_cfg::InputCfg;

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkCfg {
    Input {
        input_port: usize,
        dst_id: String,
        dst_synapse_idx: usize,
    },
    Inner {
        src_id: String,
        dst_id: String,
        dst_synapse_idx: usize,
    },
    Output {
        src_id: String,
        output_port: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NeuronConfig {
    pub id: String,
    pub input_configs: Vec<InputCfg>,
}

/// The network config structure used to describe neuron set and connections between them.
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    neurons: BTreeMap<String, NeuronConfig>,
    links: HashSet<LinkCfg>,
}
