pub trait SignalOutput<T> {
  fn output(&mut self) -> Vec<T>;
}