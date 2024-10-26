use std::cell::RefCell;
use std::cmp::min;
use std::rc::{Rc, Weak};

use crate::rnn::common::acceptor::Acceptor;
use crate::rnn::common::component::Component;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::container::Container;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::signal_msg::SignalMessage;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;

/// The capacity of the synaptic mediator resource
const DEFAULT_CAPACITY: i16 = 1;

/// The Synapse is model of connection between Axon and Dendrite
/// It is accept incoming stimulation and produce signal for dendrite
/// Th Value of produced signal depended from stimulation value, capacity and weight
#[derive(Debug)]
pub struct Synapse {
    id: String, // Pattern N000A00
    container: RefCell<Weak<RefCell<dyn Container>>>,
    max_capacity: i16,
    current_capacity: i16,
    regeneration_amount: i16,
    collector: Option<Rc<RefCell<dyn Component>>>,
}

impl Synapse {
    pub fn new(
        id: &str,
        container: &Rc<RefCell<dyn Container>>,
        capacity_opt: Option<i16>,
        regeneration: Option<i16>,
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

impl Component for Synapse {
    fn receive(&mut self, signal_msg: Box<SignalMessage>) {
        let SignalMessage(signal, _) = *signal_msg;
        let new_signal = min(signal, self.current_capacity);

        self.current_capacity -= new_signal;
        // regeneration mediator capacity
        self.current_capacity += min(
            self.current_capacity + self.regeneration_amount,
            self.max_capacity,
        );

        self.send(new_signal);
    }

    fn send(&self, signal: i16) {
        self.collector.as_ref().map(|d| {
            d.borrow_mut()
                .receive(Box::new(SignalMessage(signal, Box::new(self.get_id()))))
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

impl Connectable for Synapse {
    /// Connect to collector
    fn connect(&mut self, party_id: &str) {
        self.collector = self
            .container
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
        SpecificationType::Synapse
    }
}

impl Identity for Synapse {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Acceptor for Synapse {}
