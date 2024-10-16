use std::{cell::RefCell, error::Error, rc::Rc};

use super::{container::Container, identity::Identity, specialized::Specialized};

/// Media is a system that consists of containers with various functional elements.
/// It manages the creation, update, and deletion of both containers and components.

pub trait Media: Identity + Specialized {
  /// Gets container by id
  fn get_container(&self, id: &str) -> Option<&Rc<RefCell<dyn Container>>>;

  /// Inserts the container
  fn insert_container(&mut self, element: &Rc<RefCell<dyn Container>>)
    -> Result<String, Box<dyn Error>>;

  /// Remove container with dependencies
  fn remove_container(&mut self, id: &str) -> Result<(), Box<dyn Error>>;

  /// Verify if has container by id
  fn has_container(&self, id: &str) -> bool;
}