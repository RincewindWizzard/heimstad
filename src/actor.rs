use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use chrono::{DateTime, Utc};


pub trait Message: Display + Send + Sync + std::fmt::Debug + Clone {}

#[async_trait]
pub trait Sender<Item>: Send + Sync
    where
        Item: Message
{
    /// The type of value produced by the sink when an error occurs.
    type Error;

    async fn send(&self, msg: &Item) -> Result<(), Self::Error>;
}

// Define the TimerActor with a generic Sender
struct TimerActor<S>
    where
        S: Sender<i64, Error=Error>,
{
    output: Arc<S>,
}

impl<S> TimerActor<S>
    where
        S: Sender<i64, Error=Error> + Send + Sync,
{
    pub fn new(output: Arc<S>) -> Self {
        TimerActor { output }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let i = 0;
        loop {
            let i = i + 1;
            self.output.send(&i).await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

// Example implementation of a concrete Sender
struct ConsoleSender {
    closed: bool,
}


impl<T> Message for T
    where
        T: Display + Send + Sync + Debug + Clone
{}


#[async_trait]
impl<M: Message> Sender<M> for ConsoleSender {
    type Error = Error;

    async fn send(&self, msg: &M) -> Result<(), Self::Error> {
        if self.closed {
            Err(anyhow!("Closed"))
        } else {
            println!("Sent {}", msg);
            Ok(())
        }
    }
}


#[cfg(test)]
mod tests {
    use futures::AsyncWriteExt;
    use super::*;
    use tokio;
    use tokio::sync::broadcast;


    #[tokio::test]
    async fn test_timer_actor() -> Result<(), anyhow::Error> {
        let sender = Arc::new(ConsoleSender { closed: false });
        let actor = TimerActor::new(sender.clone());

        let handle = tokio::spawn(async move {
            actor.run().await.expect("Foo")
        });

        tokio::time::sleep(Duration::from_secs(2)).await;
        handle.await?;
        Ok(())
    }
}