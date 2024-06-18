use std::fmt::{Debug, Formatter, Write};
use std::io::Read;
use bytes::Bytes;
use serde::{Deserialize, Serialize};


pub trait Payload {
    fn try_from_bytes(value: Bytes) -> Option<Self> where Self: Sized;
    fn try_into_bytes(self) -> Option<Bytes>;
}

pub struct Message {
    topic: String,
    payload: Bytes,
}

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let payload = Bytes::try_from(self.payload.clone()).ok();
        if let Some(payload) = payload {
            if let Some(payload) = String::from_utf8(Vec::from(payload)).ok() {
                return f.write_str(&format!("Message(topic=\"{}\", payload={:?})", self.topic, payload));
            }
        }
        f.write_str(&format!("Message(topic=\"{}\", payload={:?})", self.topic, self.payload.bytes()))
    }
}

impl Message {
    pub fn new<T>(topic: &str, payload: T) -> Option<Message>
        where
            T: Payload
    {
        let payload = payload.try_into_bytes()?;
        Self::new_from_bytes(topic, payload)
    }

    pub fn new_from_bytes(topic: &str, payload: Bytes) -> Option<Message>
    {
        Some(Message {
            topic: topic.to_string(),
            payload,
        })
    }
    pub fn get_payload<T>(&self) -> Option<T>
        where
            T: Payload
    {
        Payload::try_from_bytes(self.payload.clone())
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }
}

impl<T> Payload for T
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone
{
    fn try_from_bytes(value: Bytes) -> Option<Self> where Self: Sized {
        let payload = String::from_utf8(Vec::from(value)).ok()?;
        let value = {
            let value: Self = serde_json::from_str(&payload).ok()?;
            value.clone()
        };
        Some(value)
    }

    fn try_into_bytes(self) -> Option<Bytes> {
        let data = serde_json::to_string(&self).ok()?;
        Some(Bytes::from(data))
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use crate::actors::message::{Bytes, Message, Payload};

    #[derive(Debug)]
    struct FooPayload {
        foo: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct JsonPayload {
        foo: String,
        bar: i64,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct JsonPayload2 {
        foo2: String,
        bar2: i64,
    }

    impl Payload for FooPayload {
        fn try_from_bytes(value: Bytes) -> Option<FooPayload> {
            Some(FooPayload {
                foo: String::from_utf8(Vec::from(value)).ok()?,
            })
        }

        fn try_into_bytes(self) -> Option<Bytes> {
            Some(Bytes::from(self.foo))
        }
    }


    #[tokio::test]
    async fn test_message() {
        let msg = Message::new("topic/foo", FooPayload { foo: "Hello World!".to_string() }).unwrap();
        println!("Message: {msg:?}");

        let payload: Option<FooPayload> = msg.get_payload();
        assert!(payload.is_some());
    }

    #[tokio::test]
    async fn test_message_json() {
        let msg = Message::new("topic/foo", JsonPayload { foo: "Hallo".to_string(), bar: 42 }).unwrap();
        println!("Message: {msg:?}");

        let payload: Option<JsonPayload> = msg.get_payload();
        assert!(payload.is_some());
        if let Some(payload) = payload {
            println!("Payload: {:?}", payload);
            assert_eq!("JsonPayload { foo: \"Hallo\", bar: 42 }", format!("{:?}", payload));
        }

        let payload: Option<JsonPayload2> = msg.get_payload();
        assert!(payload.is_none());
    }
}