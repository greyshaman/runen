use std::any::Any;
use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::{Rc, Weak};

use as_any::AsAny;
use as_any_derive::AsAny;

use crate::rnn::common::acceptor::Acceptor;
use crate::rnn::common::component::Component;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::container::Container;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::signal_msg::SignalMessage;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;

/// The Synapse is model of connection between Axon and Dendrite
/// It is accept incoming stimulation and produce signal for dendrite
/// Th Value of produced signal depended from stimulation value, capacity and weight
#[derive(Debug, AsAny)]
pub struct Synapse {
    id: String, // Pattern N000A00
    container: Weak<RefCell<dyn Container>>,
    max_capacity: i16,
    current_capacity: RefCell<i16>,
    regeneration_amount: i16,
    collector: RefCell<Option<Rc<RefCell<dyn Component>>>>,
}

impl Synapse {
    pub fn new(
        id: &str,
        container: &Rc<RefCell<dyn Container>>,
        capacity: i16,
        regeneration: i16,
    ) -> Synapse {
        Synapse {
            id: String::from(id),
            container: Rc::downgrade(container),
            max_capacity: capacity,
            current_capacity: RefCell::new(capacity),
            collector: RefCell::new(None),
            regeneration_amount: regeneration,
        }
    }
}

impl Component for Synapse {
    fn receive(&self, signal_msg: Box<SignalMessage>) {
        let SignalMessage(signal, _) = *signal_msg;
        let signal = max(signal, 0);
        let mut current_capacity_mut = self.current_capacity.borrow_mut();
        let new_signal = min(signal, *current_capacity_mut);

        *current_capacity_mut -= new_signal;
        // regeneration mediator capacity
        *current_capacity_mut += min(
            *current_capacity_mut + self.regeneration_amount,
            self.max_capacity,
        );

        self.send(new_signal);
    }

    fn send(&self, signal: i16) {
        let collector = self.collector.borrow();
        let collector = collector.as_ref();
        collector.map(|collector| {
            collector
                .borrow()
                .receive(Box::new(SignalMessage(signal, Box::new(self.get_id()))))
        });
    }

    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>> {
        self.container.upgrade()
    }
}

impl Connectable for Synapse {
    /// Connect to collector
    fn connect(&self, party_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if party_id == &self.get_id() {
            return Err(Box::new(RnnError::ClosedLoop));
        }

        *self.collector.borrow_mut() = self
            .container
            .upgrade()
            .unwrap()
            .borrow()
            .get_component(party_id)
            .map(|collector_rc| Rc::clone(&collector_rc));

        Ok(())
    }

    /// Disconnect from collector
    fn disconnect(&self, _party_id: &str) {
        *self.collector.borrow_mut() = None;
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

macro_rules! create_synapse {
    ($id:expr, $container:expr) => {
        Synapse::new($id, $container, 1, 1)
    };
    ($id:expr, $container:expr, $capacity:expr, $regeneration:expr) => {
        Synapse::new($id, $container, $capacity, $regeneration)
    };
}

#[cfg(test)]
mod tests {
    use crate::rnn::cyber::indicator::Indicator;
    use crate::rnn::tests::fixtures::new_indicator_fixture;
    use crate::rnn::tests::fixtures::new_network_fixture;
    use crate::rnn::tests::fixtures::new_neuron_fixture;
    use crate::rnn::tests::fixtures::new_synapse_fixture;

    use super::*;

    #[test]
    fn should_limit_input_signal_by_max_capacity() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let synapse = new_synapse_fixture(&neuron, None, None);
        let synapse = synapse.borrow();

        let indicator = new_indicator_fixture(&neuron);
        let indicator = indicator.borrow();

        synapse.connect(indicator.get_id().as_str()).unwrap();

        let raw_indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();

        synapse.receive(Box::new(SignalMessage(3, Box::new(synapse.get_id()))));
        assert_eq!(raw_indicator.get_signal(), 1);

        synapse.receive(Box::new(SignalMessage(-3, Box::new(synapse.get_id()))));
        assert_eq!(raw_indicator.get_signal(), 0);
    }

    #[test]
    fn capacity_relaxation() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let synapse = new_synapse_fixture(&neuron, Some(10), Some(2));
        let synapse = synapse.borrow();

        let indicator = new_indicator_fixture(&neuron);
        let indicator = indicator.borrow();

        synapse.connect(indicator.get_id().as_str()).unwrap();
        let raw_indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();

        synapse.receive(Box::new(SignalMessage(10, Box::new(synapse.get_id()))));
        assert_eq!(raw_indicator.get_signal(), 10);

        synapse.receive(Box::new(SignalMessage(10, Box::new(synapse.get_id()))));
        assert_eq!(raw_indicator.get_signal(), 2);
    }
}
