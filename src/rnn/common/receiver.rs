use std::{any::Any, fmt::Debug};

use super::{identity::Identity, specialized::Specialized};

/// The Component is able receive a signal.
pub trait Receiver: Identity + Specialized + Any + Debug {
  /// Receives a signal
  fn receive(&mut self, signal: i16, source_id: &str);
}