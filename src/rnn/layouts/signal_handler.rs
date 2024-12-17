use std::sync::Arc;

use tokio::sync::{
    broadcast::{Receiver, Sender},
    RwLock,
};

use crate::rnn::common::signal::Signal;

#[derive(Debug)]
pub enum SignalHandler {
    Input(Arc<RwLock<Sender<Signal>>>),
    Output(Arc<RwLock<Receiver<Signal>>>),
}
