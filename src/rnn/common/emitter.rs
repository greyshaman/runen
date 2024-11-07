use super::specialized::Specialized;

/// The Emitter is able to emit a signal, which is then received
/// by the connected Acceptors.
pub trait Emitter: Specialized {}
