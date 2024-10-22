#[derive(Clone, PartialEq, Eq, Hash)]

/// The types of specification.
pub enum SpecificationType {
  Unknown,
  Acceptor,
  Collector,
  Aggregator,
  Emitter,
  Container,
  Media,
}