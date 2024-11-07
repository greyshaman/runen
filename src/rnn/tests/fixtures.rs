use std::{cell::RefCell, rc::Rc};

use crate::rnn::common::component::Component;
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

pub fn new_synapse_fixture(
    container: &Rc<RefCell<dyn Container>>,
    max_capacity: Option<i16>,
    regeneration_amount: Option<i16>,
) -> Rc<RefCell<dyn Component>> {
    container
        .borrow_mut()
        .create_acceptor(max_capacity, regeneration_amount)
        .unwrap()
}

pub fn new_indicator_fixture(container: &Rc<RefCell<dyn Container>>) -> Rc<RefCell<dyn Component>> {
    container.borrow_mut().create_indicator().unwrap()
}

pub fn new_axon_fixture(container: &Rc<RefCell<dyn Container>>) -> Rc<RefCell<dyn Component>> {
    container.borrow_mut().create_emitter().unwrap()
}

pub fn new_dendrite_fixture(
    container: &Rc<RefCell<dyn Container>>,
    weight: Option<i16>,
) -> Rc<RefCell<dyn Component>> {
    container.borrow_mut().create_collector(weight).unwrap()
}

pub fn new_neurosoma_fixture(container: &Rc<RefCell<dyn Container>>) -> Rc<RefCell<dyn Component>> {
    container.borrow_mut().create_aggregator().unwrap()
}
