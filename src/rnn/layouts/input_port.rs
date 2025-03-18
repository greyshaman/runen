use std::{error::Error, sync::{atomic::{AtomicUsize, Ordering}, Arc}};

use tokio::sync::broadcast;

use crate::rnn::common::{arithmetic::Arithmetic, rnn_error::RnnError, signal::Signal};

static CHANNEL_CAPACITY: usize = 5;

/// The InputPort accept external signal and send it into networking to linked neurons
pub struct InputPort<S>
where
    S: Arithmetic,
{
    /// Port id
    id: Arc<String>,

    /// Signal hits counter
    signal_hits_counter: Arc<AtomicUsize>,

    /// signal sender handler
    sender: Option<broadcast::Sender<Signal<S>>>,
}

impl<S> InputPort<S>
where
    S: Arithmetic,
{
    /// Constructs a new input port
    ///
    /// # Examples
    ///
    /// ```
    /// use librunen::rnn::layouts::input_port::InputPort;
    ///
    /// let port: InputPort<u8> = InputPort::new(0);
    /// ```
    ///
    /// ```
    /// use librunen::rnn::layouts::input_port::InputPort;
    ///
    /// let port = InputPort::<u8>::new(0);
    /// ```
    pub fn new(id: usize) -> Self {
        InputPort {
            id: Arc::new(format!("I_{}", id)),
            signal_hits_counter: Arc::new(AtomicUsize::new(0)),
            sender: None,
        }
    }

    /// Provides receiver for connected party
    ///
    /// # Examples
    ///
    /// ```
    /// use librunen::rnn::layouts::input_port::InputPort;
    ///
    /// let mut port = InputPort::<u8>::new(0);
    ///
    /// let rx = port.downlink_request();
    /// ```
    pub fn downlink_request(&mut self) -> broadcast::Receiver<Signal<S>> {
        if let Some(sender) = self.sender.as_ref() {
            sender.subscribe()
        } else {
            let (sender, receiver) = broadcast::channel(CHANNEL_CAPACITY);
            self.sender = Some(sender);
            receiver
        }
    }

    /// Returns port id
    ///
    /// # Examples
    ///
    /// ```
    /// use librunen::rnn::layouts::input_port::InputPort;
    ///
    /// let port = InputPort::<u8>::new(0);
    ///
    /// assert_eq!(port.id(), "I_0");
    /// ```
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns signal hits counter value
    ///
    /// # Examples
    ///
    /// ```
    /// use librunen::rnn::layouts::input_port::InputPort;
    ///
    /// let mut port = InputPort::<u8>::new(0);
    /// assert_eq!(port.signal_hits_count(), 0);
    ///
    /// let rx = port.downlink_request();
    /// port.send(42).expect("signal sending should work");
    ///
    /// assert_eq!(port.signal_hits_count(), 1);
    /// ```
    pub fn signal_hits_count(&self) -> usize {
        self.signal_hits_counter.load(Ordering::Relaxed)
    }

    /// Sends a value through the port
    ///
    /// # Examples
    ///
    /// ```
    /// use librunen::rnn::layouts::input_port::InputPort;
    ///
    /// let mut port = InputPort::<u8>::new(0);
    /// assert_eq!(port.signal_hits_count(), 0);
    ///
    /// let rx = port.downlink_request();
    /// assert!(port.send(42).is_ok());
    /// ```
    pub fn send(&mut self, signal_value: S) -> Result<usize, Box<dyn Error>> {
        if let Some(sender) = self.sender.as_ref() {
            let result = sender.send(Signal::new(signal_value))?;
            self.signal_hits_counter.fetch_add(1, Ordering::Release);
            Ok(result)
        } else {
            Err(Box::new(RnnError::SendingWithoutConnection))
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::RwLock;

    use super::*;

    #[test]
    fn test_port_constructor() {
        let port = InputPort::<u8>::new(0);

        assert_eq!(port.id.as_str(), "I_0");
    }

    #[test]
    fn test_port_id_getter() {
        let port = InputPort::<u8>::new(1);

        assert_eq!(port.id(), "I_1");
    }

    #[test]
    fn test_signal_hits_should_be_zero_for_new_port() {
        let port = InputPort::<u8>::new(0);

        let counter = port.signal_hits_count();
        assert_eq!(counter, 0);
    }

    #[test]
    fn test_when_send_value_to_not_connected_port_returns_error() {
        let mut port = InputPort::<u8>::new(0);
        assert!(port.send(42).is_err());
    }

    #[tokio::test]
    async fn test_sending_signal_should_increment_signal_hits() -> Result<(), Box<dyn Error>> {
        let port = Arc::new(RwLock::new(InputPort::<u8>::new(0)));
        let port_cloned = port.clone();

        let mut receiver = port_cloned.write().await.downlink_request();

        let handler = tokio::spawn(async move {
            while let Ok(signal) = receiver.recv().await {
                if signal.intensity() == 4 {
                    break;
                }
            }
            assert_eq!(port_cloned.read().await.signal_hits_count(), 5);
        });

        for i in 0..5 {
            let result = port.write().await.send(i)?;
            assert!(result > 0);
        }
        handler.await?;
        Ok(())
    }
}
