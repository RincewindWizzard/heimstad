use std::sync::Arc;
use rumqttc::{MqttOptions, AsyncClient, QoS, EventLoop};
use tokio::{task, time};
use std::time::Duration;
use actix::{Actor, AsyncContext, Context, Handler, Message, WrapFuture};

struct MqttLoop {
    event_loop: Arc<EventLoop>,
}

impl Actor for MqttLoop {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut event_loop = self.event_loop.clone();
        ctx.spawn(async move {
            //while let Ok(notification) = event_loop.poll().await {
            //  println!("Received = {:?}", notification);
            // }
        }.into_actor(self));
    }
}


pub async fn mqtt_connect() {
    let mut mqttoptions = MqttOptions::new("heimstad", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("heimstad/sub", QoS::AtMostOnce).await.unwrap();

    task::spawn(async move {
        for i in 0..10 {
            client.publish("heimstad/pub", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();
            time::sleep(Duration::from_millis(100)).await;
        }
    });
}