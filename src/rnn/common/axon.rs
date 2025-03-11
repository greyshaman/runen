use std::error::Error;

pub trait Axon {
    type Signal: Sync + Send;

    async fn send(&self, signal: Self::Signal) -> Result<usize, Box<dyn Error>>;
}
