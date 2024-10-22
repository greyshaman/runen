use std::{any::Any, cell::RefCell, error::Error, rc::Rc};

use super::{container::Container, group_type::GroupType, identity::Identity, specialized::Specialized};

/// Media is a system that consists of various functional elements in containers.
/// It manages the creation, updating, and deletion of these containers.
pub trait Media: Identity + Specialized + Any {
  /// Gets container by id
  fn get_container(&self, id: &str) -> Option<&Rc<RefCell<dyn Container>>>;

  /// Create and insert container
  fn create_container(&mut self, group_type: &GroupType, media: &Rc<RefCell<dyn Media>>) -> Result<String, Box<dyn Error>>;

  /// Remove container with dependencies
  fn remove_container(&mut self, id: &str) -> Result<(), Box<dyn Error>>;

  /// Verify if has container by id
  fn has_container(&self, id: &str) -> bool;

  fn as_any(&self) -> &dyn Any;

  fn as_mut_any(&mut self) -> &mut dyn Any;
}