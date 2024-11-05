//! The Neuron is model of biological neuron cell within organelles

use std::any::Any;
use std::cell::RefCell;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::error::Error;
use std::rc::Rc;
use std::rc::Weak;

use as_any::AsAny;
use as_any_derive::AsAny;

use crate::rnn::common::component::Component;
use crate::rnn::common::container::Container;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::media::Media;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::utils::check_id_on_siblings;
use crate::rnn::common::utils::gen_id_by_spec_type;
use crate::rnn::common::utils::get_component_id_fraction;

use super::axon::Axon;
use super::dendrite::Dendrite;
use super::neurosoma::Neurosoma;
use super::synapse::Synapse;

#[derive(Debug, AsAny)]
pub struct Neuron {
    id: String,
    network: Weak<RefCell<dyn Media>>,
    components: BTreeMap<String, Rc<RefCell<dyn Component>>>,
}

impl Neuron {
    pub fn new(id: &str, media: &Rc<RefCell<dyn Media>>) -> Neuron {
        Neuron {
            id: String::from(id),
            network: Rc::downgrade(&media),
            components: BTreeMap::new(),
        }
    }

    fn get_ids_for(&self, spec_type: &SpecificationType) -> Vec<String> {
        self.components
            .values()
            .filter_map(|item| {
                let item = item.borrow();
                if item.get_spec_type() == *spec_type {
                    Some(item.get_id().to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_available_id_fraction_for(&self, spec_type: &SpecificationType) -> usize {
        self.get_ids_for(spec_type).last().map_or(0, |id| {
            get_component_id_fraction(id, spec_type).map_or(0, |id_num| id_num + 1)
        })
    }

    fn prepare_new_component_id(
        &self,
        spec_type: &SpecificationType,
    ) -> Result<String, Box<dyn Error>> {
        gen_id_by_spec_type(
            &self.id,
            self.get_available_id_fraction_for(spec_type),
            spec_type,
        )
    }
}

impl Container for Neuron {
    fn create_acceptor(
        &mut self,
        max_capacity: Option<i16>,
        regeneration_amount: Option<i16>,
    ) -> Result<Rc<RefCell<dyn Component>>, Box<dyn Error>> {
        let acceptor_id = self.prepare_new_component_id(&SpecificationType::Synapse)?;

        if !check_id_on_siblings(&acceptor_id, &SpecificationType::Synapse) {
            return Err(Box::new(RnnError::OnlySingleAllowed));
        }

        let synapse = Synapse::new(
            &acceptor_id,
            self.network
                .upgrade()
                .unwrap()
                .borrow()
                .get_container(self.get_id().as_str())
                .unwrap(),
            max_capacity.unwrap_or(1),
            regeneration_amount.unwrap_or(1),
        );

        Ok(Rc::clone(
            self.components
                .entry(acceptor_id)
                .or_insert(Rc::new(RefCell::new(synapse))),
        ))
    }

    fn create_collector(
        &mut self,
        weight: Option<i16>,
    ) -> Result<Rc<RefCell<dyn Component>>, Box<dyn Error>> {
        let collector_id = self.prepare_new_component_id(&SpecificationType::Dendrite)?;

        if !check_id_on_siblings(&collector_id, &SpecificationType::Dendrite) {
            return Err(Box::new(RnnError::OnlySingleAllowed));
        }

        let collector = Dendrite::new(
            &collector_id,
            self.network
                .upgrade()
                .unwrap()
                .borrow()
                .get_container(self.get_id().as_str())
                .unwrap(),
            weight.unwrap_or(1),
        );

        Ok(Rc::clone(
            self.components
                .entry(collector_id)
                .or_insert(Rc::new(RefCell::new(collector))),
        ))
    }

    fn create_aggregator(&mut self) -> Result<Rc<RefCell<(dyn Component)>>, Box<(dyn Error)>> {
        let aggregator_id = self.prepare_new_component_id(&SpecificationType::Neurosoma)?;

        if !check_id_on_siblings(&aggregator_id, &SpecificationType::Neurosoma) {
            return Err(Box::new(RnnError::OnlySingleAllowed));
        }

        let aggregator = Neurosoma::new(
            &aggregator_id,
            self.network
                .upgrade()
                .unwrap()
                .borrow()
                .get_container(self.get_id().as_str())
                .unwrap(),
        );

        match self.components.entry(aggregator_id) {
            Entry::Vacant(entry) => {
                let value = Rc::new(RefCell::new(aggregator));
                Ok(Rc::clone(entry.insert(value)))
            }
            Entry::Occupied(_) => Err(Box::new(RnnError::OnlySingleAllowed)),
        }
    }

    fn create_emitter(&mut self) -> Result<Rc<RefCell<dyn Component>>, Box<dyn Error>> {
        let emitter_id = self.prepare_new_component_id(&SpecificationType::Axon)?;

        if !check_id_on_siblings(&emitter_id, &SpecificationType::Axon) {
            return Err(Box::new(RnnError::OnlySingleAllowed));
        }

        let emitter = create_axon!(
            &emitter_id,
            self.network
                .upgrade()
                .unwrap()
                .borrow()
                .get_container(self.get_id().as_str())
                .unwrap()
        )?;

        match self.components.entry(emitter_id) {
            Entry::Vacant(entry) => {
                let value = Rc::new(RefCell::new(emitter));
                Ok(Rc::clone(entry.insert(value)))
            }
            Entry::Occupied(_) => Err(Box::new(RnnError::OnlySingleAllowed)),
        }
    }

    fn get_component(&self, id: &str) -> Option<&Rc<RefCell<dyn Component>>> {
        self.components.get(id)
    }

    fn get_component_mut(&mut self, id: &str) -> Option<&mut Rc<RefCell<dyn Component>>> {
        self.components.get_mut(id)
    }

    fn remove_component(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.components
            .remove(id)
            .map_or(Err(Box::new(RnnError::IdNotFound)), |_| Ok(()))
    }

    fn len(&self) -> usize {
        self.components.len()
    }

    fn len_by_spec_type(&self, spec_type: &SpecificationType) -> usize {
        self.components
            .values()
            .filter(|item| item.borrow().get_spec_type() == *spec_type)
            .count()
    }
}

impl Specialized for Neuron {
    fn get_spec_type(&self) -> SpecificationType {
        SpecificationType::Neuron
    }
}

impl Identity for Neuron {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::rnn::layouts::network::Network;

    use super::*;

    fn fixture_new_empty_neuron() -> (Box<Rc<RefCell<dyn Media>>>, Box<Rc<RefCell<dyn Container>>>)
    {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        let neuron = net
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &net)
            .unwrap();

        (Box::new(net), Box::new(neuron))
    }

    mod for_empty_neuron {
        use super::*;

        #[test]
        fn get_ids_for_should_return_empty_list() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert!(new_neuron
                .borrow()
                .as_any()
                .downcast_ref::<Neuron>()
                .unwrap()
                .get_ids_for(&SpecificationType::Neurosoma)
                .is_empty());
        }

        #[test]
        fn get_available_id_fraction_for_should_return_zero() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert_eq!(
                new_neuron
                    .borrow()
                    .as_any()
                    .downcast_ref::<Neuron>()
                    .unwrap()
                    .get_available_id_fraction_for(&SpecificationType::Synapse),
                0
            );
        }

        #[test]
        fn prepare_new_component_id_should_return_available_id() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            let neuron_id = new_neuron.borrow().get_id();

            let available_id = new_neuron
                .borrow()
                .as_any()
                .downcast_ref::<Neuron>()
                .unwrap()
                .prepare_new_component_id(&SpecificationType::Synapse)
                .unwrap();

            assert_eq!(available_id, format!("{}{}", neuron_id, "A0"));
        }

        #[test]
        fn can_add_one_neurosoma() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert!(new_neuron.borrow_mut().create_aggregator().is_ok());
        }

        #[test]
        fn can_add_one_axon() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert!(new_neuron.borrow_mut().create_emitter().is_ok());
        }

        #[test]
        fn can_add_one_synapse() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert!(new_neuron.borrow_mut().create_acceptor(None, None).is_ok());
        }

        #[test]
        fn get_container_should_return_none() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert!(new_neuron.borrow().get_component("M0Z0C0").is_none());
        }

        #[test]
        fn remove_component_should_returns_error() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert!(new_neuron.borrow_mut().remove_component("M0Z0A0").is_err());
        }

        #[test]
        fn len_should_return_zero() {
            let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

            let new_neuron = net
                .borrow_mut()
                .create_container(&SpecificationType::Neuron, &net)
                .unwrap();

            assert_eq!(new_neuron.borrow().len(), 0);
        }

        #[test]
        fn len_by_spec_type_for_some_spec_type_should_return_zero() {
            let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

            let new_neuron = net
                .borrow_mut()
                .create_container(&SpecificationType::Neuron, &net)
                .unwrap();

            assert_eq!(
                new_neuron
                    .borrow()
                    .len_by_spec_type(&SpecificationType::Synapse),
                0
            );
        }
    }

    mod for_non_empty_neuron {

        use super::*;

        #[test]
        fn get_ids_for_should_returns_non_empty_list() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            let _ = new_neuron.borrow_mut().create_acceptor(None, None);

            assert!(!new_neuron
                .borrow_mut()
                .as_any()
                .downcast_ref::<Neuron>()
                .unwrap()
                .get_ids_for(&SpecificationType::Synapse)
                .is_empty());
        }

        #[test]
        fn cannot_add_new_one_neurosoma() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert!(new_neuron.borrow_mut().create_aggregator().is_ok());
            assert!(new_neuron.borrow_mut().create_aggregator().is_err());
        }

        #[test]
        fn cannot_add_new_one_axon() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert!(new_neuron.borrow_mut().create_emitter().is_ok());
            assert!(new_neuron.borrow_mut().create_emitter().is_err());
        }

        #[test]
        fn can_add_new_one_synapse() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            assert!(new_neuron.borrow_mut().create_acceptor(None, None).is_ok());
            assert!(new_neuron.borrow_mut().create_acceptor(None, None).is_ok());
        }

        #[test]
        fn get_container_should_return_component() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            let component_id = new_neuron
                .borrow_mut()
                .create_acceptor(None, None)
                .unwrap()
                .borrow()
                .get_id();

            assert!(new_neuron.borrow().get_component(&component_id).is_some());
            assert_eq!(
                new_neuron
                    .borrow()
                    .get_component(&component_id)
                    .unwrap()
                    .borrow()
                    .get_id(),
                component_id
            );
        }

        #[test]
        fn neuron_has_no_one_component_after_remove_component_without_error() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            let component_id = new_neuron
                .borrow_mut()
                .create_acceptor(None, None)
                .unwrap()
                .borrow()
                .get_id();

            assert_eq!(
                new_neuron
                    .borrow()
                    .len_by_spec_type(&SpecificationType::Synapse),
                1
            );
            assert!(new_neuron
                .borrow_mut()
                .remove_component(&component_id)
                .is_ok());
            assert_eq!(new_neuron.borrow().len(), 0);
        }

        #[test]
        fn len_should_return_positive_number() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            let _ = new_neuron.borrow_mut().create_acceptor(None, None);

            assert!(new_neuron.borrow_mut().len() > 0);
        }

        #[test]
        fn len_by_spec_type_should_return_positive_number() {
            let (_net, new_neuron) = fixture_new_empty_neuron();

            let _ = new_neuron.borrow_mut().create_acceptor(None, None);

            assert!(
                new_neuron
                    .borrow_mut()
                    .len_by_spec_type(&SpecificationType::Synapse)
                    > 0
            );
            assert_eq!(
                new_neuron
                    .borrow_mut()
                    .len_by_spec_type(&SpecificationType::Dendrite),
                0
            );
        }
    }
}
