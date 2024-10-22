/// An entity can connect to or disconnect from another connected entity.
pub trait Connectable {
  fn connect(&mut self, _party_id: &str) {}
  fn disconnect(&mut self, _party_id: &str) {}
}