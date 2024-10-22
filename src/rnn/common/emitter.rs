use super::{component::Component, connectable::Connectable};

/// The Emitter is able to emit a signal, which is then received
/// by the connected Acceptors.
pub trait Emitter: Component + Connectable {
  /// Sending a signal to all connected devices.
  fn emit(&self, signal: u8);
}