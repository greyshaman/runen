use super::component::Component;

/// Сущность Эмиттер способная испускать результирующий сигнал, который принимают подключенные
/// акцепторы.
pub trait Emitter: Component {
  /// Sending signal to all consumers
  fn emit(&self, signal: u8);
}