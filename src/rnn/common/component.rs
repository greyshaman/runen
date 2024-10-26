use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::{any::Any, fmt::Debug};

use super::connectable::Connectable;
use super::container::Container;
use super::identity::Identity;
use super::signal_msg::SignalMessage;
use super::specialized::Specialized;

/// The fundamental aspect of a component.
/// Any structural part of a neuron can be considered a component,
/// including the neuron itself, which is a component of a neural network.
pub trait Component: Connectable + Identity + Specialized + Any + Debug {
    /// Receives a signal
    fn receive(&mut self, signal_msg: Box<SignalMessage>);

    /// Sends a signal.
    fn send(&self, signal: i16);

    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>>;

    /// A method for carrying out reflection from a characteristic
    /// to the types that implement it
    fn as_any(&self) -> &dyn Any;

    /// The same as "as_any", but for mutable entities.
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
