pub trait SignalInput<T> {
  fn input(&mut self, input_vector: Vec<T>);
}