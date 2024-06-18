use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use crate::actors::actor::{Actor, ActorResult, Message};

pub struct Topic<T> {
    name: String,
    producer: Vec<Receiver<T>>,
    consumer: Vec<Sender<T>>,
}

impl<T> Topic<T> {
    fn new(name: &str) -> Topic<T> {
        Topic {
            name: name.to_string(),
            producer: vec![],
            consumer: vec![],
        }
    }

    fn subscribe(&mut self, tx: Sender<T>) {
        self.consumer.push(tx);
    }

    fn publish(&mut self, rx: Receiver<T>) {
        self.producer.push(rx);
    }
}

type MessageTopic = Topic<Box<dyn Message>>;
pub struct Broker {
    topics: HashMap<String, MessageTopic>,
    actors: Vec<ActorResult>,
}

impl Broker {
    pub fn new() -> Broker {
        Broker {
            topics: Default::default(),
            actors: vec![],
        }
    }
    pub async fn add_actor<I: std::marker::Send, O: std::marker::Send>(&mut self, actor: Box<dyn Actor<I, O>>) {
        //let (handle, tx, rx) = actor.start().await;
        //self.actors.push(handle);

    }
}


#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_broker() {
        let broker = ();
    }
}