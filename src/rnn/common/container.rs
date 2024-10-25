use std::any::Any;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use super::grouped::Grouped;
use super::identity::Identity;
use super::receiver::Receiver;
use super::spec_type::SpecificationType;
use super::specialized::Specialized;

/// A substance that can contain different elements in its composition.
pub trait Container: Identity + Specialized + Grouped + Any + Debug {
  /// Create the acceptor component and save it in internal memory.
  fn create_acceptor(
    &mut self,
    max_capacity: Option<i16>,
    regeneration_amount: Option<i16>,
  );

  /// Create the collector component and save it in internal memory.
  fn create_collector(&mut self, weight: Option<i16>);

  /// Create the aggregator component and save it in internal memory.
  fn create_aggregator(&mut self);

  /// Create the emitter component and save it in internal memory.
  fn create_emitter(&mut self);

  /// Returns the component based on its ID.
  fn get_component(&self, id: &str) -> Option<&Rc<RefCell<dyn Receiver>>>;

  /// Deleting a component by its ID
  fn remove_component(&mut self, id: &str) -> Result<(), Box<dyn Error>>;

  /// Returns how many container contains components
  fn len(&self) -> usize;

  /// Returns how many components with specified spec_type have container
  fn len_by_spec_type(&self, spec_type: &SpecificationType) -> usize;

  /// Reflection method
  fn as_any(&self) -> &dyn Any;

  /// Reflection method
  fn as_mut_any(&mut self) -> &mut dyn Any;
}