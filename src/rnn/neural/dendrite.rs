use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::rnn::common::aggregator::Aggregator;
use crate::rnn::common::component::Component;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::container::Container;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::collector::Collector;

use super::neurosoma::Neurosoma;

const DEFAULT_WEIGHT: i8 = 1;

/// The Dendrite is model of neurons part
/// It is receive signal from synapse, weighting it and
/// retransmit to neurosoma as aggregator
#[derive(Debug)]
pub struct Dendrite {
  id: String,
  container: RefCell<Weak<RefCell<dyn Container>>>,
  weight: i8,
  aggregator: Option<Rc<RefCell<dyn Component>>>,
}

impl Dendrite {
  pub fn new(id: &str, container: &Rc<RefCell<dyn Container>>, weight: Option<i8>) -> Dendrite {
    let weight = weight.unwrap_or(DEFAULT_WEIGHT);

    Dendrite {
      id: String::from(id),
      container: RefCell::new(Rc::downgrade(&container)),
      weight,
      aggregator: None,
    }
  }
}

impl Collector for Dendrite {
  fn collect(&self, signal: u8) {
    self.aggregator.as_ref().map(|aggregator_rc| {
      let new_signal = self.weight as i16 * signal as i16;
      aggregator_rc.borrow_mut()
        .as_mut_any()
        .downcast_mut::<Neurosoma>()
        .unwrap()
        .notify(self.get_id().as_str(), new_signal);
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

impl Component for Dendrite {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
