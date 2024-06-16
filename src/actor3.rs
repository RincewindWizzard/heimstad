use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use crate::actor3::ActorError::Shutdown;


const BUFSIZE: usize = 1024;

pub type ActorFuture<'a> = Pin<Box<dyn Future<Output=Result<(), ActorError>> + Send + 'a>>;


pub enum ActorError {
    Shutdown,
}

pub trait Actor<I, O>
    where
        I: Send + 'static,
        O: Send + 'static, Self: 'static
{
    fn start(self) -> Pin<Box<dyn Future<Output=(JoinHandle<Result<(), ActorError>>, Sender<I>, Receiver<O>)> + Send>>
        where
            Self: Sized + Send,
    {
        let (tx_in, rx_in) = mpsc::channel(BUFSIZE);
        let (tx_out, rx_out) = mpsc::channel(BUFSIZE);
        Box::pin(async move {
            let handle = tokio::spawn(async move {
                self.run(rx_in, tx_out).await
            });
            (handle, tx_in, rx_out)
        })
    }
    fn run(&self, rx: Receiver<I>, tx: Sender<O>) -> ActorFuture;
}

struct Heartbeat;

struct HeartbeatEmitter {
    interval: Duration,
}

impl HeartbeatEmitter {
    fn new(interval: Duration) -> HeartbeatEmitter {
        HeartbeatEmitter {
            interval,
        }
    }
}

impl Actor<(), Heartbeat> for HeartbeatEmitter {
    fn run(&self, rx: Receiver<()>, tx: Sender<Heartbeat>) -> ActorFuture {
        Box::pin(async move {
            while !rx.is_closed() {
                tx.send(Heartbeat).await.map_err(|_| Shutdown)?;
                tokio::time::sleep(self.interval).await;
            }
            Ok(())
        })
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::actor3::{Actor, HeartbeatEmitter};

    #[tokio::test]
    async fn test_some_higher_order_stuff() {
        let interval = Duration::from_millis(10);
        let heartbeat_emitter = HeartbeatEmitter::new(interval);
        let (handle, tx, mut rx) = heartbeat_emitter.start().await;

        tokio::time::sleep(interval * 10).await;
        drop(tx);
        while let Some(_) = rx.recv().await {
            println!("Got Heartbeat!");
        }
    }
}