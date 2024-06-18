use std::fmt::{Debug, Formatter, Write};
use bytes::Bytes;


pub trait Payload {
    fn from_bytes(value: Bytes) -> Option<Box<Self>>
        where
            Self: Sized;
    fn as_bytes(&self) -> Bytes;
}


type PayloadType = Box<dyn Payload>;

pub struct Message {
    topic: String,
    payload: PayloadType,
}

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let payload = self.payload.as_bytes();
        if let Ok(payload) = String::from_utf8(Vec::from(payload)) {
            f.write_str(&format!("Message(topic=\"{}\", payload={:?})", self.topic, payload))
        } else {
            f.write_str(&format!("Message(topic=\"{}\", payload={:?})", self.topic, self.payload.as_bytes()))
        }
    }
}

impl Message {
    pub fn new<T: Payload + 'static>(topic: &str, payload: T) -> Message {
        Message {
            topic: topic.to_string(),
            payload: Box::new(payload),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actors::message::{Bytes, Message, Payload};

    struct FooPayload {
        foo: String,
    }

    impl Payload for FooPayload {
        fn from_bytes(value: Bytes) -> Option<Box<Self>> where Self: Sized {
            todo!()
        }


        fn as_bytes(&self) -> Bytes {
            Bytes::from(self.foo.clone().into_bytes())
        }
    }

    #[tokio::test]
    async fn test_message() {
        let msg = Message::new("topic/foo", FooPayload { foo: "Hello World!".to_string() });
        println!("Message: {msg:?}");
    }
}