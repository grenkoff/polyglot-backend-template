//! Integration tests of the hello service with a real NATS server.

use std::time::Duration;

use futures::StreamExt;
use hello::events::{Events, Greeted, Nats};
use prost::Message;
use testcontainers_modules::nats::Nats as NatsImage;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

#[tokio::test]
async fn publishes_events_to_nats_as_protobuf() {
    let server = NatsImage::default().start().await.unwrap();
    let url = format!(
        "nats://{}:{}",
        server.get_host().await.unwrap(),
        server.get_host_port_ipv4(4222).await.unwrap()
    );
    let client = async_nats::connect(url).await.unwrap();
    let mut subscriber = client.subscribe("hello.v1.greeted").await.unwrap();
    client.flush().await.unwrap();

    Nats::new(client, "hello.v1.greeted")
        .publish(Greeted::new("Ann", "Hello, Ann!"))
        .await
        .unwrap();

    let message = tokio::time::timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("no event arrived within 5 seconds")
        .unwrap();
    assert_eq!(
        contracts::hello::v1::Greeted::decode(message.payload).unwrap(),
        contracts::hello::v1::Greeted {
            name: String::from("Ann"),
            text: String::from("Hello, Ann!"),
        }
    );
}
