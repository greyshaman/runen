//! The Neuron is model of biological neuron cell within organelles

use std::{cell::RefCell, collections::{BTreeMap, HashMap}, rc::Rc, sync::atomic::AtomicUsize};

use crate::rnn::common::{emitter::Emitter, media::Media, spec_type::SpecificationType, specialized::Specialized, utils::{gen_id, get_component_id_fraction}};
use crate::rnn::common::container::Container;
use crate::rnn::common::component::Component;
use crate::rnn::common::collector::Collector;
use crate::rnn::common::aggregator::Aggregator;
use crate::rnn::common::acceptor::Acceptor;

use super::synapse::Synapse;

static mut ID_COUNTER: AtomicUsize = AtomicUsize::new(0_usize);

pub struct Neuron {
  id: String,
  network: Rc<RefCell<dyn Media>>,
  // components: BTreeMap<String, Rc<RefCell<dyn Component>>>,
  synapses: BTreeMap<String, Rc<RefCell<dyn Acceptor>>>,
  dendrites: BTreeMap<String, Rc<RefCell<dyn Collector>>>,
  neurosoma: Option<Rc<RefCell<dyn Aggregator>>>,
  axon: Option<Rc<RefCell<dyn Emitter>>>,

  /// used for generate unique components ids
  components_ids_tracker: HashMap<SpecificationType, usize>,
}

impl Neuron {
  pub fn new(id: &str, media: &Rc<RefCell<dyn Media>>) -> Neuron {

    let mut components_ids_tracker = HashMap::with_capacity(4);
    components_ids_tracker.insert(SpecificationType::Acceptor, 0_usize);
    components_ids_tracker.insert(SpecificationType::Collector, 0_usize);
    components_ids_tracker.insert(SpecificationType::Aggregator, 0_usize);
    components_ids_tracker.insert(SpecificationType::Emitter, 0_usize);

    Neuron {
      id: String::from(id) ,
      network: Rc::clone(media),
      components_ids_tracker,
      // components: BTreeMap::new(),
      synapses: BTreeMap::new(),
      dendrites: BTreeMap::new(),
      neurosoma: None,
      axon: None,
    }
  }
}

impl Container for Neuron {
  fn create_acceptor(
      &mut self,
      max_capacity: Option<u8>,
      regeneration_amount: Option<u8>,
  ) {
    let

    let available_id_fraction =
      if let Some(val) = self.synapses.keys().last() {
        get_component_id_fraction(val, &SpecificationType::Acceptor)
          .unwrap() + 1
      } else {
        0
      };

    let acceptor_id = gen_id(
      &self.id,
      available_id_fraction,
      SpecificationType::Acceptor,
    );

    let axon = Synapse::new(
      &acceptor_id,
      &Rc::new(RefCell::new(self)),
      max_capacity,
      regeneration_amount,
    );
  }
}

