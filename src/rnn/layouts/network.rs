use core::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

use regex::Regex;

use crate::rnn::common::container::Container;
use crate::rnn::common::group_type::GroupType;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::media::Media;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::utils::gen_id_by_spec_type;
use crate::rnn::neural::neuron::Neuron;

static mut ID_COUNTER: AtomicUsize = AtomicUsize::new(0_usize);

#[derive(Debug)]
pub struct Network {
  id: String,
  containers: HashMap<String, Rc<RefCell<dyn Container>>>,
}

impl Network {
  pub fn new() -> Network {
    let id = gen_id_by_spec_type(
      "",
      unsafe { ID_COUNTER.fetch_add(1, Ordering::Relaxed) },
      &SpecificationType::Media
    );

    Network {
      id,
      containers: HashMap::new(),
    }
  }

  fn get_ids_by_group_type(&self, group_type: &GroupType) -> Vec<String> {
    self.containers.values()
      .filter_map(|item| {
        let item = item.borrow();
        if item.get_group_type() == *group_type {
          Some(item.get_id().clone())
        } else {
          None
        }
      })
      .collect()
  }

  fn get_available_id_fraction_for(&self, group_type: &GroupType) -> usize {
    self.get_ids_by_group_type(group_type)
      .last()
      .map_or(
        0,
        |id| {
          if id.is_empty() { return 0; }

          let r_patter = match group_type {
            GroupType::Neural => r"Z(\d+)",
            GroupType::Cyber => r"Y(\d+)",
            _ => return 0,
          };

          let rex = Regex::new(&r_patter).unwrap();
          let captures = rex.captures(id).unwrap();
          if &captures.len() < &2 { return 0 }
          let id_num = captures[1].parse::<usize>().unwrap();
          id_num + 1
        }
      )
  }

}

impl Media for Network {
  fn get_container(&self, id: &str) -> Option<&Rc<RefCell<dyn Container>>> {
    self.containers.get(&id.to_string())
  }

  fn create_container(&mut self, group_type: &GroupType, media: &Rc<RefCell<dyn Media>>)
    -> Result<String, Box<dyn std::error::Error>> {
    let prefix = match group_type {
        GroupType::Neural => 'Z',
        GroupType::Cyber => 'Y',
    };
    let new_id = format!("{prefix}{}", self.get_available_id_fraction_for(group_type));

    let container = Neuron::new(&new_id, media);
    self.containers.insert(new_id.clone(), Rc::new(RefCell::new(container)));
    Ok(new_id)
  }

  fn remove_container(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    match self.containers.remove(id) {
      Some(_) => Ok(()),
      None => Err(Box::new(RnnError::KeyNotFound))
    }
  }

  fn has_container(&self, id: &str) -> bool {
    self.containers.contains_key(id)
  }

  fn as_any(&self) -> &dyn std::any::Any {
      self
  }

  fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
      self
  }
}

impl Identity for Network {
  fn get_id(&self) -> String {
    self.id.clone()
  }
}

impl Specialized for Network {
  fn get_spec_type(&self) -> SpecificationType {
    SpecificationType::Media
  }
}

impl fmt::Display for Network {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "The Network {} ", self.id)
  }
}
