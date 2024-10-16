use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use super::acceptor::Acceptor;
use super::aggregator::Aggregator;
use super::collector::Collector;
use super::emitter::Emitter;
use super::identity::Identity;
use super::specialized::Specialized;

/// An entity that can contain various components in its composition
pub trait Container: Identity + Specialized {
  fn create_acceptor(
    &mut self,
    max_capacity: Option<u8>,
    regeneration_amount: Option<u8>,
  );
  fn create_collector(&mut self, weight: Option<i8>);
  fn create_aggregator(&mut self);
  fn create_emitter(&mut self);

  fn get_acceptor(&self, id: &str) -> Option<Rc<RefCell<dyn Acceptor>>>;
  fn get_collector(&self, id: &str) -> Option<Rc<RefCell<dyn Collector>>>;
  fn get_aggregator(&self, id: &str) -> Option<Rc<RefCell<dyn Aggregator>>>;
  fn get_emitter(&self, id: &str) -> Option<Rc<RefCell<dyn Emitter>>>;

  fn remove_acceptor(&self, id: &str) -> Result<(), Box<dyn Error>>;
  fn remove_collector(&self, id: &str) -> Result<(), Box<dyn Error>>;
  fn remove_aggregator(&self, id: &str) -> Result<(), Box<dyn Error>>;
  fn remove_emitter(&self, id: &str) -> Result<(), Box<dyn Error>>;
}