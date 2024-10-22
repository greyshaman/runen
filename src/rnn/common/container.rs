use std::any::Any;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use super::component::Component;
use super::grouped::Grouped;
use super::identity::Identity;
use super::specialized::Specialized;

/// A substance that can contain different elements in its composition.
pub trait Container: Identity + Specialized + Grouped + Any + Debug {
  /// Create the acceptor component and save it in internal memory.
  fn create_acceptor(
    &mut self,
    max_capacity: Option<u8>,
    regeneration_amount: Option<u8>,
  );

  /// Create the collector component and save it in internal memory.
  fn create_collector(&mut self, weight: Option<i8>);

  /// Create the aggregator component and save it in internal memory.
  fn create_aggregator(&mut self);

  /// Create the emitter component and save it in internal memory.
  fn create_emitter(&mut self);

  /// Returns the component based on its ID.
  fn get_component(&self, id: &str) -> Option<&Rc<RefCell<dyn Component>>>;

  /// Deleting a component by its ID
  fn remove_component(&mut self, id: &str) -> Result<(), Box<dyn Error>>;

  /// Reflection method
  fn as_any(&self) -> &dyn Any;

  /// Reflection method
  fn as_mut_any(&mut self) -> &mut dyn Any;
}