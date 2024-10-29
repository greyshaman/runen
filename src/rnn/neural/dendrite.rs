use std::any::Any;
use std::cell::RefCell;
use std::cmp::max;
use std::rc::{Rc, Weak};

use as_any::AsAny;
use as_any_derive::AsAny;

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
#[derive(Debug, AsAny)]
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
        let signal = max(signal, 0);
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

#[cfg(test)]
mod tests {
    use crate::rnn::common::media::Media;
    use crate::rnn::layouts::network::Network;
    use crate::rnn::tests::mocks::MockComponent;

    use super::*;

    fn fixture_new_dendrite(
        weight: Option<i16>,
    ) -> (Box<Rc<RefCell<dyn Media>>>, Box<Rc<RefCell<dyn Component>>>) {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        let neuron = net
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &net)
            .unwrap();

        let dendrite = neuron.borrow_mut().create_collector(weight).unwrap();

        (Box::new(net), Box::new(dendrite))
    }

    #[test]
    fn should_accept_incoming_positive_input_signal() {
        let (_net, boxed_dendrite) = fixture_new_dendrite(None);
        let neurosoma_rc: Rc<RefCell<dyn Component>> =
            Rc::new(RefCell::new(MockComponent::default()));

        let mut component = boxed_dendrite.borrow_mut();
        let dendrite = component.as_mut_any().downcast_mut::<Dendrite>().unwrap();

        dendrite.aggregator = Some(Rc::clone(&neurosoma_rc));

        {
            dendrite.receive(Box::new(SignalMessage(3, Box::new(dendrite.get_id()))));
            let mock_neurosoma = neurosoma_rc.borrow();
            let mock_neurosoma = mock_neurosoma
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap();
            assert_eq!(mock_neurosoma.signal, 3);
        }

        {
            dendrite.receive(Box::new(SignalMessage(-3, Box::new(dendrite.get_id()))));
            let mock_neurosoma = neurosoma_rc.borrow();
            let mock_neurosoma = mock_neurosoma
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap();
            assert_eq!(mock_neurosoma.signal, 0);
        }
    }

    #[test]
    fn should_gain_incoming_positive_input_signal() {
        let (_net, boxed_dendrite) = fixture_new_dendrite(Some(5));
        let neurosoma_rc: Rc<RefCell<dyn Component>> =
            Rc::new(RefCell::new(MockComponent::default()));

        let mut component = boxed_dendrite.borrow_mut();
        let dendrite = component.as_mut_any().downcast_mut::<Dendrite>().unwrap();

        dendrite.aggregator = Some(Rc::clone(&neurosoma_rc));

        {
            dendrite.receive(Box::new(SignalMessage(1, Box::new(dendrite.get_id()))));
            let mock_neurosoma = neurosoma_rc.borrow();
            let mock_neurosoma = mock_neurosoma
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap();
            assert_eq!(mock_neurosoma.signal, 5);
        }

        {
            dendrite.receive(Box::new(SignalMessage(-1, Box::new(dendrite.get_id()))));
            let mock_neurosoma = neurosoma_rc.borrow();
            let mock_neurosoma = mock_neurosoma
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap();
            assert_eq!(mock_neurosoma.signal, 0);
        }
    }

    #[test]
    fn should_produce_inhibitory_signal_based_on_incoming_positive_input_signal() {
        let (_net, boxed_dendrite) = fixture_new_dendrite(Some(-2));
        let neurosoma_rc: Rc<RefCell<dyn Component>> =
            Rc::new(RefCell::new(MockComponent::default()));

        let mut component = boxed_dendrite.borrow_mut();
        let dendrite = component.as_mut_any().downcast_mut::<Dendrite>().unwrap();

        dendrite.aggregator = Some(Rc::clone(&neurosoma_rc));

        {
            dendrite.receive(Box::new(SignalMessage(5, Box::new(dendrite.get_id()))));
            let mock_neurosoma = neurosoma_rc.borrow();
            let mock_neurosoma = mock_neurosoma
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap();
            assert_eq!(mock_neurosoma.signal, -10);
        }

        {
            dendrite.receive(Box::new(SignalMessage(-5, Box::new(dendrite.get_id()))));
            let mock_neurosoma = neurosoma_rc.borrow();
            let mock_neurosoma = mock_neurosoma
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap();
            assert_eq!(mock_neurosoma.signal, 0);
        }
    }
}
