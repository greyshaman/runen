use super::component::Component;

/// сущность способная образовать связь с эмиттером, преобразовать несколько принятых
/// сигналов в один результирующий и передать его эмиттеру
pub trait Aggregator: Component {
  /// оповещающий сигнал может иметь как положительное(возбуждающий сигнал)
  /// так и отрицательный (тормозящий сигнал)
  fn notify(&mut self, collector_id: &str, signal: i16);

  fn aggregate(&self, signal: u8);
}