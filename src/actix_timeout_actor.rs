use std::fmt::Display;
use std::time::{Duration, Instant};
use actix::{Actor, Context, Handler, Message, Recipient};
use log::debug;


#[derive(Message, PartialEq, Debug, Clone, Copy)]
#[rtype(result = "()")]
enum State {
    On,
    Off,
}


struct Minuterie {
    state: State,
    timeout_duration: Duration,
    timeout: Instant,
    recipients: Vec<Recipient<State>>,
}

impl Actor for Minuterie {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Minuterie started!");
    }
}

impl Minuterie {
    fn new(timeout: Duration) -> Minuterie {
        Minuterie {
            state: State::Off,
            timeout_duration: timeout,
            timeout: Instant::now() + timeout,
            recipients: vec![],
        }
    }

    fn subscribe(&mut self, recipient: Recipient<State>) {
        self.recipients.push(recipient);
    }
    fn is_timeout(&self) -> bool {
        Instant::now() > self.timeout
    }

    fn update_timeout(&mut self) {
        self.timeout = Instant::now() + self.timeout_duration;
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
        self.update_timeout();
    }
}

impl Handler<State> for Minuterie {
    type Result = ();

    fn handle(&mut self, msg: State, ctx: &mut Self::Context) -> Self::Result {
        debug!("Minuterie got {msg:?}");
        let changed = self.state != msg;
        self.set_state(msg);
        if changed {
            for recipient in &self.recipients {
                recipient.do_send(msg);
            }
        }
    }
}

mod util {
    use actix::{Actor, Context, Handler, ResponseFuture};
    use tokio::sync::mpsc::Sender;

    pub struct ChannelWrapperActor<T> {
        sink: Sender<T>,
    }

    impl<T: 'static> Actor for ChannelWrapperActor<T> {
        type Context = Context<Self>;
    }

    impl<T> Handler<T> for ChannelWrapperActor<T>
        where T: actix::Message<Result=()> + 'static
    {
        type Result = ResponseFuture<()>;

        fn handle(&mut self, msg: T, ctx: &mut Self::Context) -> Self::Result {
            let sink = self.sink.clone();
            Box::pin(async move {
                let _ = sink.send(msg).await;
            })
        }
    }

    impl<T> From<Sender<T>> for ChannelWrapperActor<T> {
        fn from(value: Sender<T>) -> Self {
            ChannelWrapperActor {
                sink: value,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actix_timeout_actor::util::ChannelWrapperActor;
    use std::time::{Duration, Instant};
    use actix::{Actor};
    use log::Level;

    use crate::actix_timeout_actor::{Minuterie, State};
    use crate::actix_timeout_actor::State::{Off, On};
    use tokio::sync::mpsc;
    use tokio::time::timeout;

    fn setup_logging() {
        stderrlog::new()
            .module(module_path!())
            .quiet(false)
            .verbosity(Level::Debug) // show warnings and above
            .timestamp(stderrlog::Timestamp::Millisecond)
            .init()
            .expect("Could not setup logging!");
    }

    #[actix_rt::test]
    async fn test_minuterie() -> Result<(), anyhow::Error> {
        setup_logging();
        let timeout = Duration::from_millis(10);
        let (tx, mut rx) = mpsc::channel::<State>(20);
        let mut minuterie = Minuterie::new(timeout);
        let channel_wrapper = ChannelWrapperActor::from(tx).start();
        minuterie.subscribe(channel_wrapper.recipient());
        let minuterie = minuterie.start();

        let sequence = vec![
            (2, On),
            (4, Off),
            (10, On),
            (20, On),
            (25, On),
        ];

        let start = Instant::now();
        for (tick, state) in sequence {
            let until = start + timeout * tick;
            tokio::time::sleep_until(until.into()).await;
            minuterie.do_send(state);
        }
        tokio::time::sleep(timeout * 2).await;


        while let Ok(Some(msg)) = tokio::time::timeout(timeout, rx.recv()).await {
            println!("Got: {:?}", msg);
        }
        Ok(())
    }
}