use super::specialized::Specialized;

/// An entity that can receive a signal emitted from an emitter or input terminator.
/// Usually it can be represented as Synapse or as OutputTerminator.
/// Should have SpecificationType to differ from other components.
pub trait Acceptor: Specialized {}
