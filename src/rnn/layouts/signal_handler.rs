use std::sync::Arc;

use tokio::sync::{
    broadcast::{Receiver, Sender},
    RwLock,
};

use crate::rnn::common::signal::Signal;

#[derive(Debug)]
pub enum SignalHandler<S = Signal> {
    Input(Arc<RwLock<Sender<S>>>),
    Output(Arc<RwLock<Receiver<S>>>),
}
