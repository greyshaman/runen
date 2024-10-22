use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Weak;
use std::rc::Rc;

use crate::rnn::common::acceptor::Acceptor;
use crate::rnn::common::component::Component;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::emitter::Emitter;
use crate::rnn::common::container::Container;
use crate::rnn::common::connectable::Connectable;

use super::synapse::Synapse;


#[derive(Debug)]
pub struct Axon {
  id: String,
  container: RefCell<Weak<RefCell<dyn Container>>>,
  acceptors: RefCell<HashMap<String, Weak<RefCell<dyn Component>>>>,
}

impl Axon {
  pub fn new(id: &str, container_ref: &Rc<RefCell<dyn Container>>) -> Axon {
    Axon {
      id: String::from(id),
      container: RefCell::new(Rc::downgrade(&container_ref)),
      acceptors: RefCell::new(HashMap::new()),
    }
  }
}

impl Emitter for Axon {
  fn emit(&self, signal: u8) {
    // FIXME use channels to improve signal sending
    for (id, acceptor_weak) in self.acceptors.borrow_mut().iter() {
      acceptor_weak.upgrade()
        .map(|acceptor_rc| {
          acceptor_rc.borrow_mut()
          .as_mut_any()
          .downcast_mut::<Synapse>()
          .unwrap()
          .accept(signal)
        })
        .or_else(|| {
          self.acceptors.borrow_mut().remove(id);
          Some(())
        });
    }
  }
}

impl Connectable for Axon {
  fn connect(&mut self, party_id: &str) {
    self.container
      .borrow()
      .upgrade()
      .unwrap()
      .borrow()
      .get_component(party_id)
      .map(|acceptor_rc| {
        self.acceptors.borrow_mut().entry(party_id.to_string())
          .and_modify(|acceptor_weak|
            *acceptor_weak = Rc::downgrade(acceptor_rc)
          )
          .or_insert_with(|| Rc::downgrade(acceptor_rc));
      })
      .or_else(|| {
        self.acceptors
          .borrow_mut()
          .remove(party_id);
        Some(()) // FIXME check this method twice!!! Or write tests
      });
  }

  fn disconnect(&mut self, party_id: &str) {
    self.acceptors.borrow_mut().remove(party_id);
  }
}

impl Specialized for Axon {
  fn get_spec_type(&self) -> SpecificationType {
    SpecificationType::Emitter
  }
}

impl Component for Axon {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Identity for Axon {
  fn get_id(&self) -> String {
    self.id.clone()
  }
}
