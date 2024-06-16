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

macro_rules! boxed_async {
    ($body:expr) => {
        Box::pin(async move { $body })
    };
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
        boxed_async!({
            let handle = tokio::spawn(async move {
                self.run(rx_in, tx_out).await
            });
            (handle, tx_in, rx_out)
        })
    }
    fn run(&self, rx: Receiver<I>, tx: Sender<O>) -> ActorFuture;
}

struct Tick;

struct ClockActor {
    interval: Duration,
}

impl ClockActor {
    fn new(interval: Duration) -> ClockActor {
        ClockActor {
            interval,
        }
    }
}

impl Actor<(), Tick> for ClockActor {
    fn run(&self, rx: Receiver<()>, tx: Sender<Tick>) -> ActorFuture {
        boxed_async!({
            while !rx.is_closed() {
                tx.send(Tick).await.map_err(|_| Shutdown)?;
                tokio::time::sleep(self.interval).await;
            }
            Ok(())
        })
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::actor3::{Actor, ClockActor};

    #[tokio::test]
    async fn test_some_higher_order_stuff() {
        let interval = Duration::from_millis(50);
        let times: u32 = 50;
        let heartbeat_emitter = ClockActor::new(interval);
        let (handle, tx, mut rx) = heartbeat_emitter.start().await;

        tokio::time::sleep(interval * times).await;
        drop(tx);
        let mut i: i64 = 0;
        while let Some(_) = rx.recv().await {
            i += 1;
        }
        println!("Got {i} Ticks!");
        assert!(i.abs_diff(times as i64) <= 1);
    }
}