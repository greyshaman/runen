use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;

use crate::rnn::common::acceptor::Acceptor;
use crate::rnn::common::component::Component;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::container::Container;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::spec_type::SpecificationType;

/// Ёмкость медиаторного ресурса синапса
const DEFAULT_CAPACITY: u8 = 1;

/// The Synapse is model of connection between Axon and Dendrite
/// It is accept incoming stimulation and produce signal for dendrite
/// Th Value of produced signal depended from stimulation value, capacity and weight
pub struct Synapse {
  id: String, // Pattern N000A00
  container: Rc<RefCell<dyn Container>>,
  max_capacity: u8,
  current_capacity: u8,
  regeneration_amount: u8,
  collector_id: String,
}

impl Synapse {
  pub fn new(
    id: &str,
    container: &Rc<RefCell<dyn Container>>,
    capacity_opt: Option<u8>,
    regeneration: Option<u8>
  ) -> Synapse {

    let capacity = match capacity_opt {
      None => DEFAULT_CAPACITY,
      Some(val) => val,
    };

    let regeneration_amount = match regeneration {
      None => capacity,
      Some(val) => val % (capacity + 1),
    };

    Synapse {
      id: String::from(id),
      container: Rc::clone(container),
      max_capacity: capacity,
      current_capacity: capacity,
      collector_id: String::from(""),
      regeneration_amount,
    }
  }
}

impl Acceptor for Synapse {
  fn accept(&mut self, signal: u8) {
    let new_signal = min(signal, self.current_capacity);
    self.current_capacity -= signal;

    self.current_capacity += min(self.current_capacity + self.regeneration_amount, self.max_capacity);

    if let Some(linked_collector) =
      &self.container.borrow().get_collector(&self.collector_id) {
        linked_collector.borrow().collect(new_signal);
      }
  }
}

impl Connectable for Synapse {
  /// Connect to collector
  fn connect(&mut self, party_id: &str) {
    self.collector_id = party_id.to_string();
  }

  /// Disconnect from collector
  fn disconnect(&mut self, _party_id: &str) {
      self.collector_id = "".to_string();
  }
}

impl Specialized for Synapse {
  fn get_spec_type(&self) -> SpecificationType {
    SpecificationType::Acceptor
  }
}

impl Component for Synapse {}

impl Identity for Synapse {
  fn get_id(&self) -> String {
    self.id.clone()
  }
}
