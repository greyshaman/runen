use std::cell::RefCell;
use std::cmp::min;
use std::rc::{Rc, Weak};

use crate::rnn::common::acceptor::Acceptor;
use crate::rnn::common::collector::Collector;
use crate::rnn::common::component::Component;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::container::Container;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::spec_type::SpecificationType;

use super::dendrite::Dendrite;

/// The capacity of the synaptic mediator resource

const DEFAULT_CAPACITY: u8 = 1;

/// The Synapse is model of connection between Axon and Dendrite
/// It is accept incoming stimulation and produce signal for dendrite
/// Th Value of produced signal depended from stimulation value, capacity and weight
#[derive(Debug)]
pub struct Synapse {
  id: String, // Pattern N000A00
  container: RefCell<Weak<RefCell<dyn Container>>>,
  max_capacity: u8,
  current_capacity: u8,
  regeneration_amount: u8,
  collector: Option<Rc<RefCell<dyn Component>>>,
}

impl Synapse {
  pub fn new(
    id: &str,
    container: &Rc<RefCell<dyn Container>>,
    capacity_opt: Option<u8>,
    regeneration: Option<u8>
  ) -> Synapse {
    let max_capacity = capacity_opt.unwrap_or(DEFAULT_CAPACITY);
    let regeneration_amount = regeneration.unwrap_or(DEFAULT_CAPACITY);

    Synapse {
      id: String::from(id),
      container: RefCell::new(Rc::downgrade(container)),
      max_capacity,
      current_capacity: max_capacity,
      collector: None,
      regeneration_amount,
    }
  }
}

impl Acceptor for Synapse {
  fn accept(&mut self, signal: u8) {
    let new_signal = min(signal, self.current_capacity);

    self.current_capacity -= new_signal;
    // regeneration mediator capacity
    self.current_capacity += min(self.current_capacity + self.regeneration_amount, self.max_capacity);

    self.collector.as_ref().map(|d| {
      d.borrow()
        .as_any()
        .downcast_ref::<Dendrite>()
        .unwrap()
        .collect(new_signal)
    });
  }
}

impl Connectable for Synapse {
  /// Connect to collector
  fn connect(&mut self, party_id: &str) {
    self.collector = self.container
      .borrow()
      .upgrade()
      .unwrap()
      .borrow()
      .get_component(party_id)
      .map(|collector_rc| Rc::clone(&collector_rc));
  }

  /// Disconnect from collector
  fn disconnect(&mut self, _party_id: &str) {
      self.collector = None;
  }
}

impl Specialized for Synapse {
  fn get_spec_type(&self) -> SpecificationType {
    SpecificationType::Acceptor
  }
}

impl Component for Synapse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Identity for Synapse {
  fn get_id(&self) -> String {
    self.id.clone()
  }
}
