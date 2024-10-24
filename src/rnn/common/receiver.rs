use std::{any::Any, fmt::Debug};

use super::specialized::Specialized;
use super::signal_msg::SignalMessage;
use super::identity::Identity;

/// The Component is able receive a signal.
pub trait Receiver: Identity + Specialized + Any + Debug {
  /// Receives a signal
  fn receive(&mut self, signal_msg: Box<SignalMessage>);
}