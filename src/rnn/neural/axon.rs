use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Weak;
use std::rc::Rc;

use crate::rnn::common::receiver::Receiver;
use crate::rnn::common::sender::Sender;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::container::Container;
use crate::rnn::common::connectable::Connectable;

/// The Axon is able to emit a signal, which is then received
/// by the connected Synapses.
#[derive(Debug)]
pub struct Axon {
  id: String,
  container: RefCell<Weak<RefCell<dyn Container>>>,
  acceptors: RefCell<HashMap<String, Weak<RefCell<dyn Receiver>>>>,
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

impl Receiver for Axon {
  fn receive(&mut self, signal: i16, _: &str) {
    self.send(signal);
  }
}

impl Sender for Axon {
  fn send(&self, signal: i16) {
    // FIXME use channels to improve signal sending
  for (id, acceptor_weak) in self.acceptors.borrow_mut().iter() {
    acceptor_weak.upgrade()
      .map(|acceptor_rc| {
        acceptor_rc.borrow_mut().receive(signal, self.get_id().as_str());
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

impl Identity for Axon {
  fn get_id(&self) -> String {
    self.id.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct MockAcceptor {
    accepted_signal: u8,
  }

  #[test]
  fn test1() {
    let t = 1;
    assert_eq!(t, 1);
  }

}
