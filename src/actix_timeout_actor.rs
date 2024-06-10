use std::time::{Duration, Instant};
use actix::{Actor, Context, Handler, Message, Recipient};


#[derive(Message, PartialEq)]
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
        let changed = self.state != msg;
        self.set_state(msg);
        if changed {}
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
    use actix::{Actor, Arbiter, Context, Handler, Recipient, System};
    use tokio::time::timeout;
    use crate::actix_timeout_actor::{Minuterie, State};
    use crate::actix_timeout_actor::State::{Off, On};
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::error::SendError;
    use tokio::sync::mpsc::Sender;


    #[actix_rt::test]
    async fn test_actix() -> Result<(), anyhow::Error> {
        let timeout = Duration::from_millis(10);
        let (tx, rx) = mpsc::channel::<State>(20);
        let mut minuterie = Minuterie::new(timeout);
        let channel_wrapper = ChannelWrapperActor::from(tx).start();
        minuterie.subscribe(channel_wrapper.recipient());
        let minuterie = minuterie.start();

        let sequence = vec![
            (2, On),
            (4, Off),
        ];

        let start = Instant::now();
        for (tick, state) in sequence {
            let until = start + timeout * tick;
            tokio::time::sleep_until(until.into());
            minuterie.send(state).await?;
        }
        Ok(())
    }
}