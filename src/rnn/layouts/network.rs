use core::fmt;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::any::Any;

use as_any::AsAny;
use as_any_derive::AsAny;
use regex::Regex;

use crate::rnn::common::container::Container;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::media::Media;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::utils::gen_id_by_spec_type;
use crate::rnn::neural::neuron::Neuron;

static mut ID_COUNTER: AtomicUsize = AtomicUsize::new(0_usize);

#[derive(Debug, AsAny)]
pub struct Network {
    id: String,
    containers: HashMap<String, Rc<RefCell<dyn Container>>>,
}

impl Network {
    pub fn new() -> Network {
        let id = gen_id_by_spec_type(
            "",
            unsafe { ID_COUNTER.fetch_add(1, Ordering::Relaxed) },
            &SpecificationType::Network,
        )
        .unwrap();

        Network {
            id,
            containers: HashMap::new(),
        }
    }

    fn get_ids_by_spec_type(&self, spec_type: &SpecificationType) -> Vec<String> {
        self.containers
            .values()
            .filter_map(|item| {
                let item = item.borrow();
                if item.get_spec_type() == *spec_type {
                    Some(item.get_id().clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_available_id_fraction_for(&self, spec_type: &SpecificationType) -> usize {
        self.get_ids_by_spec_type(spec_type).last().map_or(0, |id| {
            if id.is_empty() {
                return 0;
            }

            let r_patter = match spec_type {
                &SpecificationType::Neuron => r"^M\d+Z(\d+)$",
                &SpecificationType::InputTerminator => r"^M\d+Y(\d+)$",
                &SpecificationType::OutputTerminator => r"^M\d+X(\d+)$",
                _ => r"^$",
            };

            let rex = Regex::new(&r_patter).unwrap();
            let captures = rex.captures(id).unwrap();
            if &captures.len() < &2 {
                return 0;
            }
            let id_num = captures[1].parse::<usize>().unwrap();
            id_num + 1
        })
    }
}

impl Media for Network {
    fn get_container(&self, id: &str) -> Option<&Rc<RefCell<dyn Container>>> {
        self.containers.get(&id.to_string())
    }

    fn create_container(
        &mut self,
        spec_type: &SpecificationType,
        media: &Rc<RefCell<dyn Media>>,
    ) -> Result<Rc<RefCell<dyn Container>>, Box<dyn std::error::Error>> {
        let prefix = match spec_type {
            &SpecificationType::Neuron => 'Z',
            &SpecificationType::Receptor => 'Y',
            &SpecificationType::Activator => 'X',
            _ => return Err(Box::new(RnnError::NotSupportedArgValue)),
        };
        let new_id = format!(
            "{}{prefix}{}",
            self.get_id(),
            self.get_available_id_fraction_for(spec_type)
        );
        match self.containers.entry(new_id.clone()) {
            Entry::Vacant(entry) => Ok(Rc::clone(
                entry.insert(Rc::new(RefCell::new(Neuron::new(&new_id, media)))),
            )),
            Entry::Occupied(_) => Err(Box::new(RnnError::IdBusy)),
        }
    }

    fn remove_container(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self.containers.remove(id) {
            Some(_) => Ok(()),
            None => Err(Box::new(RnnError::IdNotFound)),
        }
    }

    fn has_container(&self, id: &str) -> bool {
        self.containers.contains_key(id)
    }

    fn len(&self) -> usize {
        self.containers.len()
    }
}

impl Identity for Network {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Specialized for Network {
    fn get_spec_type(&self) -> SpecificationType {
        SpecificationType::Network
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The Network {} ", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_two_unique_networks() {
        let n1 = Network::new();
        let n2 = Network::new();

        assert_ne!(n1.id, n2.id);
    }

    #[test]
    fn should_create_two_neurons() {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        for _ in 0..=1 {
            assert!(net
                .borrow_mut()
                .create_container(&SpecificationType::Neuron, &net)
                .is_ok());
        }

        assert_eq!(net.borrow().len(), 2);
    }

    #[test]
    fn network_can_get_container_after_create() {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        let neuron_rc = net
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &net)
            .unwrap();
        let neuron_id = neuron_rc.borrow().get_id();

        assert_eq!(net.borrow().len(), 1);
        assert!(
            net.borrow().get_container(neuron_id.as_str()).is_some(),
            "Container not found"
        );
        assert!(
            net.borrow().get_container("missed").is_none(),
            "Should be nothing"
        );
    }

    #[test]
    fn network_can_remove_container_after_create() {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        let neuron_rc = net
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &net)
            .unwrap();
        assert_eq!(net.borrow().len(), 1);

        assert!(net
            .borrow_mut()
            .remove_container(&neuron_rc.borrow().get_id())
            .is_ok());
        assert_eq!(net.borrow().len(), 0);
    }

    #[test]
    fn network_should_return_error_if_remove_by_incorrect_id() {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        let container_rc = net
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &net)
            .unwrap();
        let container_id = container_rc.borrow().get_id();
        assert_eq!(net.borrow().len(), 1);

        assert!(
            net.borrow_mut().remove_container("missed").is_err(),
            "Should return error"
        );

        let binding_media = net.borrow();
        let result = binding_media.get_container(&container_id).unwrap();
        assert_eq!(result.borrow().get_id(), container_id);
    }

    #[test]
    fn network_can_verify_if_contains_container_with_specified_id() {
        let net: Rc<RefCell<dyn Media>> = Rc::new(RefCell::new(Network::new()));

        let neuron_rc = net
            .borrow_mut()
            .create_container(&SpecificationType::Neuron, &net)
            .unwrap();

        assert!(net
            .borrow()
            .has_container(neuron_rc.borrow().get_id().as_str()));
        assert!(!net.borrow().has_container("missed"));
    }

    #[test]
    fn network_returns_correct_id_by_get_id() {
        let net = Network::new();
        assert_eq!(net.id, net.get_id());
    }

    #[test]
    fn should_return_correct_spec_type() {
        let net = Network::new();
        assert_eq!(net.get_spec_type(), SpecificationType::Network);
    }
}
