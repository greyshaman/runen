use std::{error::Error, sync::Arc};

use tokio::sync::{RwLock, broadcast};

use crate::rnn::common::{arithmetic::Arithmetic, rnn_error::RnnError, signal::Signal};

static CHANNEL_CAPACITY: usize = 5;

/// The InputPort accept external signal and send it into networking to linked neurons
pub struct InputPort<S>
where
    S: Arithmetic,
{
    /// Port id
    id: String,

    /// Signal hits counter
    signal_hits_counter: RwLock<usize>,

    /// signal sender handler
    sender: RwLock<Option<broadcast::Sender<Signal<S>>>>,
}

impl<S> InputPort<S>
where
    S: Arithmetic,
{
    pub fn new(id: usize) -> Self {
        InputPort {
            id: format!("I_{}", id),
            signal_hits_counter: RwLock::new(0),
            sender: RwLock::new(None),
        }
    }

    pub async fn downlink_request(&self) -> broadcast::Receiver<Signal<S>> {
        let mut w_sender = self.sender.write().await;
        if let Some(sender) = w_sender.as_ref() {
            sender.subscribe()
        } else {
            let (sender, receiver) = broadcast::channel(CHANNEL_CAPACITY);
            *w_sender = Some(sender);
            receiver
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub async fn signal_hits_count(&self) -> usize {
        self.signal_hits_counter.read().await.clone()
    }

    /// Send value through port
    pub async fn send(&self, signal_value: S) -> Result<usize, Box<dyn Error>> {
        let r_sender = self.sender.read().await;
        if let Some(sender) = r_sender.as_ref() {
            let result = sender.send(Signal::new(signal_value))?;
            let mut w_signal_hits_counter = self.signal_hits_counter.write().await;
            *w_signal_hits_counter += 1;
            Ok(result)
        } else {
            Err(Box::new(RnnError::SendingWithoutConnection))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_constructor() {
        let port = InputPort::<u8>::new(0);

        assert_eq!(port.id, String::from("I_0"));
    }

    #[test]
    fn test_port_id_getter() {
        let port = InputPort::<u8>::new(1);

        assert_eq!(port.id(), String::from("I_1"));
    }

    #[tokio::test]
    async fn test_signal_hits_should_be_zero_for_new_port() {
        let port = InputPort::<u8>::new(0);

        let counter = port.signal_hits_count().await;
        assert_eq!(counter, 0);
    }

    #[tokio::test]
    async fn test_sending_signal_should_increment_signal_hits() -> Result<(), Box<dyn Error>> {
        let port = Arc::new(InputPort::<u8>::new(0));
        let port_cloned = port.clone();

        let mut receiver = port.downlink_request().await;

        let handler = tokio::spawn(async move {
            while let Ok(signal) = receiver.recv().await {
                if signal.intensity() == 4 {
                    break;
                }
            }
            assert_eq!(port_cloned.signal_hits_count().await, 5);
        });

        for i in 0..5 {
            let result = port.send(i).await?;
            assert!(result > 0);
        }
        handler.await?;
        Ok(())
    }
}
