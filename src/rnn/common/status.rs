use chrono::{DateTime, Utc};

use super::signal::{Signal, Weight};

/// Current neuron state.
#[derive(Debug, Clone)]
pub struct NeuronInfo {
    /// The status of the neuron at a time.
    pub timestamp: DateTime<Utc>,

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
    pub accumulator: Weight,

    /// The number of active receivers.
    pub receiver_count: usize,

    /// The sum of dendrites weight
    pub total_weight: Weight,
}

#[derive(Debug, Clone)]
pub struct PortInfo {
    /// The port status at a time.
    pub timestamp: DateTime<Utc>,
    pub id: String,
    pub hit_count: u64,
    pub recent_signal: Signal,
}

#[derive(Debug, Clone)]
pub enum Status {
    Neuron(NeuronInfo),

    Port(PortInfo),
}
