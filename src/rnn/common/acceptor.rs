use super::{component::Component, connectable::Connectable};

/// An entity that can receive a signal emitted from an emitter.
pub trait Acceptor: Component + Connectable {
  /// Receives the signal and processes it for further transmission.
  fn accept(&mut self, signal: u8);
}