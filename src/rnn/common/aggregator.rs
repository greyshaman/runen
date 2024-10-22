use super::{component::Component, connectable::Connectable};

/// An entity that is able to establish a connection with
/// an emitter and convert multiple received signals into
/// a single output signal, which it then sends to the emitter.
pub trait Aggregator: Component + Connectable {
  /// The incoming signal can be both positive (exciting)
  /// and negative (braking). It is processed and the results
  /// are accumulated.
  fn notify(&mut self, collector_id: &str, signal: i16);

  /// The signal is sent to the emitter.
  fn kick(&self, signal: u8);
}