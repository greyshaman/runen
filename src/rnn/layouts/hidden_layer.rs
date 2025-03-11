use std::{
    collections::HashSet,
    sync::{Arc, Weak},
};

use crate::rnn::{common::arithmetic::Arithmetic, neural::neuron::Neuron};

use super::layer::Layer;

/// A hidden layer is a structural component of a neural network that consists of neurons
/// connected to each other and to neurons in other layers.
/// The layers are connected in such a way that there is a specific direction in which signals
/// can flow through the network. This relationship between the layers determines
/// the main flow of information in the neural network.
pub struct HiddenLayer<S>
where
    S: Arithmetic,
{
    /// The neurons set
    neurons: Vec<Arc<Neuron<S>>>,

    /// Layers located closer to the input layer
    predecessors: HashSet<Weak<Layer<S>>>,

    /// Layers located closer to the output layer
    successors: HashSet<Arc<Layer<S>>>,
}
