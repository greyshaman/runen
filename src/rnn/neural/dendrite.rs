use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::rnn::common::receiver::Receiver;
use crate::rnn::common::sender::Sender;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::container::Container;
use crate::rnn::common::connectable::Connectable;

const DEFAULT_WEIGHT: i16 = 1;

/// The Dendrite is model of neuron's part
/// It is receive signal from synapse, weighting it and
/// retransmit to neurosoma as aggregator
#[derive(Debug)]
pub struct Dendrite {
  id: String,
  container: RefCell<Weak<RefCell<dyn Container>>>,
  weight: i16,
  aggregator: Option<Rc<RefCell<dyn Receiver>>>,
}

impl Dendrite {
  pub fn new(id: &str, container: &Rc<RefCell<dyn Container>>, weight: Option<i16>) -> Dendrite {
    let weight = weight.unwrap_or(DEFAULT_WEIGHT);

    Dendrite {
      id: String::from(id),
      container: RefCell::new(Rc::downgrade(&container)),
      weight,
      aggregator: None,
    }
  }
}

impl Receiver for Dendrite {
  fn receive(&mut self, signal: i16, _: &str) {
    let new_signal = self.weight * signal;

    self.send(new_signal);
  }
}

impl Sender for Dendrite {
  fn send(&self, signal: i16) {
    self.aggregator.as_ref().map(|aggregator_rc|{
      aggregator_rc.borrow_mut().receive(signal, self.get_id().as_str());
    });
  }
}

impl Connectable for Dendrite {
    fn connect(&mut self, party_id: &str) {
      self.aggregator = self.container
        .borrow()
        .upgrade()
        .unwrap() // Neuron should anyway
        .borrow()
        .get_component(party_id)
        .map(|aggregator_rc| Rc::clone(&aggregator_rc));
    }

    fn disconnect(&mut self, _party_id: &str) {
      self.aggregator = None;
    }
}

impl Specialized for Dendrite {
  fn get_spec_type(&self) -> SpecificationType {
    SpecificationType::Collector
  }
}

impl Identity for Dendrite {
  fn get_id(&self) -> String {
    self.id.clone()
  }
}
