#[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// The types of specification.
pub enum SpecificationType {
  Acceptor,
  Collector,
  Aggregator,
  Emitter,
  Container,
  Media,
}