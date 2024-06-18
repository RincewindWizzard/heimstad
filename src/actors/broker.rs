use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::actors::actor::Message;

struct Topic<T> {
    name: String,
    producer: Receiver<T>,
    consumer: Sender<T>,
}

struct Broker {
    topics: HashMap<String, Topic<Box<dyn Message>>>,
}


#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_broker() {
        let broker = ();
    }
}