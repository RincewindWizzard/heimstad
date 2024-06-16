use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;


const BUFSIZE: usize = 1024;

pub enum ActorError {
    Shutdown,
}


pub type ActorFuture = Pin<Box<dyn Future<Output=Result<(), ActorError>> + Send>>;
pub type ActorRunMethod<I, O> = Box<dyn FnMut(Receiver<I>, Sender<O>) -> ActorFuture + Send>;

impl<I, O> StartAble<I, O> for ActorRunMethod<I, O>
    where
        I: Send + 'static,
        O: Send + 'static,
{
    fn start(mut self) -> RunningActor<I, O> {
        let (tx_in, rx_in) = mpsc::channel(BUFSIZE);
        let (tx_out, rx_out) = mpsc::channel(BUFSIZE);
        RunningActor {
            future: tokio::spawn(async move {
                self(rx_in, tx_out).await
            }),
            tx: tx_in,
            rx: rx_out,
        }
    }
}

pub trait StartAble<I, O> {
    fn start(self) -> RunningActor<I, O>;
}

pub struct RunningActor<I, O> {
    future: JoinHandle<Result<(), ActorError>>,
    pub tx: Sender<I>,
    pub rx: Receiver<O>,
}

