use std::fmt::{Debug};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;



/// This represents a Message to be sent across wires.
pub trait Message: Send + Sync + std::fmt::Debug + Clone + Serialize + DeserializeOwned {}


/// Messages can be everything that has the traits that we expect from messages.
impl<T> Message for T
    where
        T: Send + Sync + Debug + Clone + Serialize + DeserializeOwned
{}


/// The sender consumes messages and sends them to another actor or to another system via network.
#[async_trait]
pub trait Sender<Msg>: Send + Sync
    where
        Msg: Message
{
    /// The type of value produced by the sender when an error occurs.
    type Error;

    /// Send a message to the receiving side.
    async fn send(&mut self, msg: &Msg) -> Result<(), Self::Error>;
}


#[async_trait]
pub trait Receiver<Msg>: Send + Sync
    where
        Msg: Message
{
    /// The type of value produced by the receiver when an error occurs.
    type Error;

    /// Send a message to the receiving side.
    async fn recv(&mut self) -> Result<Msg, Self::Error>;
}

#[async_trait]
impl<M> Sender<M> for broadcast::Sender<M>
    where M: Message
{
    type Error = tokio::sync::broadcast::error::SendError<M>;

    async fn send(&mut self, msg: &M) -> Result<(), Self::Error> {
        self.send(msg).await?;
        Ok(())
    }
}

#[async_trait]
impl<M> Receiver<M> for broadcast::Receiver<M>
    where M: Message
{
    type Error = tokio::sync::broadcast::error::RecvError;

    async fn recv(&mut self) -> Result<M, Self::Error> {
        Ok(self.recv().await?)
    }
}


#[async_trait]
pub trait Actor: Send + Sync {
    type Error: Send;

    async fn run(mut self) -> Result<(), Self::Error>;

    async fn start(self) -> JoinHandle<Pin<Box<dyn Future<Output=Result<(), Self::Error>> + Send>>>
        where
            Self: Sized + 'static
    {
        tokio::spawn(async move {
            Box::pin(self.run()) as Pin<Box<dyn Future<Output=Result<(), Self::Error>> + Send>>
        })
    }
}

pub struct SquareActor<R, S>
    where
        S: Sender<i64, Error=Error>,
        R: Receiver<i64, Error=Error>,
{
    source: R,
    sink: S,
}

#[async_trait]
impl<R, S> Actor for SquareActor<R, S>
    where
        S: Sender<i64, Error=Error>,
        R: Receiver<i64, Error=Error>,
{
    type Error = anyhow::Error;

    async fn run(mut self) -> Result<(), Self::Error> {
        loop {
            let input = self.source.recv().await?;
            let output = input * input;
            self.sink.send(&output).await?;
        }
    }
}

impl<R, S> SquareActor<R, S>
    where
        S: Sender<i64, Error=Error>,
        R: Receiver<i64, Error=Error>,
{}


/// A sample actor which sends messages a regular intervals
pub struct TimerActor<S>
    where
        S: Sender<i64, Error=Error>,
{
    /// Destination of the messages.
    sink: S,

    /// Pause between messages
    interval: Duration,
}

impl<S> TimerActor<S>
    where
        S: Sender<i64, Error=Error> + Send + Sync,
{
    pub fn new(sink: S, interval: Duration) -> Self {
        TimerActor { sink, interval }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let mut i = 0;
        loop {
            i = i + 1;
            self.sink.send(&i).await?;
            tokio::time::sleep(self.interval).await;
        }
    }
}

// Example implementation of a concrete Sender
struct ConsoleSender {
    capacity: usize,
}


#[async_trait]
impl<M: Message> Sender<M> for ConsoleSender {
    type Error = Error;

    async fn send(&mut self, msg: &M) -> Result<(), Self::Error> {
        self.capacity = if self.capacity > 0 { self.capacity - 1 } else { 0 };
        if self.capacity == 0 {
            Err(anyhow!("Closed"))
        } else {
            let json = serde_json::to_string(&msg)?;
            println!("Sent {}", json);
            Ok(())
        }
    }
}


#[cfg(test)]
mod tests {
    use log::info;
    use super::*;
    use tokio;
    use tokio::sync::broadcast;


    #[tokio::test]
    async fn test_timer_actor() -> Result<(), anyhow::Error> {
        let sender = ConsoleSender { capacity: 10 };
        let mut actor = TimerActor::new(sender, Duration::from_millis(1));

        let handle = tokio::spawn(async move {
            let result = actor.run().await;

            if let Err(e) = result {
                info!("Closed sender with {:?}", e);
            }
        });

        tokio::time::sleep(Duration::from_secs(2)).await;
        handle.await?;
        Ok(())
    }

    async fn test_broadcast() {
        let (tx, mut rx1) = broadcast::channel(16);
        let mut rx2 = tx.subscribe();

        tokio::spawn(async move {
            assert_eq!(rx1.recv().await.unwrap(), 10);
            assert_eq!(rx1.recv().await.unwrap(), 20);
        });

        tokio::spawn(async move {
            assert_eq!(rx2.recv().await.unwrap(), 10);
            assert_eq!(rx2.recv().await.unwrap(), 20);
        });

        tx.send(10).unwrap();
        tx.send(20).unwrap();
    }
}