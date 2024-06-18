use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc::{Receiver, Sender};
use anyhow::anyhow;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use crate::boxed_async;


const BUFFER_SIZE: usize = 1024;

pub type ActorFuture<'a> = Pin<Box<dyn Future<Output=Result<(), ActorError>> + Send + 'a>>;
pub type Payload = Vec<u8>;

pub enum ActorError {
    Shutdown,
}

pub trait Message {}

impl<T> Message for T where T: TryInto<Payload> + TryFrom<Payload> {}

pub type ActorResult = JoinHandle<Result<(), ActorError>>;

pub trait Actor<I, O>
    where
        I: Send + 'static,
        O: Send + 'static, Self: 'static
{
    fn start(self) -> Pin<Box<dyn Future<Output=(ActorResult, Sender<I>, Receiver<O>)> + Send>>
        where
            Self: Sized + Send,
    {
        let (tx_in, rx_in) = mpsc::channel(BUFFER_SIZE);
        let (tx_out, rx_out) = mpsc::channel(BUFFER_SIZE);
        boxed_async!({
            let handle = tokio::spawn(async move {
                self.run(rx_in, tx_out).await
            });
            (handle, tx_in, rx_out)
        })
    }
    fn run(&self, rx: Receiver<I>, tx: Sender<O>) -> ActorFuture;
}


pub async fn wrap_receiver<I, O>(mut rx: Receiver<I>) -> Receiver<O>
    where
        O: Send + std::marker::Sync + 'static,
        I: TryInto<O> + Send + 'static
{
    let (tx, rx_out) = mpsc::channel(BUFFER_SIZE);

    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            tx.send(data.try_into().map_err(|_| anyhow!("Could not convert data."))?).await?;
        }
        Ok::<(), anyhow::Error>(())
    });

    rx_out
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::sync::mpsc;
    use crate::actors::actor::{Actor, BUFFER_SIZE, wrap_receiver};
    use crate::actors::heartbeat::{HeartbeatEmitter, Heartbeat};

    #[tokio::test]
    async fn test_some_higher_order_stuff() {
        let interval = Duration::from_millis(50);
        let times: u32 = 50;
        let heartbeat_emitter = HeartbeatEmitter::new(interval);
        let (_, tx, mut rx) = heartbeat_emitter.start().await;

        tokio::time::sleep(interval * times).await;
        drop(tx);
        let mut i: i64 = 0;
        while let Some(_) = rx.recv().await {
            i += 1;
        }
        println!("Got {i} Ticks!");
        assert!(i.abs_diff(times as i64) <= 1);
    }

    #[tokio::test]
    async fn test_serializing() {
        let tick = Heartbeat;
        let data: Option<Vec<u8>> = tick.try_into().ok();
        assert!(data.is_some());
        data.map(|data| assert_eq!(data, Vec::from("Heartbeat".as_bytes())));

        let tick = Heartbeat::try_from(Vec::from("Heartbeat".as_bytes())).ok();
        assert!(tick.is_some());
    }

    #[tokio::test]
    async fn test_wrap_receiver() {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);
        let mut rx = wrap_receiver(rx).await;
        let tick = Heartbeat;
        let _ = tx.send(tick).await;


        let data: Option<Vec<u8>> = rx.recv().await;
        assert!(data.is_some());
        data.map(|data| {
            assert_eq!(data, Vec::from("Heartbeat".as_bytes()))
        });
    }
}