use std::sync::atomic::{AtomicUsize, Ordering};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::rnn::common::container::Container;
use crate::rnn::common::identity::Identity;
use crate::rnn::common::media::Media;
use crate::rnn::common::rnn_error::RnnError;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::utils::gen_id;

static mut ID_COUNTER: AtomicUsize = AtomicUsize::new(0_usize);

pub struct Network {
  id: String,
  containers: HashMap<String, Rc<RefCell<dyn Container>>>,
}

impl Network {
  pub fn new() -> Network {
    let id = gen_id(
      "",
      unsafe { ID_COUNTER.fetch_add(1, Ordering::Relaxed) },
      SpecificationType::Media
    );

    Network {
      id,
      containers: HashMap::new(),
    }
  }
}

impl Media for Network {
  fn get_container(&self, id: &str) -> Option<&Rc<RefCell<dyn Container>>> {
    self.containers.get(&id.to_string())
  }

  fn insert_container(&mut self, container_ref: &Rc<RefCell<dyn Container>>)
    -> Result<String, Box<dyn std::error::Error>> {
    let id = container_ref.as_ref().borrow().get_id();
    if self.containers.contains_key(&id) {
      return Err(Box::new(RnnError::OccupiedKey));
    }

    let value = Rc::clone(container_ref);
    self.containers.insert(id.to_string(), value);
    Ok(id)
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