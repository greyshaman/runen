use std::sync::Arc;

use tokio::sync::{broadcast::Receiver, RwLock};

/// Input configuration
#[derive(Debug)]
pub struct InputCfg {
    /// High limit of synapse (input) capacity
    pub capacity_max: u8,

    /// The amount of capacity recovery after its reduction
    pub regeneration: u8,

    /// Th dendrite's weight
    pub weight: i16,
}

/// Neuron's input (the dendrite)
#[derive(Debug)]
pub struct Dendrite {
    /// Input configuration
    pub config: InputCfg,

    /// Synapse capacity
    pub syn_capacity: u8,

    /// Keep party id or none
    pub connected: Option<String>,

    /// Receiver part of channel between axon and synapse
    pub input: Option<Arc<RwLock<Receiver<u8>>>>,
}

impl InputCfg {
    pub fn new(capacity_max: u8, regeneration: u8, weight: i16) -> Self {
        InputCfg {
            capacity_max,
            regeneration,
            weight,
        }
    }
}
