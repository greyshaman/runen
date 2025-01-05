use crate::rnn::common::signal::Signal;

/// The Reconstructible trait defines an API for interpreting information processed by a neural network.
/// The result returned by the neural network should be able to return variants depending on the port
/// number with the highest value(The statement with the highest probability).
/// If the same maximum values are received on different ports,
/// the solution contains noise and does not contain a useful response.
/// This solution does not contain an unambiguously useful answer.
/// The output data type must implement trait Reconstructible,
/// which has the reconstruct() method returns an optional type with
/// the value of solution/output or None if there is no unambiguous answer.
/// The result can also have a direct value equal to the value of the single output port.
/// This should be determined by the implementation of Reconstructible.
pub trait Reconstructible<T = Signal>: Sized {
    fn reconstruct(raw_data: Vec<T>) -> Option<Self>;
    fn dimension(&self) -> usize;
}
