use super::component::Component;

/// Сущность способная принимать сигнал испущенный эмиттером
pub trait Acceptor: Component {
  /// Принимает сигнал для дальнейшей обработки и возможной передачи далее
  fn accept(&mut self, signal: u8);
}