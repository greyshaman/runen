use std::{any::Any, fmt::Debug};

use super::{identity::Identity, specialized::Specialized};

/// The fundamental aspect of a component.
/// Any structural part of a neuron can be considered a component,
/// including the neuron itself, which is a component of a neural network.
pub trait Component: Identity + Specialized + Any + Debug {
  /// A method for carrying out reflection from a characteristic
  /// to the types that implement it
  fn as_any(&self) -> &dyn Any;

  /// The same as "as_any", but for mutable entities.
  fn as_mut_any(&mut self) -> &mut dyn Any;
}