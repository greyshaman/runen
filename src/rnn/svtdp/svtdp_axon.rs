use tokio::sync::broadcast;

use crate::rnn::common::{arithmetic::Arithmetic, signal::Signal};

pub struct SvtdpAxon<S>
where
    S: Arithmetic,
{
    sender: broadcast::Sender<Signal<S>>,
}
