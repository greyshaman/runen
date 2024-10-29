use std::{cell::RefCell, rc::Rc};

use crate::rnn::common::container::Container;
use crate::rnn::common::media::Media;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::layouts::network::Network;

pub fn new_network_fixture() -> Box<Rc<RefCell<dyn Media>>> {
    Box::new(Rc::new(RefCell::new(Network::new())))
}

pub fn new_neuron_fixture(network: &Rc<RefCell<dyn Media>>) -> Box<Rc<RefCell<dyn Container>>> {
    Box::new(
        network
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &network)
            .unwrap(),
    )
}
