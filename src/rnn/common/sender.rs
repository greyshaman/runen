use super::connectable::Connectable;

/// The component is able to send a signal.
pub trait Sender: Connectable {
    /// Sends a signal.
    fn send(&self, signal: i16);
}
