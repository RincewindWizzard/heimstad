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
            let next_state =
                match tokio::time::timeout(self.timeout, self.rx.recv()).await {
                    Ok(Some(_)) => {
                        State::On
                    }
                    Err(_) => {
                        State::Off
                    }
                    Ok(None) => {
                        return Err(Shutdown);
                    }
                };
            self.set_state(next_state).await.map_err(|_| Shutdown)?;
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use log::debug;
    use crate::just_channels::{Heartbeat, Minuterie};


    #[tokio::test]
    async fn test_timerie() -> Result<(), anyhow::Error> {
        let timeout = Duration::from_millis(10);
        let (mut minuterie, tx, mut rx) = Minuterie::new(timeout);

        let handle = tokio::spawn(async move {
            let _ = minuterie.run().await;
        });

        let heartbeats = vec![
            0,
            1,
            2,
            4,
            15,
            40,
        ];


        let begin = Instant::now();
        let tick = timeout / 10;

        let read_handle = {
            let begin = begin.clone();
            tokio::spawn(async move {
                let mut result = vec![];
                while let Some(state) = rx.recv().await {
                    let since = discrete_time(Instant::now() - begin, tick);
                    println!("Received {since:?}: {state:?}");
                    result.push((since, state));
                }
                result
            })
        };
        for instant in &heartbeats {
            let instant = begin + (*instant) * tick;
            tokio::time::sleep_until(tokio::time::Instant::from(instant)).await;
            tx.send(Heartbeat).await?;
            println!("Sent Heartbeat at {}", discrete_time(Instant::now() - begin, tick));
        }

        tokio::time::sleep(timeout * 2).await;
        drop(tx);
        let result = read_handle.await?;

        println!("Input {:?}\nOutput {:?}", heartbeats, result);

        let _ = handle.await;
        Ok(())
    }

    fn discrete_time(d: Duration, discrete: Duration) -> usize {
        (d.as_nanos() / discrete.as_nanos()) as usize
    }
}