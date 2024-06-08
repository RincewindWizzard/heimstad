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

    async fn send(&mut self, msg: &Item) -> Result<(), Self::Error>;
}

// Define the TimerActor with a generic Sender
struct TimerActor<S>
    where
        S: Sender<i64, Error=Error>,
{
    sink: S,
}

impl<S> TimerActor<S>
    where
        S: Sender<i64, Error=Error> + Send + Sync,
{
    pub fn new(sink: S) -> Self {
        TimerActor { sink }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let mut i = 0;
        loop {
            i = i + 1;
            self.sink.send(&i).await?;
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
}

// Example implementation of a concrete Sender
struct ConsoleSender {
    capacity: usize,
}


impl<T> Message for T
    where
        T: Display + Send + Sync + Debug + Clone
{}


#[async_trait]
impl<M: Message> Sender<M> for ConsoleSender {
    type Error = Error;

    async fn send(&mut self, msg: &M) -> Result<(), Self::Error> {
        self.capacity = if self.capacity > 0 { self.capacity - 1 } else { 0 };
        if self.capacity == 0 {
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
    use log::info;
    use super::*;
    use tokio;
    use tokio::sync::broadcast;


    #[tokio::test]
    async fn test_timer_actor() -> Result<(), anyhow::Error> {
        let sender = ConsoleSender { capacity: 10 };
        let mut actor = TimerActor::new(sender);

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
}