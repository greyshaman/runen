use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::rc::Weak;
use std::any::Any;

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
    container: RefCell<Weak<RefCell<dyn Container>>>,
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
            container: RefCell::new(Rc::downgrade(&container_ref)),
            acceptors: RefCell::new(HashMap::new()),
        })
    }
}

impl Component for Axon {
    fn receive(&mut self, signal_msg: Box<SignalMessage>) {
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
                        .borrow_mut()
                        .receive(Box::new(SignalMessage(signal, Box::new(self.get_id()))));
                })
                .or_else(|| {
                    self.acceptors.borrow_mut().remove(id);
                    Some(())
                });
        }
    }

    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>> {
        self.container.borrow().upgrade()
    }
}

impl Connectable for Axon {
    fn connect(&mut self, party_id: &str) {
        self.container
            .borrow()
            .upgrade()
            .unwrap()
            .borrow()
            .get_component(party_id)
            .map(|acceptor_rc| {
                self.acceptors
                    .borrow_mut()
                    .entry(party_id.to_string())
                    .and_modify(|acceptor_weak| *acceptor_weak = Rc::downgrade(acceptor_rc))
                    .or_insert_with(|| Rc::downgrade(acceptor_rc));
            })
            .or_else(|| {
                self.acceptors.borrow_mut().remove(party_id);
                Some(()) // FIXME check this method twice!!! Or write tests
            });
    }

    fn disconnect(&mut self, party_id: &str) {
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

#[cfg(test)]
mod tests {
    use crate::rnn::tests::mock::mocks::MockComponent;
    use crate::rnn::{common::media::Media, layouts::network::Network};

    use super::*;

    fn fixture_new_axon() -> (Box<Rc<RefCell<dyn Media>>>, Box<Rc<RefCell<dyn Component>>>) {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        let neuron = net
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &net)
            .unwrap();

        let axon = neuron.borrow_mut().create_emitter().unwrap();

        (Box::new(net), Box::new(axon))
    }

    #[test]
    fn can_send_only_positive_signal_with_save_value_as_received_to_all_connected_synapses() {
        let (_net, boxed_axon) = fixture_new_axon();
        let synapse1: Rc<RefCell<dyn Component>> = Rc::new(RefCell::new(MockComponent::default()));
        let synapse2: Rc<RefCell<dyn Component>> = Rc::new(RefCell::new(MockComponent::default()));

        {
            let binding = boxed_axon.borrow();
            let axon = binding.as_any().downcast_ref::<Axon>().unwrap();

            axon.acceptors
                .borrow_mut()
                .insert("1".to_string(), Rc::downgrade(&synapse1));
            axon.acceptors
                .borrow_mut()
                .insert("2".to_string(), Rc::downgrade(&synapse2));
        }

        let mut binding = boxed_axon.borrow_mut();
        let axon_mut = binding.as_mut_any().downcast_mut::<Axon>().unwrap();
        axon_mut.receive(Box::new(SignalMessage(5, Box::new(axon_mut.get_id()))));

        assert_eq!(
            synapse1
                .borrow()
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap()
                .signal,
            5
        );
        assert_eq!(
            synapse2
                .borrow()
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap()
                .signal,
            5
        );

        axon_mut.receive(Box::new(SignalMessage(-5, Box::new(axon_mut.get_id()))));
        assert_eq!(
            synapse1
                .borrow()
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap()
                .signal,
            0
        );
        assert_eq!(
            synapse2
                .borrow()
                .as_any()
                .downcast_ref::<MockComponent>()
                .unwrap()
                .signal,
            0
        );
    }
}
