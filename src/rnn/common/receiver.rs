use std::{any::Any, fmt::Debug};

use super::identity::Identity;
use super::signal_msg::SignalMessage;
use super::specialized::Specialized;

/// The Component is able receive a signal.
pub trait Receiver: Identity + Specialized + Any + Debug {
    /// Receives a signal
    fn receive(&mut self, signal_msg: Box<SignalMessage>);
}
