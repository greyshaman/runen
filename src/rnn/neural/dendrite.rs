use std::rc::Rc;

use crate::rnn::common::component::Component;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::container::Container;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::collector::Collector;


const DEFAULT_WEIGHT: i8 = 1;

/// The Dendrite is model of neurons part
/// It is receive signal from synapse, weighting it and
/// retransmit to neurosoma as aggregator
pub struct Dendrite {
  id: String,
  container: Rc<dyn Container>,
  weight: i8,
}

impl Dendrite {
  pub fn new(id: &str, container: &Rc<dyn Container>, weight: Option<i8>) -> Dendrite {
    let weight = match weight {
      None => DEFAULT_WEIGHT,
      Some(val) => val,
    };

    Dendrite {
      id: String::from(id),
      container: Rc::clone(container),
      weight,
    }
  }
}

impl Collector for Dendrite {
  fn collect(&self, signal: u8) {
    if let Some(
      linked_aggregator
    ) = &self.container.get_aggregator() {
      let new_signal = self.weight as i16 * signal as i16;

      linked_aggregator
        .borrow_mut()
        .notify(&self.id, new_signal);
    }
  }
}

impl Connectable for Dendrite {}

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

impl Component for Dendrite {}
