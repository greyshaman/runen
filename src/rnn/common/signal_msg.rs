/// The message that carries the signal and the sender's identification information.
#[derive(Debug)]
pub struct SignalMessage(pub i16, pub Box<String>);
