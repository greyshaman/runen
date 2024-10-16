/// Cущность умеет подключаться/отключаться к/от другой подключаемой сущности
pub trait Connectable {
  fn connect(&mut self, _party_id: &str) {}
  fn disconnect(&mut self, _party_id: &str) {}
}