use std::{error::Error, pin::Pin, sync::{atomic::{AtomicUsize, Ordering}, Arc}, task::Poll};

use tokio::sync::broadcast;
use tokio_stream::Stream;

use crate::rnn::common::{arithmetic::Arithmetic, rnn_error::RnnError, signal::Signal};

/// The output port receives a signal from the neuron and emits an event into the external world.
pub struct OutputPort<S>
where
    S: Arithmetic,
{
    /// Port id
    id: Arc<String>,

    /// signal hits counter
    signal_hits_counter: Arc<AtomicUsize>,

    /// signal receiver
    receiver: Option<broadcast::Receiver<Signal<S>>>,
}

impl<S> OutputPort<S>
where
    S: Arithmetic,
{
    /// Constructs a new output port
    ///
    /// # Examples
    ///
    /// ```
    /// use librunen::rnn::layouts::output_port::OutputPort;
    ///
    /// let port: OutputPort<u8> = OutputPort::new(0);
    /// ```
    ///
    /// ```
    /// use librunen::rnn::layouts::output_port::OutputPort;
    ///
    /// let port = OutputPort::<u8>::new(0);
    /// ```
    pub fn new(id: usize) -> Self {
        OutputPort {
            id: Arc::new(format!("O_{}", id)),
            signal_hits_counter: Arc::new(AtomicUsize::new(0)),
            receiver: None,
        }
    }

    /// Accepts connection from neuron, input port or to broadcast channel
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::broadcast;
    /// use librunen::rnn::layouts::output_port::OutputPort;
    ///
    /// let mut port = OutputPort::<u8>::new(0);
    ///
    /// let (_tx, rx) = broadcast::channel(5);
    /// assert!(port.connect(rx).is_ok());
    /// assert!(port.is_connected());
    /// ```
    pub fn connect(&mut self, rx: broadcast::Receiver<Signal<S>>) -> Result<(), Box<dyn Error>> {
        if self.receiver.is_none() {
            self.receiver = Some(rx);
            Ok(())
        } else {
            Err(Box::new(RnnError::PortBusy(format!(
                "Port {} already connected",
                self.id()
            ))))
        }
    }

    /// Creates new receiver
    ///
    /// # Examples
    ///
    /// ```
    /// use librunen::rnn::layouts::output_port::OutputPort;
    /// use tokio::sync::broadcast;
    ///
    /// let mut port = OutputPort::<u8>::new(0);
    ///
    /// let (_tx, rx) = broadcast::channel(5);
    /// assert!(port.connect(rx).is_ok());
    ///
    /// assert!(port.resubscribe().is_some());
    /// ```
    pub fn resubscribe(&self) -> Option<broadcast::Receiver<Signal<S>>> {
        if let Some(rx) = self.receiver.as_ref() {
            Some(rx.resubscribe())
        } else {
            None
        }
    }

    /// Returns port id
    ///
    /// # Examples
    ///
    /// ```
    /// use librunen::rnn::layouts::output_port::OutputPort;
    ///
    /// let port = OutputPort::<u8>::new(5);
    ///
    /// assert_eq!(port.id(), "O_5");
    /// ```
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns signal hits counter value.
    /// The counter takes into account the reception of signals by the port
    /// if the port is connected to the output stream of values
    ///
    /// # Examples
    ///
    /// ```
    /// use std::error::Error;
    /// use std::sync::Arc;
    /// use chrono::Duration;
    /// use librunen::rnn::layouts::output_port::OutputPort;
    /// use librunen::rnn::common::signal::Signal;
    /// use tokio::sync::{broadcast, Mutex};
    /// use tokio_stream::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let (tx, rx) = broadcast::channel(3);
    ///     let mut port = OutputPort::new(2);
    ///     port.connect(rx)?;
    ///
    ///     let port = Arc::new(Mutex::new(port));
    ///
    ///     let port_cloned = port.clone();
    ///
    ///     let handler_result = tokio::spawn(async move {
    ///         let g_port = port.lock().await;
    ///         let mut stream = Box::pin(g_port);
    ///
    ///         while let Some(msg) = stream.next().await {
    ///             match msg {
    ///                 Ok(msg) => assert!(msg > 0),
    ///                 Err(_) => panic!("Error on signal processing"),
    ///             }
    ///         }
    ///     });
    ///
    ///     for i in 1..=3 {
    ///         tx.send(Signal::new(i))?;
    ///     }
    ///
    ///     drop(tx);
    ///     let (result,) = tokio::join!(handler_result);
    ///     result?;
    ///
    ///     let g_port = port_cloned.lock().await;
    ///     assert_eq!(g_port.signal_hits_count(), 3);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn signal_hits_count(&self) -> usize {
        self.signal_hits_counter.load(Ordering::Relaxed)
    }

    /// Disconnects port from channel
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::broadcast;
    /// use librunen::rnn::layouts::output_port::OutputPort;
    ///
    /// let mut port = OutputPort::<u8>::new(0);
    ///
    /// let (_tx, rx) = broadcast::channel(5);
    /// assert!(port.connect(rx).is_ok());
    ///
    /// port.disconnect();
    /// assert!(!port.is_connected());
    /// ```
    pub fn disconnect(&mut self) {
        if self.receiver.is_some() {
            self.receiver = None;
        }
    }

    /// Checkes if port has incoming connection
    pub fn is_connected(&self) -> bool {
        self.receiver.is_some()
    }
}

impl<S> Stream for OutputPort<S>
where
    S: Arithmetic,
{
    type Item = Result<S, broadcast::error::RecvError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let receiver = match self.receiver.as_mut() {
            Some(rx) => rx,
            None => return Poll::Ready(None),
        };

        match receiver.try_recv() {
            Ok(item) => {
                self.signal_hits_counter.fetch_add(1, Ordering::Release);
                Poll::Ready(Some(Ok(item.intensity())))
            }
            Err(broadcast::error::TryRecvError::Empty) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Lagged(skipped)) => {
                Poll::Ready(Some(Err(broadcast::error::RecvError::Lagged(skipped))))
            }
            Err(broadcast::error::TryRecvError::Closed) => Poll::Ready(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio_stream::StreamExt;
    use tokio::sync::Mutex;

    use super::*;

    async fn process_message<S>(
        port: OutputPort<S>,
        results: Arc<Mutex<Vec<Result<S, broadcast::error::RecvError>>>>,
    ) where
        S: Arithmetic,
    {
        let mut stream = Box::pin(port);
        let mut w_results = results.lock().await;
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(msg) => w_results.push(Ok(msg)),
                Err(err) => w_results.push(Err(err)),
            }
        }
    }

    #[test]
    fn test_port_constructor() {
        let port = OutputPort::<u8>::new(0);

        assert_eq!(port.id.as_str(), "O_0");
    }

    #[test]
    fn test_port_id_getter() {
        let port = OutputPort::<u8>::new(1);

        assert_eq!(port.id(), "O_1");
    }

    #[test]
    fn for_new_port_is_connected_should_returns_false() {
        let port = OutputPort::<u8>::new(1);

        assert!(!port.is_connected());
    }

    #[test]
    fn test_output_port_can_connect_channel() -> Result<(), Box<dyn Error>> {
        let (_tx, rx) = broadcast::channel::<Signal<u8>>(5);
        let mut output_port = OutputPort::<u8>::new(0);

        output_port.connect(rx)?;
        assert!(output_port.is_connected());

        Ok(())
    }

    #[tokio::test]
    async fn when_send_signal_output_port_should_increasing_signal_hits_count()
    -> Result<(), Box<dyn Error>> {
        let (tx, rx) = broadcast::channel::<Signal<u8>>(5);
        let mut output_port = OutputPort::<u8>::new(0);

        output_port.connect(rx)?;

        let output_port = Arc::new(Mutex::new(output_port));

        let port_cloned = output_port.clone();

        let handler_result = tokio::spawn(async move {
            let g_port = output_port.lock().await;
            let mut stream = Box::pin(g_port);

            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(msg) => assert_eq!(msg, 42),
                    Err(_) => panic!("Error on signal processing"),
                }
            }
        });

        tx.send(Signal::new(42))?;
        tx.send(Signal::new(42))?;

        drop(tx);
        let (result,) = tokio::join!(handler_result);

        assert!(result.is_ok());


        let g_port = port_cloned.lock().await;
        assert_eq!(g_port.signal_hits_count(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn consumer_can_subscribe_to_output_port() -> Result<(), Box<dyn Error>> {
        let (tx, rx) = broadcast::channel::<Signal<u8>>(5);
        let mut output_port = OutputPort::<u8>::new(0);

        let results = Arc::new(Mutex::new(vec![]));
        output_port.connect(rx)?;

        let consumer = tokio::spawn(process_message(output_port, results.clone()));

        for i in 1_u8..=5_u8 {
            let result = tx.send(Signal::new(i))?;
            assert!(result > 0);
        }
        drop(tx);

        let _ = tokio::spawn(consumer);

        let g_results = results.lock().await;
        for result in g_results.iter() {
            assert!(result.is_ok());
        }

        Ok(())
    }
}
