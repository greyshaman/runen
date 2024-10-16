use super::component::Component;

/// Сущность способная собирать и взвешивать принятый сигнал
/// и передавать обработанный сигнал для дальнейшей обработки агрегатору
pub trait Collector: Component {
  fn collect(&self, signal: u8);
}