use std::any::Any;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::rc::Weak;

use as_any::AsAny;
use as_any_derive::AsAny;

use crate::rnn::common::component::Component;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::container::Container;
use crate::rnn::common::emitter::Emitter;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::signal_msg::SignalMessage;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;

/// The Axon is able to emit a signal, which is then received
/// by the connected Synapses.
#[derive(Debug, AsAny)]
pub struct Axon {
    id: String,
    container: Weak<RefCell<dyn Container>>,
    acceptors: RefCell<HashMap<String, Weak<RefCell<dyn Component>>>>,
}

impl Axon {
    pub fn new(
        id: &str,
        container_ref: &Rc<RefCell<dyn Container>>,
    ) -> Result<Axon, Box<dyn Error>> {
        let spec_type = SpecificationType::Axon;

        if !spec_type.is_id_valid(id) {
            return Err(Box::new(RnnError::NotSupportedArgValue));
        }

        Ok(Axon {
            id: String::from(id),
            container: Rc::downgrade(&container_ref),
            acceptors: RefCell::new(HashMap::new()),
        })
    }
}

impl Component for Axon {
    fn receive(&self, signal_msg: Box<SignalMessage>) {
        let SignalMessage(signal, _) = *signal_msg;
        self.send(max(signal, 0));
    }

    fn send(&self, signal: i16) {
        // FIXME use channels to improve signal sending
        for (id, acceptor_weak) in self.acceptors.borrow_mut().iter() {
            acceptor_weak
                .upgrade()
                .map(|acceptor_rc| {
                    acceptor_rc
                        .borrow()
                        .receive(Box::new(SignalMessage(signal, Box::new(self.get_id()))));
                })
                .or_else(|| {
                    self.acceptors.borrow_mut().remove(id);
                    Some(())
                });
        }
    }

    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>> {
        self.container.upgrade()
    }
}

impl Connectable for Axon {
    fn connect(&self, party_id: &str) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
        if party_id == &self.get_id() {
            return Err(Box::new(RnnError::ClosedLoop));
        }

        self.container
            .upgrade()
            .unwrap()
            .borrow()
            .get_component(party_id)
            .map(|acceptor_rc| {
                self.acceptors
                    .borrow_mut()
                    .entry(party_id.to_string())
                    .and_modify(|acceptor_weak| *acceptor_weak = Rc::downgrade(&acceptor_rc))
                    .or_insert_with(|| Rc::downgrade(&acceptor_rc));
            })
            .or_else(|| {
                self.acceptors.borrow_mut().remove(party_id);
                Some(()) // FIXME check this method twice!!! Or write tests
            });

        Ok(())
    }

    fn disconnect(&self, party_id: &str) {
        self.acceptors.borrow_mut().remove(party_id);
    }
}

impl Specialized for Axon {
    fn get_spec_type(&self) -> SpecificationType {
        SpecificationType::Axon
    }
}

impl Identity for Axon {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Emitter for Axon {}

macro_rules! create_axon {
    ($id:expr, $container:expr) => {
        Axon::new($id, $container)
    };
}

#[cfg(test)]
mod tests {

    use crate::rnn::cyber::indicator::Indicator;
    use crate::rnn::tests::fixtures::new_axon_fixture;
    use crate::rnn::tests::fixtures::new_indicator_fixture;
    use crate::rnn::tests::fixtures::new_network_fixture;
    use crate::rnn::tests::fixtures::new_neuron_fixture;

    use super::*;

    #[test]
    fn axon_macro_should_create_new_instance_as_result() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let neuron_id = neuron.borrow().get_id();
        let axon_id = format!("{neuron_id}{}", "E0");

        assert!(create_axon!(&axon_id, &(*neuron)).is_ok())
    }

    #[test]
    fn can_send_only_positive_signal_with_save_value_as_received_to_all_connected_synapses() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);
        let axon = new_axon_fixture(&neuron);
        let indicator1 = new_indicator_fixture(&neuron);
        let indicator2 = new_indicator_fixture(&neuron);

        let axon = axon.borrow();
        axon.connect(indicator1.borrow().get_id().as_str()).unwrap();
        axon.connect(indicator2.borrow().get_id().as_str()).unwrap();

        let indicator1 = indicator1.borrow();
        let indicator1 = indicator1.as_any().downcast_ref::<Indicator>().unwrap();

        let indicator2 = indicator2.borrow();
        let indicator2 = indicator2.as_any().downcast_ref::<Indicator>().unwrap();

        axon.receive(Box::new(SignalMessage(5, Box::new(axon.get_id()))));

        assert_eq!(indicator1.get_signal(), 5);
        assert_eq!(indicator2.get_signal(), 5);

        axon.receive(Box::new(SignalMessage(-5, Box::new(axon.get_id()))));
        assert_eq!(indicator1.get_signal(), 0);
        assert_eq!(indicator2.get_signal(), 0);
    }

    #[test]
    fn should_send_signal_to_connected_synapse_from_same_neuron() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);
        let axon = new_axon_fixture(&neuron);

        let indicator = new_indicator_fixture(&neuron);

        let axon = axon.borrow();
        axon.connect(indicator.borrow().get_id().as_str()).unwrap();

        axon.receive(Box::new(SignalMessage(2, Box::new(String::from("test")))));

        let indicator = indicator.borrow();
        let indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();
        assert_eq!(indicator.get_signal(), 2);
        assert_eq!(indicator.get_source_id(), axon.get_id().as_str())
    }

    #[test]
    fn should_send_signal_to_connected_synapse_from_other_neuron() {
        let net = new_network_fixture();
        let neuron1 = new_neuron_fixture(&net);
        let neuron2 = new_neuron_fixture(&net);
        let axon = new_axon_fixture(&neuron1);

        let indicator = new_indicator_fixture(&neuron2);

        let axon = axon.borrow();
        axon.connect(indicator.borrow().get_id().as_str()).unwrap();

        assert_eq!(Rc::weak_count(&indicator), 1);

        axon.receive(Box::new(SignalMessage(2, Box::new(String::from("test")))));

        let indicator = indicator.borrow();
        let indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();
        assert_eq!(indicator.get_signal(), 2);
        assert_eq!(indicator.get_source_id(), axon.get_id().as_str())
    }
}
