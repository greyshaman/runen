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
use crate::rnn::common::rnn_error::RnnError;
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
    reported_collectors: RefCell<HashSet<String>>,
    accumulator: RefCell<i16>,
    emitter: RefCell<Option<Rc<RefCell<dyn Component>>>>,
}

impl Neurosoma {
    pub fn new(id: &str, container: &Rc<RefCell<dyn Container>>) -> Neurosoma {
        Neurosoma {
            id: String::from(id),
            container: Rc::downgrade(container),
            connected_collectors: RefCell::new(HashSet::new()),
            reported_collectors: RefCell::new(HashSet::new()),
            accumulator: RefCell::new(1_i16),
            emitter: RefCell::new(None),
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

    fn reset(&self) -> i16 {
        let new_signal = max(*self.accumulator.borrow(), 0);

        *self.accumulator.borrow_mut() = 1;
        self.reported_collectors.borrow_mut().clear();

        new_signal
    }

    fn register_signal(&self, signal: i16, source_id: &str) {
        *self.accumulator.borrow_mut() += signal;
        self.reported_collectors
            .borrow_mut()
            .insert(source_id.to_string());
    }
}

impl Component for Neurosoma {
    fn receive(&self, signal_msg: Box<SignalMessage>) {
        let SignalMessage(signal, boxed_source_id) = *signal_msg;
        let collector_id = *boxed_source_id;
        if self.reported_collectors.borrow().contains(&collector_id) {
            let new_signal = self.reset();
            self.register_signal(signal, &collector_id);
            self.send(new_signal);
        } else {
            self.register_signal(signal, &collector_id);
            if self.reported_collectors.borrow().len() >= self.count_referrals() {
                let new_signal = self.reset();
                self.send(new_signal);
            }
        }
    }

    fn send(&self, signal: i16) {
        self.emitter.borrow().as_ref().map(|emitter_rc| {
            emitter_rc
                .borrow()
                .receive(Box::new(SignalMessage(signal, Box::new(self.get_id()))));
        });
    }

    fn get_container(&self) -> Option<Rc<RefCell<dyn Container>>> {
        self.container.upgrade()
    }
}

impl Connectable for Neurosoma {
    fn connect(&self, party_id: &str) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
        if party_id == &self.get_id() {
            return Err(Box::new(RnnError::ClosedLoop));
        }

        *self.emitter.borrow_mut() = self
            .container
            .upgrade()
            .unwrap()
            .borrow()
            .get_component(party_id)
            .map(|emitter_rc| Rc::clone(&emitter_rc));

        Ok(())
    }

    fn disconnect(&self, _party_id: &str) {
        *self.emitter.borrow_mut() = None;
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
    use crate::rnn::{
        cyber::indicator::Indicator,
        tests::fixtures::{
            new_dendrite_fixture, new_indicator_fixture, new_network_fixture, new_neuron_fixture,
            new_neurosoma_fixture,
        },
    };

    use super::*;

    #[test]
    fn should_add_two_unit_signals_with_activation_and_produce_triple_signal() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let neurosoma = new_neurosoma_fixture(&neuron);
        let neurosoma = neurosoma.borrow();

        let dendrite1 = new_dendrite_fixture(&neuron, None);
        let dendrite1 = dendrite1.borrow();

        let dendrite2 = new_dendrite_fixture(&neuron, None);
        let dendrite2 = dendrite2.borrow();

        dendrite1.connect(neurosoma.get_id().as_str()).unwrap();
        dendrite2.connect(neurosoma.get_id().as_str()).unwrap();

        let indicator = new_indicator_fixture(&neuron);
        let indicator = indicator.borrow();
        neurosoma.connect(indicator.get_id().as_str()).unwrap();
        let raw_indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();

        dendrite1.receive(Box::new(SignalMessage(1, Box::new(String::default()))));
        dendrite2.receive(Box::new(SignalMessage(1, Box::new(String::default()))));

        assert_eq!(raw_indicator.get_signal(), 3);
    }

    #[test]
    fn should_produce_zero_signal_when_stopping_signal_prevails() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let neurosoma = new_neurosoma_fixture(&neuron);
        let neurosoma = neurosoma.borrow();

        let dendrite1 = new_dendrite_fixture(&neuron, None);
        let dendrite1 = dendrite1.borrow();

        let dendrite2 = new_dendrite_fixture(&neuron, Some(-3));
        let dendrite2 = dendrite2.borrow();

        dendrite1.connect(neurosoma.get_id().as_str()).unwrap();
        dendrite2.connect(neurosoma.get_id().as_str()).unwrap();

        let indicator = new_indicator_fixture(&neuron);
        let indicator = indicator.borrow();
        neurosoma.connect(indicator.get_id().as_str()).unwrap();
        let raw_indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();

        dendrite1.receive(Box::new(SignalMessage(1, Box::new(String::default()))));
        dendrite2.receive(Box::new(SignalMessage(1, Box::new(String::default()))));

        assert_eq!(raw_indicator.get_signal(), 0);
    }

    #[test]
    fn should_not_produce_signal_when_some_dendrites_did_not_sent_signals() {
        let net = new_network_fixture();
        let neuron = new_neuron_fixture(&net);

        let neurosoma = new_neurosoma_fixture(&neuron);
        let neurosoma = neurosoma.borrow();

        let dendrite1 = new_dendrite_fixture(&neuron, None);
        let dendrite1 = dendrite1.borrow();

        let dendrite2 = new_dendrite_fixture(&neuron, None);
        let dendrite2 = dendrite2.borrow();

        dendrite1.connect(neurosoma.get_id().as_str()).unwrap();
        dendrite2.connect(neurosoma.get_id().as_str()).unwrap();

        let indicator = new_indicator_fixture(&neuron);
        let indicator = indicator.borrow();
        neurosoma.connect(indicator.get_id().as_str()).unwrap();
        let raw_indicator = indicator.as_any().downcast_ref::<Indicator>().unwrap();

        dendrite1.receive(Box::new(SignalMessage(1, Box::new(String::default()))));

        assert_eq!(raw_indicator.get_signal(), 0);
    }
}
