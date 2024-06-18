use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use anyhow::anyhow;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
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

#[derive(Debug)]
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

type Payload = Vec<u8>;


impl TryInto<Payload> for Tick {
    type Error = ();

    fn try_into(self) -> Result<Payload, Self::Error> {
        Ok(Vec::from("Tick".as_bytes()))
    }
}

impl TryFrom<Payload> for Tick {
    type Error = ();

    fn try_from(data: Payload) -> Result<Self, Self::Error> {
        if data == "Tick".as_bytes() {
            return Ok(Tick);
        }
        Err(())
    }
}


type ByteReceiveResult<'a> = Pin<Box<dyn Future<Output=Option<Vec<u8>>> + 'a>>;

pub trait SerializeReceiver {
    fn recv(&mut self) -> ByteReceiveResult;
}

impl<T> SerializeReceiver for Receiver<T>
    where
        T: TryInto<Payload>,
{
    fn recv(&mut self) -> ByteReceiveResult {
        boxed_async!({
            let doc = self.recv().await?;
            doc.try_into().ok()
        })
    }
}

async fn wrap_receiver<I, O>(mut rx: Receiver<I>) -> Receiver<O>
    where
        O: Send + std::marker::Sync + 'static,
        I: TryInto<O> + Send + 'static
{
    let (tx, mut rx_out) = mpsc::channel(BUFSIZE);

    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            tx.send(data.try_into().map_err(|_| anyhow!("Could not convert data."))?).await?;
        }
        Ok::<(), anyhow::Error>(())
    });

    rx_out
}

struct Topic<T> {
    name: String,
    producer: Receiver<T>,
    consumer: Sender<T>,
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::sync::mpsc;
    use crate::actor3::{Actor, BUFSIZE, ClockActor, SerializeReceiver, Tick, wrap_receiver};

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

    #[tokio::test]
    async fn test_broker() {
        let broker = ();
    }

    #[tokio::test]
    async fn test_serializing() {
        let tick = Tick;
        let data: Option<Vec<u8>> = tick.try_into().ok();
        assert!(data.is_some());
        data.map(|data| assert_eq!(data, Vec::from("Tick".as_bytes())));

        let tick = Tick::try_from(Vec::from("Tick".as_bytes())).ok();
        assert!(tick.is_some());
    }

    #[tokio::test]
    async fn test_wrap_receiver() {
        let (tx, mut rx) = mpsc::channel(BUFSIZE);
        let mut rx = wrap_receiver(rx).await;
        let tick = Tick;
        let _ = tx.send(tick).await;


        let data: Option<Vec<u8>> = rx.recv().await;
        assert!(data.is_some());
        data.map(|data| {
            assert_eq!(data, Vec::from("Tick".as_bytes()))
        });
    }
}