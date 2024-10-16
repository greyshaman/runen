use std::{collections::HashSet, rc::Rc};

use crate::rnn::common::identity::Identity;
use crate::rnn::common::specialized::Specialized;
use crate::rnn::common::spec_type::SpecificationType;
use crate::rnn::common::container::Container;
use crate::rnn::common::connectable::Connectable;
use crate::rnn::common::component::Component;
use crate::rnn::common::aggregator::Aggregator;

/// Нейросома аккумулирует сигналы принятые от дендритов и передаёт
/// результирующий сигнал аксону когда получает повторное уведомление от одного
/// из дендритов
pub struct Neurosoma {
  id: String,
  container: Rc<dyn Container>,

  /// Идентификаторы коллекторов которые прислали обработанные сигналы
  /// агрегатору. Используется для того чтобызапускать агрегацию сигналов
  /// когда поступает повторный сигнал от любого коллектора.
  reported_collectors: HashSet<String>,
  emitter_id: String,
  accumulator: i16,
  connected_collectors_counter: usize,
}

impl Neurosoma {
  pub fn new(id: &str, container: &Rc<dyn Container>) -> Neurosoma {

    Neurosoma {
      id: String::from(id) ,
      container: Rc::clone(container),
      reported_collectors: HashSet::new(),
      emitter_id: "".to_string(),
      accumulator: 1_i16,
      connected_collectors_counter: 0,
    }
  }
}

impl Aggregator for Neurosoma {
    fn notify(&mut self, collector_id: &str, signal: i16) {
      if self.reported_collectors.contains(collector_id)
        || self.reported_collectors.len() >= self.connected_collectors_counter - 1 {
        let new_signal = if self.accumulator > 0 {
          self.accumulator as u8
        } else {
          0_u8
        };
        self.reported_collectors.clear();

        self.accumulator = signal + 1_i16;
        self.reported_collectors.insert(collector_id.to_owned());

        self.aggregate(new_signal);
      } else {
        self.accumulator += signal;
        self.reported_collectors.insert(collector_id.to_owned());
      }
    }

    fn aggregate(&self, signal: u8) {
      if let Some(linked_emitter) =
        &self.container.get_emitter() {
          linked_emitter
            .borrow_mut()
            .emit(signal); // FIXME check and resolve execution collision
        }
    }
}

/// Так как нейросома соединяется только с одним аксоном то соединение определяется
/// нейроном-контейнером
impl Connectable for Neurosoma {}

impl Specialized for Neurosoma {
  fn get_spec_type(&self) -> SpecificationType {
    SpecificationType::Aggregator
  }
}

impl Component for Neurosoma {}

impl Identity for Neurosoma {
  fn get_id(&self) -> String {
    self.id.clone()
  }
}