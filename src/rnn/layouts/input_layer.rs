use std::{collections::HashSet, sync::Arc};

use crate::rnn::common::arithmetic::Arithmetic;

use super::{input_port::InputPort, layer::Layer};

/// The input layer is the first layer of a neural network,
/// and it is responsible for receiving information from the outside world.
/// It consists of a set of input nodes, which are connected
/// to the hidden layers of the neural network through a network of connections.
/// These connections allow the input layer to pass information to the next layer,
/// and so on, until the output layer produces the final result.
pub struct InputLayer<S>
where
    S: Arithmetic,
{
    /// The list of input ports
    inputs: Vec<Arc<InputPort<S>>>,

    /// The set of underlying layers
    successors: HashSet<Arc<Layer<S>>>,
}
