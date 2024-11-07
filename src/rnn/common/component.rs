use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use as_any::AsAny;

use super::connectable::Connectable;
use super::container::Container;
use super::identity::Identity;
use super::signal_msg::SignalMessage;
use super::specialized::Specialized;

/// The fundamental aspect of a component.
/// Any structural part of a neuron can be considered a component,
/// including the neuron itself, which is a component of a neural network.
pub trait Component: Connectable + Identity + Specialized + AsAny + Debug {
    /// Receives a signal
    fn receive(&self, signal_msg: Box<SignalMessage>);

    /// Sends a signal.
    fn send(&self, signal: i16);

    /// Return parent container which owned this component.
    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>>;
}
