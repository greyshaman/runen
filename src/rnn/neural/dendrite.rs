use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::rnn::common::collector::Collector;
use crate::rnn::common::component::Component;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::container::Container;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::signal_msg::SignalMessage;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;

const DEFAULT_WEIGHT: i16 = 1;

/// The Dendrite is model of neuron's part
/// It is receive signal from synapse, weighting it and
/// retransmit to neurosoma as aggregator
#[derive(Debug)]
pub struct Dendrite {
    id: String,
    container: RefCell<Weak<RefCell<dyn Container>>>,
    weight: i16,
    aggregator: Option<Rc<RefCell<dyn Component>>>,
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

impl Component for Dendrite {
    fn receive(&mut self, signal_msg: Box<SignalMessage>) {
        let SignalMessage(signal, _) = *signal_msg;
        let new_signal = self.weight * signal;

        self.send(new_signal);
    }

    fn send(&self, signal: i16) {
        self.aggregator.as_ref().map(|aggregator_rc| {
            aggregator_rc
                .borrow_mut()
                .receive(Box::new(SignalMessage(signal, Box::new(self.get_id()))));
        });
    }

    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>> {
        self.container.borrow().upgrade()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Connectable for Dendrite {
    fn connect(&mut self, party_id: &str) {
        self.aggregator = self
            .container
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
        SpecificationType::Dendrite
    }
}

impl Identity for Dendrite {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Collector for Dendrite {}
