use std::{collections::HashSet, rc::Rc};

use crate::rnn::common::component::Component;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::emitter::Emitter;
use crate::rnn::common::container::Container;
use crate::rnn::common::connectable::Connectable;


pub struct Axon {
  id: String,
  container: Rc<dyn Container>,
  acceptors_ids: HashSet<String>,
}

impl Axon {
  pub fn new(id: &str, container_ref: &Rc<dyn Container>) -> Axon {
    Axon {
      id: String::from(id),
      container: Rc::clone(container_ref),
      acceptors_ids: HashSet::new()
    }
  }
}

impl Emitter for Axon {
  fn emit(&self, signal: u8) {
    // FIXME use channels to improve signal sending
    for key in &self.acceptors_ids {
      if let Some(acceptor_ref) = &self.container.get_acceptor(key) {
        acceptor_ref.borrow_mut().accept(signal);
      }
    }
  }
}

impl Connectable for Axon {
  fn connect(&mut self, party_id: &str) {
    &self.acceptors_ids.insert(party_id.to_string());
  }

  fn disconnect(&mut self, party_id: &str) {
    &self.acceptors_ids.remove(party_id);
  }
}

impl Specialized for Axon {
  fn get_spec_type(&self) -> SpecificationType {
    SpecificationType::Emitter
  }
}

impl Component for Axon {}

impl Identity for Axon {
  fn get_id(&self) -> String {
    self.id.clone()
  }
}
