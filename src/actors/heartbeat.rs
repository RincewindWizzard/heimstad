use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::actors::actor::{Actor, ActorFuture, Payload};
use crate::actors::actor::ActorError::Shutdown;
use crate::boxed_async;

#[derive(Debug)]
pub struct Heartbeat;

const HEARTBEAT_REPR: &str = "Heartbeat";

pub struct HeartbeatEmitter {
    interval: Duration,
}

impl TryInto<Payload> for Heartbeat {
    type Error = ();

    fn try_into(self) -> Result<Payload, Self::Error> {
        Ok(Vec::from(HEARTBEAT_REPR.as_bytes()))
    }
}

impl TryFrom<Payload> for Heartbeat {
    type Error = ();

    fn try_from(data: Payload) -> Result<Self, Self::Error> {
        if data == HEARTBEAT_REPR.as_bytes() {
            return Ok(Heartbeat);
        }
        Err(())
    }
}

impl HeartbeatEmitter {
    pub fn new(interval: Duration) -> HeartbeatEmitter {
        HeartbeatEmitter {
            interval,
        }
    }
}

impl Actor<(), Heartbeat> for HeartbeatEmitter {
    fn run(&self, rx: Receiver<()>, tx: Sender<Heartbeat>) -> ActorFuture {
        boxed_async!({
            while !rx.is_closed() {
                tx.send(Heartbeat).await.map_err(|_| Shutdown)?;
                tokio::time::sleep(self.interval).await;
            }
            Ok(())
        })
    }
}