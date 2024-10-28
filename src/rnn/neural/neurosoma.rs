use std::cell::RefCell;
use std::cmp::max;
use std::rc::Weak;
use std::{collections::HashSet, rc::Rc};
use std::any::Any;

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
    container: RefCell<Weak<RefCell<dyn Container>>>,

    /// The IDs of the collectors who sent the processed signals
    /// to the aggregator are used to trigger signal aggregation
    /// when a repeated signal is received from any
    /// of these collectors.
    reported_collectors: HashSet<String>,
    emitter: Option<Rc<RefCell<dyn Component>>>,
    accumulator: i16,
}

impl Neurosoma {
    pub fn new(id: &str, container: &Rc<RefCell<dyn Container>>) -> Neurosoma {
        Neurosoma {
            id: String::from(id),
            container: RefCell::new(Rc::downgrade(container)),
            reported_collectors: HashSet::new(),
            emitter: None,
            accumulator: 1_i16,
        }
    }

    fn count_referrals(&self) -> usize {
        self.container
            .borrow()
            .upgrade()
            .and_then(|container_rc| {
                container_rc
                    .borrow()
                    .get_component(self.get_id().as_str())
                    .map(|collector_rc| Rc::strong_count(&collector_rc))
            })
            .unwrap_or(0_usize)
    }
}

impl Component for Neurosoma {
    fn receive(&mut self, signal_msg: Box<SignalMessage>) {
        let SignalMessage(signal, boxed_source_id) = *signal_msg;
        let collector_id = *boxed_source_id;
        if self.reported_collectors.contains(&collector_id)
            || self.reported_collectors.len() >= self.count_referrals() - 1
        {
            let new_signal = max(self.accumulator, 0);

            self.reported_collectors.clear();

            self.accumulator = signal + 1_i16;
            self.reported_collectors.insert(collector_id);

            self.send(new_signal);
        } else {
            self.accumulator += signal;
            self.reported_collectors.insert(collector_id);
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
        self.container.borrow().upgrade()
    }
}

impl Connectable for Neurosoma {
    fn connect(&mut self, party_id: &str) {
        self.emitter = self
            .container
            .borrow()
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

#[cfg(test)]
mod tests {
    use crate::rnn::{common::media::Media, layouts::network::Network};

    use super::*;

    fn fixture_new_neurosoma() -> (Box<Rc<RefCell<dyn Media>>>, Box<Rc<RefCell<dyn Component>>>) {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        let neuron = net
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &net)
            .unwrap();

        let neurosoma = neuron.borrow_mut().create_aggregator().unwrap();

        (Box::new(net), Box::new(neurosoma))
    }
}
