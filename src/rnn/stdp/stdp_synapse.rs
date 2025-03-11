use std::sync::Arc;

use chrono::TimeDelta;
use tokio::sync::{RwLock, broadcast};

use crate::rnn::common::{arithmetic::Arithmetic, signal::Signal, synapse::Synapse};

pub struct StdpSynapse<S>
where
    S: Arithmetic,
{
    id: usize,
    processing_delay: Arc<RwLock<TimeDelta>>,
    // connected: Arc<RwLock<Option<String>>>,
    receiver: Option<Arc<RwLock<broadcast::Receiver<Signal<S>>>>>,
}

impl<S> StdpSynapse<S>
where
    S: Arithmetic,
{
    pub fn new(id: usize, delay: TimeDelta) -> Self {
        StdpSynapse {
            id,
            processing_delay: Arc::new(RwLock::new(delay)),
            // connected: Arc::new(RwLock::new(None)),
            receiver: None,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub async fn set_processing_delay(&self, delay: TimeDelta) {
        let mut w_processing_delay = self.processing_delay.write().await;
        *w_processing_delay = delay;
    }

    pub async fn processing_delay(&self) -> TimeDelta {
        self.processing_delay.read().await.clone()
    }

    pub fn receiver(&self) -> Option<Arc<RwLock<broadcast::Receiver<Signal<S>>>>> {
        self.receiver.as_ref().map(|recv| recv.clone())
    }
}

impl<S> Synapse<S> for StdpSynapse<S>
where
    S: Arithmetic,
{
    async fn receive(&self, signal: Signal<S>) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn is_connected(&self) -> bool {
        self.receiver.is_some()
    }
}
