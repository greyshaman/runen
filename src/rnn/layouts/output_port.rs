use std::{
    error::Error,
    pin::Pin,
    sync::{Arc, mpsc::Receiver},
};

use tokio::sync::{RwLock, broadcast, mpsc};
use tokio_stream::Stream;

use crate::rnn::common::{arithmetic::Arithmetic, rnn_error::RnnError, signal::Signal};

pub trait Subscriber<S>: Stream<Item = S>
where
    S: Arithmetic,
{
    fn set_receiver(&mut self, receiver: mpsc::Receiver<S>) -> Result<(), Box<dyn Error>>;
}

/// The OutputPort receive signal from neuron and should emit event into external world
/// Выходной порт может иметь подписчиков которые подписываются на изменение данных
/// и выполняет функции замыкания подписчиков
pub struct OutputPort<S>
where
    S: Arithmetic,
{
    /// Port id
    id: Arc<String>,

    /// signal hits counter
    signal_hits_counter: Arc<RwLock<usize>>,

    /// signal receive
    inner_receiver: Option<Arc<RwLock<broadcast::Receiver<Signal<S>>>>>,

    /// output channel
    outgoing_sender: Arc<RwLock<Option<Arc<mpsc::Sender<S>>>>>,
}

impl<S> OutputPort<S>
where
    S: Arithmetic,
{
    pub fn new(id: usize) -> Self {
        OutputPort {
            id: Arc::new(format!("O_{}", id)),
            signal_hits_counter: Arc::new(RwLock::new(0)),
            inner_receiver: None,
            outgoing_sender: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn subscribe(
        &self,
        subscriber: Arc<RwLock<dyn Subscriber<S>>>,
        outgoing_buffer_size: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut w_outgoing_sender_opt = self.outgoing_sender.write().await;

        if w_outgoing_sender_opt.is_none() {
            let (tx, rx) = mpsc::channel(outgoing_buffer_size);
            *w_outgoing_sender_opt = Some(Arc::new(tx));
            let mut w_subscriber = subscriber.write().await;
            w_subscriber.set_receiver(rx)
        } else {
            Err(Box::new(RnnError::PortBusy(format!(
                "Port {} is busy",
                self.id
            ))))
        }
    }

    pub async fn unsubscribe(&self) {
        let mut w_outgoing_sender_opt = self.outgoing_sender.write().await;

        if w_outgoing_sender_opt.is_some() {
            *w_outgoing_sender_opt = None;
        }
    }

    pub async fn is_subscribed(&self) -> bool {
        self.outgoing_sender.read().await.is_some()
    }

    pub fn id(&self) -> Arc<String> {
        self.id.clone()
    }

    pub async fn signal_hits_count(&self) -> usize {
        self.signal_hits_counter.read().await.clone()
    }

    pub async fn inner_connect(
        &mut self,
        receiver: broadcast::Receiver<Signal<S>>,
    ) -> Result<(), Box<dyn Error>> {
        if self.inner_receiver.is_none() {
            let inner_receiver = Arc::new(RwLock::new(receiver));
            self.inner_receiver = Some(inner_receiver.clone());

            let signal_hits_counter = self.signal_hits_counter.clone();
            let outgoing_sender_opt = self.outgoing_sender.clone();

            tokio::spawn(async move {
                let mut w_inner_receiver = inner_receiver.write().await;
                while let Ok(signal) = w_inner_receiver.recv().await {
                    let mut w_signal_hits_counter = signal_hits_counter.write().await;
                    *w_signal_hits_counter += 1;

                    let r_outgoing_sender_opt = outgoing_sender_opt.read().await;
                    if let Some(outgoing_sender) = r_outgoing_sender_opt.as_ref() {
                        outgoing_sender.send(signal.intensity()).await; // TODO: handle result to notify about sending error
                    }
                }
            });

            Ok(())
        } else {
            Err(Box::new(RnnError::PortBusy(format!(
                "Port {} is busy",
                self.id
            ))))
        }
    }

    pub fn inner_disconnect(&mut self) {
        if self.inner_receiver.is_some() {
            self.inner_receiver = None;
        }
    }

    pub fn is_inner_connected(&self) -> bool {
        self.inner_receiver.is_some()
    }
}

#[cfg(test)]
mod tests {

    use std::{pin::Pin, task::Poll, time::Duration};

    use tokio_stream::{Stream, StreamExt};

    use super::*;

    struct ConcreteSubscriber<S>
    where
        S: Arithmetic,
    {
        receiver: Option<mpsc::Receiver<S>>,
    }

    impl<S> Subscriber<S> for ConcreteSubscriber<S>
    where
        S: Arithmetic,
    {
        fn set_receiver(&mut self, receiver: mpsc::Receiver<S>) -> Result<(), Box<dyn Error>> {
            if self.receiver.is_none() {
                self.receiver = Some(receiver);
                Ok(())
            } else {
                Err(Box::new(std::fmt::Error::default()) as Box<dyn Error>)
            }
        }
    }

    impl<S> Stream for ConcreteSubscriber<S>
    where
        S: Arithmetic,
    {
        type Item = S;

        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            if let Some(receiver) = &mut self.receiver {
                Pin::new(receiver).poll_recv(cx)
            } else {
                Poll::Ready(None)
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

        assert_eq!(port.id().as_str(), "O_1");
    }

    #[test]
    fn for_new_port_is_inner_connected_should_returns_false() {
        let port = OutputPort::<u8>::new(1);

        assert!(!port.is_inner_connected());
    }

    #[tokio::test]
    async fn test_output_port_can_inner_connect_channel() -> Result<(), Box<dyn Error>> {
        let (_tx, rx) = broadcast::channel::<Signal<u8>>(5);
        let mut output_port = OutputPort::<u8>::new(0);

        output_port.inner_connect(rx).await?;
        assert!(output_port.is_inner_connected());

        Ok(())
    }

    #[tokio::test]
    async fn when_send_signal_output_port_should_increasing_signal_hits_count()
    -> Result<(), Box<dyn Error>> {
        let (tx, rx) = broadcast::channel::<Signal<u8>>(5);
        let mut output_port = OutputPort::<u8>::new(0);

        output_port.inner_connect(rx).await?;

        tx.send(Signal::new(42))?;

        tokio::time::sleep(Duration::from_micros(1)).await;

        assert_eq!(output_port.signal_hits_count().await, 1);

        Ok(())
    }

    #[tokio::test]
    async fn consumer_can_subscribe_to_output_port() -> Result<(), Box<dyn Error>> {
        let (tx, rx) = broadcast::channel::<Signal<u8>>(5);
        let mut output_port = OutputPort::<u8>::new(0);
        let concrete_subscriber = ConcreteSubscriber { receiver: None };
        let concrete_subscriber = Arc::new(RwLock::new(concrete_subscriber));

        output_port.inner_connect(rx).await?;

        output_port
            .subscribe(concrete_subscriber.clone(), 5)
            .await?;

        tokio::spawn(async move {
            let mut w_concrete_subscriber = concrete_subscriber.write().await;
            while let Some(msg) = w_concrete_subscriber.next().await {
                assert!(msg > 0);
                if msg == 5 {
                    break;
                }
            }
        });

        for i in 1_u8..=5_u8 {
            let result = tx.send(Signal::new(i))?;
            assert!(result > 0);
        }
        Ok(())
    }

    #[tokio::test]
    async fn for_new_output_port_is_subscribed_returns_false() {
        let port = OutputPort::<u8>::new(0);

        assert!(!port.is_subscribed().await);
    }

    #[tokio::test]
    async fn when_consumer_was_subscribed_to_output_port_is_subscribed_should_return_true() {
        let port = OutputPort::<i64>::new(0);

        let concrete_subscriber = Arc::new(RwLock::new(ConcreteSubscriber { receiver: None }));

        port.subscribe(concrete_subscriber, 5).await;

        assert!(port.is_subscribed().await);
    }

    #[tokio::test]
    async fn test_unsubscribe_subscribed_consumer_from_port() {
        let port = OutputPort::<usize>::new(0);

        let consumer = Arc::new(RwLock::new(ConcreteSubscriber { receiver: None }));

        port.subscribe(consumer, 2).await;

        port.unsubscribe().await;

        assert!(!port.is_subscribed().await);
    }
}
