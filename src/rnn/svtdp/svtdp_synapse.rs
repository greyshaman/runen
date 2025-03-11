use std::sync::Arc;

use chrono::TimeDelta;
use tokio::sync::{RwLock, broadcast};

use crate::rnn::common::{arithmetic::Arithmetic, signal::Signal, synapse::Synapse};

pub struct SvtdpSynapse<S>
where
    S: Arithmetic,
{
    id: usize,
    capacity_max: Arc<RwLock<S>>,
    regeneration: Arc<RwLock<S>>,
    weight: Arc<RwLock<S>>,
    processing_delay: Arc<RwLock<TimeDelta>>,

    receiver: Option<Arc<RwLock<broadcast::Receiver<Signal<S>>>>>,
}

impl<S> SvtdpSynapse<S>
where
    S: Arithmetic,
{
    pub fn new(
        id: usize,
        capacity_max: S,
        regeneration: S,
        weight: S,
        processing_delay: TimeDelta,
    ) -> Self {
        SvtdpSynapse {
            id,
            capacity_max: Arc::new(RwLock::new(capacity_max)),
            regeneration: Arc::new(RwLock::new(regeneration)),
            weight: Arc::new(RwLock::new(weight)),
            processing_delay: Arc::new(RwLock::new(processing_delay)),
            receiver: None,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub async fn set_capacity_max(&self, capacity_max: S) {
        let mut w_capacity_max = self.capacity_max.write().await;
        *w_capacity_max = capacity_max;
    }

    pub async fn capacity_max(&self) -> S {
        self.capacity_max.read().await.clone()
    }

    pub async fn set_regeneration(&self, regeneration: S) {
        let mut w_regeneration = self.regeneration.write().await;
        *w_regeneration = regeneration
    }

    pub async fn regeneration(&self) -> S {
        self.regeneration.read().await.clone()
    }

    pub async fn set_weight(&self, weight: S) {
        let mut w_weight = self.weight.write().await;
        *w_weight = weight;
    }

    pub async fn weight(&self) -> S {
        self.weight.read().await.clone()
    }

    pub async fn set_processing_delay(&self, delay: TimeDelta) {
        let mut w_processing_delay = self.processing_delay.write().await;
        *w_processing_delay = delay;
    }

    pub async fn processing_delay(&self) -> TimeDelta {
        self.processing_delay.read().await.clone()
    }
}

impl<S> Synapse<S> for SvtdpSynapse<S>
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
