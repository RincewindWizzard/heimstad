use std::sync::Arc;
use async_trait::async_trait;

enum MailboxError {
    Unknown
}

#[async_trait]
pub trait Actor {
    async fn started(&self, ctx: &mut Context) {}

    async fn handle(&self, msg: Message, ctx: &mut Context) -> Result<(), MailboxError> {}
}

#[derive(Clone)]
struct Context {
    system: Arc<ActorSystem>,
}

impl Context {
    fn from(system: ActorSystem) -> Context {
        Context {
            system: Arc::from(system)
        }
    }
}

struct ActorSystem {
    actors: Vec<Box<dyn Actor>>,
}

#[derive(Clone)]
struct Message {
    payload: i64,
}


impl ActorSystem {
    fn new() -> ActorSystem {
        ActorSystem {
            actors: vec![]
        }
    }

    async fn push(&mut self, actor: Box<dyn Actor>) {
        let actor_ref = &actor;
        self.actors.push(actor);
        let ctx = Arc::new(self);
        actor_ref.started(self).await;
    }

    async fn send_message(&mut self, message: Message) {
        for actor in self.actors {
            let _ = actor.handle(message.clone(), Arc::new(self)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use async_trait::async_trait;
    use tokio;
    use crate::actor2::{Actor, ActorSystem, Context, MailboxError, Message};

    struct Emitter {}

    struct Mapper {}


    struct Consumer {}

    #[async_trait]
    impl Actor for Emitter {
        async fn started(&self, ctx: &mut Context) {
            tokio::spawn(async move {
                for i in 0.. {
                    ctx.send_message(Message { payload: 0 }).await;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            });
        }
    }

    #[async_trait]
    impl Actor for Mapper {
        async fn handle(&self, msg: Message, ctx: &mut Context) -> Result<(), MailboxError> {
            ctx.send_message(Message { payload: msg.payload * 2 }).await;
            Ok(())
        }
    }

    #[async_trait]
    impl Actor for Consumer {
        async fn handle(&self, msg: Message, ctx: &mut Context) -> Result<(), MailboxError> {
            println!("Got: {}", msg.payload);
            Ok(())
        }
    }


    #[tokio::test]
    async fn test_timer_actor() -> Result<(), anyhow::Error> {
        let mut system = ActorSystem::new();
        let emitter = Emitter {};
        let mapper = Mapper {};
        let consumer = Consumer {};

        system.push(Box::from(emitter)).await;
        system.push(Box::from(mapper)).await;
        system.push(Box::from(consumer)).await;

        Ok(())
    }
}