use std::sync::Arc;

use tokio::sync::{broadcast::Receiver, RwLock};

use crate::rnn::common::{input_cfg::InputCfg, signal::Signal};

/// Neuron's input (the dendrite)
#[derive(Debug)]
pub struct Dendrite {
    /// Input configuration
    pub config: InputCfg,

    /// Synapse capacity
    pub synapse_capacity: Signal,

    /// Keep party id or none
    pub connected: Option<String>,

    /// Receiver part of channel between axon and synapse
    pub synapse: Option<Arc<RwLock<Receiver<Signal>>>>,
}
