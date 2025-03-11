use std::{
    error::Error,
    pin::Pin,
    task::{Context, Poll},
};

use librunen::rnn::common::rnn_error::RnnError;
use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt};

pub struct Subscriber {
    receiver: mpsc::Receiver<String>,
}

pub struct Publisher {
    sender: Option<mpsc::Sender<String>>,
}

impl Subscriber {
    pub fn new(receiver: mpsc::Receiver<String>) -> Self {
        Subscriber { receiver }
    }
}

impl Stream for Subscriber {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_recv(cx)
    }
}

impl Publisher {
    pub fn new() -> Self {
        Publisher { sender: None }
    }

    pub fn build_subscriber(&mut self, channel_size: usize) -> Result<Subscriber, Box<dyn Error>> {
        if self.sender.is_some() {
            Err(Box::new(RnnError::PortBusy(String::from("Port busy"))))
        } else {
            let (tx, rx) = mpsc::channel(channel_size);
            self.sender = Some(tx);
            Ok(Subscriber::new(rx))
        }
    }

    pub async fn send(&self, msg: &str) -> Result<(), Box<dyn Error>> {
        if let Some(sender) = &self.sender {
            Ok(sender.send(String::from(msg)).await?)
        } else {
            Err(Box::new(RnnError::SendingWithoutConnection))
        }
    }

    pub fn close(&mut self) {
        self.sender = None;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut publisher = Publisher::new();

    let mut subscriber = publisher.build_subscriber(10)?;

    tokio::spawn(async move {
        while let Some(msg) = subscriber.next().await {
            println!("Полученно сообщение: {}", msg);
        }
    });

    publisher.send("Hello").await?;
    publisher.send("world").await?;

    publisher.close();

    Ok(())
}
