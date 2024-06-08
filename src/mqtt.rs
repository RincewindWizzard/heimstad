use rumqttc::{MqttOptions, AsyncClient, QoS};
use tokio::{task, time};
use std::time::Duration;
use std::error::Error;

async fn mqtt_connect() {
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

    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
    }
}