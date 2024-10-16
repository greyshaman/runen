#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SpecificationType {
  Unknown,
  Acceptor,
  Collector,
  Aggregator,
  Emitter,
  Composer,
  Media,
}