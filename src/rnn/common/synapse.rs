use std::error::Error;

use super::{arithmetic::Arithmetic, signal::Signal};

pub trait Synapse<S>
where
    S: Arithmetic,
{
    async fn receive(&self, signal: Signal<S>) -> Result<(), Box<dyn Error>>;

    fn is_connected(&self) -> bool;
}
