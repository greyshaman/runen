use std::any::{Any, TypeId};
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
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::signal_msg::SignalMessage;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;

use super::neurosoma::Neurosoma;

/// The Dendrite is model of neuron's part
/// It is receive signal from synapse, weighting it and
/// retransmit to neurosoma as aggregator
#[derive(Debug, AsAny)]
pub struct Dendrite {
    id: String,
    container: Weak<RefCell<dyn Container>>,
    weight: i16,
    aggregator: RefCell<Option<Rc<RefCell<dyn Component>>>>,
}

impl Dendrite {
    pub fn new(id: &str, container: &Rc<RefCell<dyn Container>>, weight: i16) -> Dendrite {
        Dendrite {
            id: String::from(id),
            container: Rc::downgrade(&container),
            weight,
            aggregator: RefCell::new(None),
        }
    }
}

impl Component for Dendrite {
    fn receive(&self, signal_msg: Box<SignalMessage>) {
        let SignalMessage(signal, _) = *signal_msg;
        let signal = max(signal, 0);
        let new_signal = self.weight * signal;

        self.send(new_signal);
    }

    fn send(&self, signal: i16) {
        self.aggregator.borrow().as_ref().map(|aggregator_rc| {
            aggregator_rc
                .borrow()
                .receive(Box::new(SignalMessage(signal, Box::new(self.get_id()))));
        });
    }

    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>> {
        self.container.upgrade()
    }
}

impl Connectable for Dendrite {
    fn connect(&self, party_id: &str) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
        if party_id == &self.get_id() {
            return Err(Box::new(RnnError::ClosedLoop));
        }

        let container = self.container.upgrade().unwrap();
        let container = container.borrow();
        if let Some(component) = container.get_component(party_id) {
            *self.aggregator.borrow_mut() = Some(Rc::clone(&component));

            let neurosoma = component.borrow();
            if TypeId::of::<Neurosoma>() == neurosoma.as_any().type_id() {
                let neurosoma = neurosoma.as_any().downcast_ref::<Neurosoma>().unwrap();
                neurosoma.add_signal_source(self.get_id().as_str());
            }
        }

        Ok(())
    }

    fn disconnect(&self, _party_id: &str) {
        self.aggregator.borrow().as_ref().map(|aggregator_rc| {
            aggregator_rc
                .borrow()
                .as_any()
                .downcast_ref::<Neurosoma>()
                .unwrap()
                .remove_signal_source(self.get_id().as_str());
        });
        *self.aggregator.borrow_mut() = None;
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

macro_rules! create_dendrite {
    ($id:expr, $container:expr) => {
        Dendrite::new($id, $container, 1)
    };
    ($id:expr, $container:expr, $weight:expr) => {
        Dendrite::new($id, $container, $weight)
    };
}

#[cfg(test)]
mod tests {
    use crate::rnn::cyber::indicator::Indicator;
    use crate::rnn::tests::fixtures::new_dendrite_fixture;
    use crate::rnn::tests::fixtures::new_indicator_fixture;
    use crate::rnn::tests::fixtures::new_network_fixture;
    use crate::rnn::tests::fixtures::new_neuron_fixture;

    use super::*;

    #[test]
    fn should_accept_incoming_positive_input_signal() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let dendrite_ref = new_dendrite_fixture(&neuron, None);
        let dendrite = dendrite_ref.borrow();

        let indicator_ref = new_indicator_fixture(&neuron);
        let indicator = indicator_ref.borrow();

        dendrite.connect(indicator.get_id().as_str()).unwrap();
        let raw_indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();

        dendrite.receive(Box::new(SignalMessage(3, Box::new(String::from("test")))));
        assert_eq!(raw_indicator.get_signal(), 3);

        dendrite.receive(Box::new(SignalMessage(-3, Box::new(String::from("test")))));
        assert_eq!(raw_indicator.get_signal(), 0);
    }

    #[test]
    fn should_gain_incoming_positive_input_signal() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let dendrite_ref = new_dendrite_fixture(&neuron, Some(5));
        let dendrite = dendrite_ref.borrow();

        let indicator_ref = new_indicator_fixture(&neuron);
        let indicator = indicator_ref.borrow();

        dendrite.connect(indicator.get_id().as_str()).unwrap();
        let raw_indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();

        dendrite.receive(Box::new(SignalMessage(1, Box::new(String::from("test")))));
        assert_eq!(raw_indicator.get_signal(), 5);

        dendrite.receive(Box::new(SignalMessage(-1, Box::new(String::from("test")))));
        assert_eq!(raw_indicator.get_signal(), 0);
    }

    #[test]
    fn should_produce_inhibitory_signal_based_on_incoming_positive_input_signal() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let dendrite_ref = new_dendrite_fixture(&neuron, Some(-2));
        let dendrite = dendrite_ref.borrow();

        let indicator_ref = new_indicator_fixture(&neuron);
        let indicator = indicator_ref.borrow();

        dendrite.connect(indicator.get_id().as_str()).unwrap();
        let raw_indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();

        dendrite.receive(Box::new(SignalMessage(5, Box::new(dendrite.get_id()))));
        assert_eq!(raw_indicator.get_signal(), -10);

        dendrite.receive(Box::new(SignalMessage(-5, Box::new(dendrite.get_id()))));
        assert_eq!(raw_indicator.get_signal(), 0);
    }
}
