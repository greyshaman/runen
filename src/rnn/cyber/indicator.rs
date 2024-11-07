use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use as_any::AsAny;
use as_any_derive::AsAny;

use crate::rnn::common::component::Component;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::container::Container;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::signal_msg::SignalMessage;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;

#[derive(Debug, AsAny)]
pub struct Indicator {
    /// The id
    id: String,

    /// Object which contains the indicator
    container: Weak<RefCell<dyn Container>>,

    /// monitored signal
    recent_signal: RefCell<i16>,

    /// monitored signal source id
    recent_source_id: RefCell<String>,
}

impl Indicator {
    pub fn new(id: &str, container: &Rc<RefCell<dyn Container>>) -> Self {
        Indicator {
            id: String::from(id),
            container: Rc::downgrade(container),
            recent_signal: RefCell::new(0),
            recent_source_id: RefCell::new(String::default()),
        }
    }

    pub fn get_signal(&self) -> i16 {
        *self.recent_signal.borrow()
    }

    pub fn get_source_id(&self) -> String {
        self.recent_source_id.borrow().to_string()
    }
}

impl Component for Indicator {
    fn receive(&self, signal_msg: Box<crate::rnn::common::signal_msg::SignalMessage>) {
        let SignalMessage(signal, source_id) = *signal_msg;

        *self.recent_signal.borrow_mut() = signal;
        *self.recent_source_id.borrow_mut() = source_id.to_string();
    }

    fn send(&self, _signal: i16) {
        unimplemented!("Never send signal to other parties");
    }

    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>> {
        self.container.upgrade()
    }
}

impl Specialized for Indicator {
    fn get_spec_type(&self) -> SpecificationType {
        SpecificationType::Indicator
    }
}

impl Identity for Indicator {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Connectable for Indicator {}

#[cfg(test)]
mod tests {
    use crate::rnn::tests::fixtures::new_network_fixture;
    use crate::rnn::tests::fixtures::new_neuron_fixture;
    use crate::rnn::tests::fixtures::{new_indicator_fixture, new_synapse_fixture};

    use super::*;

    #[test]
    fn should_store_received_signal() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);
        let indicator = new_indicator_fixture(&neuron);

        indicator
            .borrow()
            .receive(Box::new(SignalMessage(1, Box::new("123".to_string()))));
        assert_eq!(
            indicator
                .borrow()
                .as_any()
                .downcast_ref::<Indicator>()
                .unwrap()
                .get_signal(),
            1
        );
    }

    #[test]
    fn should_receive_signal_from_observable_component() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let acceptor = new_synapse_fixture(&neuron, Some(2), None);
        let acceptor = acceptor.borrow();

        let indicator = new_indicator_fixture(&neuron);
        let indicator = indicator.borrow();

        let _ = acceptor.connect(indicator.get_id().as_str());
        let raw_indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();

        acceptor.receive(Box::new(SignalMessage(10, Box::new(String::from("test")))));
        assert_eq!(raw_indicator.get_signal(), 2);
    }
}
