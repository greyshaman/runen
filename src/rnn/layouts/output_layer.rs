use std::{
    collections::HashSet,
    sync::{Arc, Weak},
};

use crate::rnn::common::arithmetic::Arithmetic;

use super::{layer::Layer, output_port::OutputPort};

/// The output layer is the final component of a neural network,
/// located at the end of the signal path. It receives information
/// from neurons in the previous layers and processes it before sending
/// it to the next stage or to external users.
/// The output layer consists of output nodes, which are connected to the neurons
/// in the preceding layers. These nodes receive input signals
/// and process them according to a set of rules or algorithms.
/// The processed information can then be sent to other parts of the network
/// or to external devices or users.
pub struct OutputLayer<S>
where
    S: Arithmetic,
{
    /// The list of output ports
    outputs: Vec<Arc<OutputPort<S>>>,

    /// The set of previous layers
    predecessors: HashSet<Weak<Layer<S>>>,
}
