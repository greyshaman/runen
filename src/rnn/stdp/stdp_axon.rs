use tokio::sync::broadcast;

use crate::rnn::common::{arithmetic::Arithmetic, signal::Signal};

pub struct StdpAxon<S>
where
    S: Arithmetic,
{
    sender: broadcast::Sender<Signal<S>>,
}
