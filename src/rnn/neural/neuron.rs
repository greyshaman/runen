//! The Neuron is model of biological neuron cell within organelles

use std::rc::Weak;
use std::rc::Rc;
use std::collections::BTreeMap;
use std::cell::RefCell;

use crate::rnn::common::group_type::GroupType;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::utils::gen_id_by_spec_type;
use crate::rnn::common::{grouped::Grouped, utils::get_component_id_fraction};
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::media::Media;
use crate::rnn::common::container::Container;
use crate::rnn::common::component::Component;

use super::axon::Axon;
use super::dendrite::Dendrite;
use super::neurosoma::Neurosoma;
use super::synapse::Synapse;

#[derive(Debug)]
pub struct Neuron {
  id: String,
  network: RefCell<Weak<RefCell<dyn Media>>>,
  components: BTreeMap<String, Rc<RefCell<dyn Component>>>,
}

impl Neuron {
  pub fn new(id: &str, media: &Rc<RefCell<dyn Media>>) -> Neuron {

    Neuron {
      id: String::from(id) ,
      network: RefCell::new(Rc::downgrade(&media)),
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
    self.get_ids_for(spec_type)
      .last()
      .map_or(
        0,
        |id| get_component_id_fraction(id, spec_type).map_or(0, |id_num| id_num + 1)
      )
  }

  fn prepare_new_component_id(&self, spec_type: &SpecificationType) -> String {
    gen_id_by_spec_type(
      &self.id,
      self.get_available_id_fraction_for(spec_type),
      spec_type,
    )
  }
}

impl Container for Neuron {
  fn create_acceptor(&mut self, max_capacity: Option<u8>, regeneration_amount: Option<u8>) {
    let acceptor_id = self.prepare_new_component_id(&SpecificationType::Acceptor);

    let synapse = Synapse::new(
      &acceptor_id,
      self.network.borrow().upgrade().unwrap().borrow().get_container(self.get_id().as_str()).unwrap(),
      max_capacity,
      regeneration_amount,
    );

    self.components.insert(acceptor_id, Rc::new(RefCell::new(synapse)));
  }

  fn create_collector(&mut self, weight: Option<i8>) {
      let collector_id = self.prepare_new_component_id(&SpecificationType::Collector);

      let collector = Dendrite::new(
        &collector_id,
        self.network.borrow().upgrade().unwrap().borrow().get_container(self.get_id().as_str()).unwrap(),
        weight
      );

      self.components.insert(collector_id, Rc::new(RefCell::new(collector)));
  }

  fn create_aggregator(&mut self) {
      let aggregator_id = self.prepare_new_component_id(&SpecificationType::Aggregator);

      let aggregator = Neurosoma::new(
        &aggregator_id,
        self.network.borrow().upgrade().unwrap().borrow().get_container(self.get_id().as_str()).unwrap(),
      );

      self.components.insert(aggregator_id, Rc::new(RefCell::new(aggregator)));
  }

  fn create_emitter(&mut self) {
    let emitter_id = self.prepare_new_component_id(&SpecificationType::Emitter);

    let emitter = Axon::new(
      &emitter_id,
      self.network.borrow().upgrade().unwrap().borrow().get_container(self.get_id().as_str()).unwrap(),
    );

    self.components.insert(emitter_id, Rc::new(RefCell::new(emitter)));
  }

  fn get_component(&self, id: &str) -> Option<&Rc<RefCell<dyn Component>>> {
    self.components.get(id)
  }

  fn remove_component(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    self.components.remove(id)
      .map_or(
        Err(Box::new(RnnError::KeyNotFound)),
        |_| Ok(())
      )
  }

  fn as_any(&self) -> &dyn std::any::Any {
      self
  }

  fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
      self
  }
}

impl Grouped for Neuron {
    fn get_group_type(&self) -> GroupType {
        GroupType::Neural
    }
}

impl Specialized for Neuron {
    fn get_spec_type(&self) -> SpecificationType {
        SpecificationType::Container
    }
}

impl Identity for Neuron {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
