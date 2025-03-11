use std::fmt::Debug;

use chrono::{DateTime, Utc};

use super::{arithmetic::Arithmetic, signal::Signal};

/// Current neuron state.
#[derive(Debug, Clone)]
pub struct NeuronInfo<S>
where
    S: Arithmetic,
{
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
    pub accumulator: S,

    /// The number of active receivers.
    pub receiver_count: usize,

    /// The sum of dendrites weight
    pub total_weight: S,
}

#[derive(Debug, Clone)]
pub struct PortInfo<S>
where
    S: Arithmetic,
{
    /// The port status at a time.
    pub timestamp: DateTime<Utc>,

    /// The port id
    pub id: String,

    /// The number of total signal through port
    pub hit_count: u64,

    /// Keep last signal value
    pub recent_signal_value: S,
}

#[derive(Debug, Clone)]
pub enum Status<S>
where
    S: Arithmetic,
{
    Neuron(NeuronInfo<S>),

    Port(PortInfo<S>),
}
