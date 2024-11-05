use std::any::Any;
use std::cell::RefCell;
use std::cmp::max;
use std::rc::Weak;
use std::{collections::HashSet, rc::Rc};

use as_any::AsAny;
use as_any_derive::AsAny;

use crate::rnn::common::aggregator::Aggregator;
use crate::rnn::common::component::Component;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::container::Container;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::signal_msg::SignalMessage;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;

/// The neurosoma collects the signals received from
/// the dendrites and sends the resulting signal down
/// the axon when it receives repeated signals from one
/// of the dendrites.
#[derive(Debug, AsAny)]
pub struct Neurosoma {
    id: String,
    container: Weak<RefCell<dyn Container>>,
    connected_collectors: RefCell<HashSet<String>>,

    /// The IDs of the collectors who sent the processed signals
    /// to the aggregator are used to trigger signal aggregation
    /// when a repeated signal is received from any
    /// of these collectors.
    reported_collectors: HashSet<String>,
    accumulator: i16,
    emitter: Option<Rc<RefCell<dyn Component>>>,
}

impl Neurosoma {
    pub fn new(id: &str, container: &Rc<RefCell<dyn Container>>) -> Neurosoma {
        Neurosoma {
            id: String::from(id),
            container: Rc::downgrade(container),
            connected_collectors: RefCell::new(HashSet::new()),
            reported_collectors: HashSet::new(),
            accumulator: 1_i16,
            emitter: None,
        }
    }

    pub fn add_signal_source(&self, source_id: &str) {
        self.connected_collectors
            .borrow_mut()
            .insert(source_id.to_string());
    }

    pub fn remove_signal_source(&self, source_id: &str) {
        self.connected_collectors.borrow_mut().remove(source_id);
    }

    fn count_referrals(&self) -> usize {
        self.connected_collectors.borrow().len()
    }

    fn reset(&mut self) -> i16 {
        let new_signal = max(self.accumulator, 0);

        self.accumulator = 1;
        self.reported_collectors.clear();

        new_signal
    }

    fn register_signal(&mut self, signal: i16, source_id: &str) {
        self.accumulator += signal;
        self.reported_collectors.insert(source_id.to_string());
    }
}

impl Component for Neurosoma {
    fn receive(&mut self, signal_msg: Box<SignalMessage>) {
        let SignalMessage(signal, boxed_source_id) = *signal_msg;
        let collector_id = *boxed_source_id;
        if self.reported_collectors.contains(&collector_id) {
            let new_signal = self.reset();
            self.register_signal(signal, &collector_id);
            self.send(new_signal);
        } else {
            self.register_signal(signal, &collector_id);
            if self.reported_collectors.len() >= self.count_referrals() {
                let new_signal = self.reset();
                self.send(new_signal);
            }
        }
    }

    fn send(&self, signal: i16) {
        self.emitter.as_ref().map(|emitter_rc| {
            emitter_rc
                .borrow_mut()
                .receive(Box::new(SignalMessage(signal, Box::new(self.get_id()))));
        });
    }

    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>> {
        self.container.upgrade()
    }
}

impl Connectable for Neurosoma {
    fn connect(&mut self, party_id: &str) {
        self.emitter = self
            .container
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
        SpecificationType::Neurosoma
    }
}

impl Identity for Neurosoma {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Aggregator for Neurosoma {}

macro_rules! create_neurosoma {
    ($id:expr, $container:expr) => {
        Neurosoma::new($id, $container)
    };
}

#[cfg(test)]
mod tests {
    use crate::rnn::common::media::Media;
    use crate::rnn::tests::fixtures::{new_network_fixture, new_neuron_fixture};
    use crate::rnn::tests::mocks::MockComponent;

    use super::*;

    fn new_neurosoma_fixture() -> (Box<Rc<RefCell<dyn Media>>>, Box<Rc<RefCell<dyn Component>>>) {
        let boxed_net = new_network_fixture();

        let boxed_neuron = new_neuron_fixture(&boxed_net);

        let neurosoma = boxed_neuron.borrow_mut().create_aggregator().unwrap();

        (boxed_net, Box::new(neurosoma))
    }

    #[test]
    fn should_add_two_unit_signals_with_activation_and_produce_triple_signal() {
        let (_net, boxed_neurosoma) = new_neurosoma_fixture();

        let neuron = Rc::clone(&boxed_neurosoma.borrow().get_container().unwrap());

        let dendrite1 = neuron.borrow_mut().create_collector(None).unwrap();

        let dendrite2 = neuron.borrow_mut().create_collector(None).unwrap();

        let mock_axon: Rc<RefCell<dyn Component>> = Rc::new(RefCell::new(MockComponent::default()));

        {
            dendrite1
                .borrow_mut()
                .connect(boxed_neurosoma.borrow().get_id().as_str());
            dendrite2
                .borrow_mut()
                .connect(boxed_neurosoma.borrow().get_id().as_str());
        }

        {
            let mut component = boxed_neurosoma.borrow_mut();
            let neurosoma = component.as_any_mut().downcast_mut::<Neurosoma>().unwrap();

            neurosoma.emitter = Some(Rc::clone(&mock_axon));
        }

        dendrite1
            .borrow_mut()
            .receive(Box::new(SignalMessage(1, Box::new(String::default()))));
        dendrite2
            .borrow_mut()
            .receive(Box::new(SignalMessage(1, Box::new(String::default()))));

        let axon_binding = mock_axon.borrow();
        let mock_axon = axon_binding
            .as_any()
            .downcast_ref::<MockComponent>()
            .unwrap();
        assert_eq!(mock_axon.signal, 3);
    }

    #[test]
    fn should_produce_zero_signal_when_stopping_signal_greater() {
        let (_net, boxed_neurosoma) = new_neurosoma_fixture();

        let neuron = Rc::clone(&boxed_neurosoma.borrow().get_container().unwrap());

        let dendrite1 = neuron.borrow_mut().create_collector(None).unwrap();

        let dendrite2 = neuron.borrow_mut().create_collector(Some(-3)).unwrap();

        let mock_axon: Rc<RefCell<dyn Component>> = Rc::new(RefCell::new(MockComponent::default()));

        {
            dendrite1
                .borrow_mut()
                .connect(boxed_neurosoma.borrow().get_id().as_str());
            dendrite2
                .borrow_mut()
                .connect(boxed_neurosoma.borrow().get_id().as_str());
        }

        {
            let mut component = boxed_neurosoma.borrow_mut();
            let neurosoma = component.as_any_mut().downcast_mut::<Neurosoma>().unwrap();

            neurosoma.emitter = Some(Rc::clone(&mock_axon));
        }

        dendrite1
            .borrow_mut()
            .receive(Box::new(SignalMessage(1, Box::new(String::default()))));
        dendrite2
            .borrow_mut()
            .receive(Box::new(SignalMessage(1, Box::new(String::default()))));

        let axon_binding = mock_axon.borrow();
        let mock_axon = axon_binding
            .as_any()
            .downcast_ref::<MockComponent>()
            .unwrap();
        assert_eq!(mock_axon.signal, 0);
    }

    #[test]
    fn should_not_produce_signal_when_some_dendrites_did_not_sent_signals() {
        let (_net, boxed_neurosoma) = new_neurosoma_fixture();

        let neuron = Rc::clone(&boxed_neurosoma.borrow().get_container().unwrap());

        let dendrite1 = neuron.borrow_mut().create_collector(None).unwrap();

        let dendrite2 = neuron.borrow_mut().create_collector(None).unwrap();

        let mock_axon: Rc<RefCell<dyn Component>> = Rc::new(RefCell::new(MockComponent::default()));

        {
            dendrite1
                .borrow_mut()
                .connect(boxed_neurosoma.borrow().get_id().as_str());
            dendrite2
                .borrow_mut()
                .connect(boxed_neurosoma.borrow().get_id().as_str());
        }

        {
            let mut component = boxed_neurosoma.borrow_mut();
            let neurosoma = component.as_any_mut().downcast_mut::<Neurosoma>().unwrap();

            neurosoma.emitter = Some(Rc::clone(&mock_axon));
        }

        dendrite1
            .borrow_mut()
            .receive(Box::new(SignalMessage(1, Box::new(String::default()))));

        let axon_binding = mock_axon.borrow();
        let mock_axon = axon_binding
            .as_any()
            .downcast_ref::<MockComponent>()
            .unwrap();
        assert_eq!(mock_axon.signal, 0);
    }
}
