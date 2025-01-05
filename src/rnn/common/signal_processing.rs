use std::{error::Error, future::Future, sync::Arc};

use tokio::sync::{broadcast::Receiver, RwLock};

use super::signal::Signal;

pub trait SignalProcessing<S = Signal> {
    /// Send signal to port connected to synapse
    fn input(&self, signal: S, port: usize) -> impl Future<Output = Result<usize, Box<dyn Error>>>;

    fn get_output_receiver(&self, port: usize) -> impl Future<Output = Option<Arc<RwLock<Receiver<S>>>>>;
}