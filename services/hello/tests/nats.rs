use std::time::Duration;

use contracts::hello::v1::Greeted;
use futures::StreamExt;
use hello::events::{Events, Nats};
use prost::Message;
use testcontainers_modules::nats::Nats as NatsImage;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

#[tokio::test]
async fn publishes_events_to_nats() {
    let server = NatsImage::default().start().await.unwrap();
    let url = format!(
        "nats://{}:{}",
        server.get_host().await.unwrap(),
        server.get_host_port_ipv4(4222).await.unwrap()
    );
    let client = async_nats::connect(url).await.unwrap();
    let mut subscriber = client.subscribe("hello.v1.greeted").await.unwrap();
    client.flush().await.unwrap();
    let event = Greeted {
        name: String::from("Ann"),
        text: String::from("Hello, Ann!"),
    };

    Nats::new(client, "hello.v1.greeted")
        .publish(event.clone())
        .await
        .unwrap();

    let message = tokio::time::timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("no event arrived within 5 seconds")
        .unwrap();
    assert_eq!(Greeted::decode(message.payload).unwrap(), event);
}
