use std::time::Instant;
use std::time::Duration;
use log::debug;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use crate::just_channels::ActorError::Shutdown;

struct Heartbeat;

#[derive(PartialEq, Clone, Copy, Debug)]
enum State {
    On,
    Off,
}

enum ActorError {
    Shutdown,
}

struct Minuterie {
    rx: Receiver<Heartbeat>,
    tx: Sender<State>,
    state: State,
    timeout: Duration,
    timeout_instant: Instant,
}

impl Minuterie {
    fn new(timeout: Duration) -> (Minuterie, Sender<Heartbeat>, Receiver<State>) {
        let (tx_heartbeat, rx_heartbeat) = mpsc::channel(16);
        let (tx_state, rx_state) = mpsc::channel(16);
        (
            Minuterie {
                rx: rx_heartbeat,
                tx: tx_state,
                state: State::Off,
                timeout,
                timeout_instant: Instant::now() + timeout,
            },
            tx_heartbeat,
            rx_state
        )
    }

    fn update_timestamp(&mut self) {
        self.timeout_instant = Instant::now() + self.timeout;
    }

    fn is_timeout(&self) -> bool {
        self.timeout_instant < Instant::now()
    }

    async fn set_state(&mut self, state: State) -> Result<(), SendError<State>> {
        let changed = self.state != state;
        self.state = state;
        if changed {
            self.tx.send(state).await?;
        }
        Ok(())
    }
    async fn run(&mut self) -> Result<(), ActorError> {
        loop {
            match tokio::time::timeout(self.timeout, self.rx.recv()).await {
                Ok(Some(_)) => {
                    self.set_state(State::On).await.map_err(|_| Shutdown)?;
                }
                Ok(None) => {
                    return Err(Shutdown);
                }
                Err(_) => {
                    self.set_state(State::Off).await.map_err(|_| Shutdown)?;
                }
            }
        }
    }
}


pub async fn main() -> Result<(), anyhow::Error> {
    let timeout = Duration::from_millis(10);
    let (mut minuterie, tx, mut rx) = Minuterie::new(timeout);

    let handle = tokio::spawn(async move {
        let _ = minuterie.run().await;
    });

    tx.send(Heartbeat).await?;

    tokio::time::sleep(timeout * 2).await;
    drop(tx);

    while let Some(state) = rx.recv().await {
        debug!("Recieved: {state:?}");
    }
    Ok(())
}