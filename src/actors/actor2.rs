#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serialize};
    use tokio::join;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::{Receiver, Sender};
    use crate::actors::actor::{ActorError, ActorFuture, BUFFER_SIZE};
    use crate::actors::actor::ActorError::Shutdown;
    use crate::actors::message::Message;


    #[derive(Serialize, Deserialize, Debug)]
    struct InputA;

    #[derive(Serialize, Deserialize, Debug)]
    struct InputB;

    #[derive(Serialize, Deserialize, Debug)]
    struct OutputA;

    #[derive(Serialize, Deserialize, Debug)]
    struct OutputB;

    struct MultiActor {}

    impl MultiActor {
        async fn run(&self, rx: Receiver<Message>, tx: Sender<Message>) -> Result<(), ActorError> {
            let (rx_a, rx_b) = Self::split_receiver(rx).await;
            let (tx_a, tx_b) = Self::split_sender(tx).await;
            self.multi_actor(rx_a, rx_b, tx_a, tx_b).await
        }

        async fn split_sender(tx: Sender<Message>) -> (Sender<OutputA>, Sender<OutputB>) {
            let (tx_a, mut rx_a) = mpsc::channel(BUFFER_SIZE);
            let (tx_b, mut rx_b) = mpsc::channel(BUFFER_SIZE);

            {
                let tx = tx.clone();
                tokio::spawn(async move {
                    while let Some(msg) = rx_a.recv().await {
                        if let Some(out_msg) = Message::new_from_bytes("topic/output/a", Bytes::from("OutputA")) {
                            tx.send(out_msg).await.map_err(|_| Shutdown)?;
                        }
                    }
                    Ok::<(), ActorError>(())
                });
            }
            {
                let tx = tx.clone();
                tokio::spawn(async move {
                    while let Some(msg) = rx_b.recv().await {
                        if let Some(out_msg) = Message::new_from_bytes("topic/output/b", Bytes::from("OutputB")) {
                            tx.send(out_msg).await.map_err(|_| Shutdown)?;
                        }
                    }
                    Ok::<(), ActorError>(())
                });
            }
            (tx_a, tx_b)
        }

        async fn split_receiver(mut rx: Receiver<Message>) -> (Receiver<InputA>, Receiver<InputB>) {
            let (tx_a, rx_a) = mpsc::channel(BUFFER_SIZE);
            let (tx_b, rx_b) = mpsc::channel(BUFFER_SIZE);

            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if msg.topic() == "topic/a" {
                        tx_a.send(InputA).await.map_err(|_| Shutdown)?;
                    } else if msg.topic() == "topic/b" {
                        tx_b.send(InputB).await.map_err(|_| Shutdown)?;
                    }
                }
                Ok::<(), ActorError>(())
            });
            (rx_a, rx_b)
        }
        async fn multi_actor(
            &self,
            mut rx_a: Receiver<InputA>,
            mut rx_b: Receiver<InputB>,
            tx_a: Sender<OutputA>,
            tx_b: Sender<OutputB>,
        ) -> Result<(), ActorError> {
            let handle_a: tokio::task::JoinHandle<Result<(), ActorError>> = tokio::spawn(async move {
                while let Some(a) = rx_a.recv().await {
                    tx_a.send(OutputA).await.map_err(|_| Shutdown)?;
                }
                Ok(())
            });

            let handle_b: tokio::task::JoinHandle<Result<(), ActorError>> = tokio::spawn(async move {
                while let Some(a) = rx_b.recv().await {
                    tx_b.send(OutputB).await.map_err(|_| Shutdown)?;
                }
                Ok(())
            });

            let _ = join!(handle_a, handle_b);

            Ok(())
        }
    }

    #[tokio::test]
    async fn test_multi() {
        let (tx_in, rx_in) = mpsc::channel(BUFFER_SIZE);
        let (tx_out, mut rx_out) = mpsc::channel(BUFFER_SIZE);
        let multi_actor = MultiActor {};

        tx_in.send(Message::new_from_bytes("topic/a", Bytes::from("Hello")).unwrap()).await.unwrap();
        tx_in.send(Message::new_from_bytes("topic/b", Bytes::from("Welt")).unwrap()).await.unwrap();

        let handle = tokio::spawn(async move {
            multi_actor.run(rx_in, tx_out).await
        });

        let first = rx_out.recv().await;
        let second = rx_out.recv().await;

        println!("First: {:?}", first);
        println!("Second: {:?}", second);
    }
}