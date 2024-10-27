use std::cell::RefCell;
use std::cmp::{max, min};
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
        let signal = max(signal, 0);
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

#[cfg(test)]
mod tests {
    use std::boxed;

    use crate::rnn::common::media::Media;
    use crate::rnn::layouts::network::Network;
    use crate::rnn::tests::mock::mocks::MockComponent;

    use super::*;

    fn fixture_new_synapse(
        max_capacity: Option<i16>,
        regeneration_amount: Option<i16>,
    ) -> (Box<Rc<RefCell<dyn Media>>>, Box<Rc<RefCell<dyn Component>>>) {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        let neuron = net
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &net)
            .unwrap();

        let synapse = neuron
            .borrow_mut()
            .create_acceptor(max_capacity, regeneration_amount)
            .unwrap();

        (Box::new(net), Box::new(synapse))
    }

    #[test]
    fn should_limit_input_signal_by_max_capacity() {
        let (_net, boxed_synapse) = fixture_new_synapse(None, None);
        let collector: Rc<RefCell<dyn Component>> = Rc::new(RefCell::new(MockComponent::default()));

        let mut component = boxed_synapse.borrow_mut();
        let synapse = component.as_mut_any().downcast_mut::<Synapse>().unwrap();

        synapse.collector = Some(Rc::clone(&collector));

        {
            synapse.receive(Box::new(SignalMessage(3, Box::new(synapse.get_id()))));
            let mock_collector = collector.borrow();
            let mock_collector = mock_collector
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap();
            assert_eq!(mock_collector.signal, 1);
        }

        {
            synapse.receive(Box::new(SignalMessage(-3, Box::new(synapse.get_id()))));
            let mock_collector = collector.borrow();
            let mock_collector = mock_collector
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap();
            assert_eq!(mock_collector.signal, 0);
        }
    }
}
