use crate::rnn::common::signal::Signal;

/// The Splittable trait determines how the input data will be transformed
/// over the input ports of the neural network.
pub trait Splittable<T = Signal> {
    fn dimension(&self) -> usize;
    fn split(&self) -> Vec<T>;
}
