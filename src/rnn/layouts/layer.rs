use crate::rnn::common::arithmetic::Arithmetic;

use super::{hidden_layer::HiddenLayer, input_layer::InputLayer, output_layer::OutputLayer};

/// An enumeration describing the various layers that make up
/// a neural network and ensure the organization of the signal processing flow.
pub enum Layer<S>
where
    S: Arithmetic,
{
    Input(InputLayer<S>),
    Hidden(HiddenLayer<S>),
    Output(OutputLayer<S>),
}
