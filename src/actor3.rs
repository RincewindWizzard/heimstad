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

pub trait TopicBytesSerde
    where Self: Sized
{
    fn from_bytes(data: &[u8]) -> Option<Self>;

    fn to_bytes(&self) -> Option<Vec<u8>>;
}

impl TopicBytesSerde for Tick {
    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data == "Tick".as_bytes() {
            return Some(Tick);
        }
        None
    }

    fn to_bytes(&self) -> Option<Vec<u8>> {
        Some(Vec::from("Tick".as_bytes()))
    }
}


type ByteReceiveResult<'a> = Pin<Box<dyn Future<Output=Option<Vec<u8>>> + 'a>>;

pub trait SerializeReceiver {
    fn recv(&mut self) -> ByteReceiveResult;
}

impl<T> SerializeReceiver for Receiver<T>
    where
        T: TopicBytesSerde,
{
    fn recv(&mut self) -> ByteReceiveResult {
        boxed_async!({
            let doc = self.recv().await?;
            doc.to_bytes()
        })
    }
}

/*
impl<'de, R, T> SerializeReceiver<'de> for Receiver<T>
    where
        T: serde::Serialize<'de>,
{
    fn recv(&mut self) -> ByteReceiveResult {
        boxed_async!({
            let doc = self.recv().await?;
            let data = doc.serialize().map_err(|| None)?;


            Some(data)
        })
    }
}

pub trait DeserializeSender<'de> {
    fn send(&self, data: &'de [u8]) -> Result<(), serde_json::Error>;
}

impl<'de, T> DeserializeSender<'de> for Sender<T>
    where
        T: serde::Deserialize<'de>,
{
    fn send(&self, data: &'de [u8]) -> Result<(), serde_json::Error> {
        let doc = serde_json::from_slice::<T>(data)?;
        Ok(())
    }
}*/

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::sync::mpsc;
    use crate::actor3::{Actor, BUFSIZE, ClockActor, SerializeReceiver, Tick, TopicBytesSerde};

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
        let data = tick.to_bytes();
        assert!(data.is_some());
        data.map(|data| assert_eq!(data, Vec::from("Tick".as_bytes())));

        let tick = Tick::from_bytes("Tick".as_bytes());
        assert!(tick.is_some());
    }

    #[tokio::test]
    async fn test_serializing_receiver() {
        let (tx, mut rx) = mpsc::channel(BUFSIZE);
        let tick = Tick;
        let _ = tx.send(tick).await;

        let data: Option<Vec<u8>> = SerializeReceiver::recv(&mut rx).await;
        assert!(data.is_some());
        data.map(|data| assert_eq!(data, Vec::from("Tick".as_bytes())));
    }
}