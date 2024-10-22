use super::{component::Component, connectable::Connectable};

/// An entity that can collect and weigh the received signal,
/// and transmit the processed signal to the aggregator
/// for further processing.
pub trait Collector: Component + Connectable {
  /// The method receives a signal.
  fn collect(&self, signal: u8);
}