use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Weak;
use std::rc::Rc;

use crate::rnn::common::emitter::Emitter;
use crate::rnn::common::receiver::Receiver;
use crate::rnn::common::sender::Sender;
use crate::rnn::common::signal_msg::SignalMessage;
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
  fn receive(&mut self, signal_msg: Box<SignalMessage>) {
    let SignalMessage(signal, _) = *signal_msg;
    self.send(signal);
  }
}

impl Sender for Axon {
  fn send(&self, signal: i16) {
    // FIXME use channels to improve signal sending
  for (id, acceptor_weak) in self.acceptors.borrow_mut().iter() {
    acceptor_weak.upgrade()
      .map(|acceptor_rc| {
        acceptor_rc.borrow_mut().receive(
          Box::new(SignalMessage(signal, Box::new(self.get_id())))
        );
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

impl Emitter for Axon {}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

use super::*;

  struct MockNeuron {
    id: String,
    components: BTreeMap<String, Rc<RefCell<dyn Receiver>>>,
    my_ref_opt: Option<Rc<RefCell<dyn Container>>>,
  }

  impl MockNeuron {
    pub fn set_my_ref(&mut self, my_ref: &Rc<RefCell<dyn Container>>) {
      self.my_ref_opt = Some(Rc::clone(my_ref));
    }
  }

  // impl Container for MockNeuron {
  //   fn create_acceptor(
  //       &mut self,
  //       max_capacity: Option<i16>,
  //       regeneration_amount: Option<i16>,
  //     ) {
  //       todo!()
  //   }

  //   fn create_collector(&mut self, weight: Option<i16>) {
  //       todo!()
  //   }

  //   fn create_aggregator(&mut self) {
  //       todo!()
  //   }

  //   fn create_emitter(&mut self) {
  //     let axon_id = "Z0E0".to_string();
  //     self.components.insert(
  //       axon_id.clone(),
  //       Rc::new(RefCell::new(Axon::new(&axon_id, &self.my_ref_opt.unwrap()))));
  //   }

  //   fn get_component(&self, id: &str) -> Option<&Rc<RefCell<dyn Receiver>>> {
  //       todo!()
  //   }

  //   fn remove_component(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
  //       todo!()
  //   }

  //   fn as_any(&self) -> &dyn std::any::Any {
  //       todo!()
  //   }

  //   fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
  //       todo!()
  //   }
  // }

  struct MockAcceptor {
    accepted_signal: i16,
  }

  // #[test]
  // fn test_can_receive_positive_signal() {
  //   let mut neuron = Rc::new(
  //     RefCell::new(
  //       MockNeuron { id: String::from("Z0"), components: BTreeMap::new(), my_ref_opt: None }
  //     )
  //   );
  //   // neuron.borrow_mut().set_my_ref(&Rc::clone(&neuron));
  //   let t = 1;
  //   assert_eq!(t, 1);
  // }

}
