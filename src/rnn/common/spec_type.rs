/// The types of specification.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SpecificationType {
    Acceptor,
    Collector,
    Aggregator,
    Emitter,
    Container,
    Media,
}

impl SpecificationType {
    pub fn is_multiple_allowed(&self) -> bool {
        match *self {
            SpecificationType::Aggregator | SpecificationType::Emitter => false,
            _ => true,
        }
    }
}
