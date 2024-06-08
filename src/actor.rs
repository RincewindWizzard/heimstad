use std::time::Duration;
use anyhow::Error;
use tokio::sync::broadcast;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use futures::sink::Sink;
use futures::SinkExt;
use futures::stream::Stream;

struct Actor<MessageIn, MessageOut> {
    input: dyn Stream<Item=MessageIn>,
    output: dyn Sink<MessageOut, Error=anyhow::Error>,
}


pub trait Runnable<R, E> {
    async fn run(&mut self) -> Result<R, E>;
}

struct TimerActor {
    output: dyn Sink<DateTime<Utc>, Error=anyhow::Error>,
}

impl Runnable<(), anyhow::Error> for TimerActor {
    async fn run(&mut self) -> Result<(), Error> {
        loop {
            self.output.send(DateTime::now()).await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        Ok(())
    }
}


#[async_trait]
pub trait MessageSocket {
    type MessageIn;

    type MessageOut;

    async fn read(&self) -> Self::MessageIn;
    async fn write(&self, msg: Self::MessageOut);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    struct SimpleMessageSocket {}

    #[async_trait]
    impl MessageSocket for SimpleMessageSocket {
        type MessageIn = &'static str;
        type MessageOut = &'static str;

        async fn read(&self) -> Self::MessageIn {
            "success"
        }

        async fn write(&self, msg: Self::MessageOut) {}
    }

    #[tokio::test]
    async fn test_message_socket() {
        let socket = SimpleMessageSocket {};
        socket.write("").await;
        let result = socket.read().await;

        assert_eq!("success", result);
    }



    #[tokio::test]
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