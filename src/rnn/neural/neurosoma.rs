use std::cell::RefCell;
use std::cmp::max;
use std::rc::Weak;
use std::{collections::HashSet, rc::Rc};

use crate::rnn::common::aggregator::Aggregator;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::receiver::Receiver;
use crate::rnn::common::sender::Sender;
use crate::rnn::common::signal_msg::SignalMessage;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::container::Container;
use crate::rnn::common::connectable::Connectable;

/// The neurosoma collects the signals received from
/// the dendrites and sends the resulting signal down
/// the axon when it receives repeated signals from one
/// of the dendrites.
#[derive(Debug)]
pub struct Neurosoma {
  id: String,
  container: RefCell<Weak<RefCell<dyn Container>>>,

  /// The IDs of the collectors who sent the processed signals
  /// to the aggregator are used to trigger signal aggregation
  /// when a repeated signal is received from any
  /// of these collectors.

  reported_collectors: HashSet<String>,
  emitter: Option<Rc<RefCell<dyn Receiver>>>,
  accumulator: i16,
}

impl Neurosoma {
  pub fn new(id: &str, container: &Rc<RefCell<dyn Container>>) -> Neurosoma {

    Neurosoma {
      id: String::from(id) ,
      container: RefCell::new(Rc::downgrade(container)),
      reported_collectors: HashSet::new(),
      emitter: None,
      accumulator: 1_i16,
    }
  }

  fn count_referrals(&self) -> usize {
    self.container
      .borrow()
      .upgrade()
      .and_then(|container_rc| {
        container_rc
          .borrow()
          .get_component(self.get_id().as_str())
          .map(|collector_rc| Rc::strong_count(&collector_rc))
      })
      .unwrap_or(0_usize)
  }
}

impl Receiver for Neurosoma {
    fn receive(&mut self, signal_msg: Box<SignalMessage>) {
      let SignalMessage(signal, boxed_source_id) = *signal_msg;
      let collector_id = *boxed_source_id;
      if self.reported_collectors.contains(&collector_id)
        || self.reported_collectors.len() >= self.count_referrals() - 1 {
        let new_signal = max(self.accumulator, 0);

        self.reported_collectors.clear();

        self.accumulator = signal + 1_i16;
        self.reported_collectors.insert(collector_id);

        self.send(new_signal);
      } else {
        self.accumulator += signal;
        self.reported_collectors.insert(collector_id);
      }
    }
}

impl Sender for Neurosoma {
  fn send(&self, signal: i16) {
    self.emitter
      .as_ref()
      .map(|emitter_rc| {
        emitter_rc.borrow_mut().receive(
          Box::new(SignalMessage(signal, Box::new(self.get_id())))
        );
      });
  }
}

impl Connectable for Neurosoma {
    fn connect(&mut self, party_id: &str) {
      self.emitter = self.container
        .borrow()
        .upgrade()
        .unwrap()
        .borrow()
        .get_component(party_id)
        .map(|emitter_rc| Rc::clone(&emitter_rc));
    }

    fn disconnect(&mut self, _party_id: &str) {
      self.emitter = None;
    }
}

impl Specialized for Neurosoma {
  fn get_spec_type(&self) -> SpecificationType {
    SpecificationType::Aggregator
  }
}

impl Identity for Neurosoma {
  fn get_id(&self) -> String {
    self.id.clone()
  }
}

impl Aggregator for Neurosoma {}