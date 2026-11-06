use std::future::Future;

use prost::Message;

use crate::proto::Greeted;

#[derive(Debug, thiserror::Error)]
#[error("event was not published: {0}")]
pub struct Undelivered(String);

impl Undelivered {
    #[must_use]
    pub fn new(reason: impl std::fmt::Display) -> Self {
        Self(reason.to_string())
    }
}

pub trait Events: Send + Sync + 'static {
    fn publish(&self, event: Greeted) -> impl Future<Output = Result<(), Undelivered>> + Send;
}

#[derive(Debug, Clone)]
pub struct Nats {
    client: async_nats::Client,
    subject: String,
}

impl Nats {
    #[must_use]
    pub fn new(client: async_nats::Client, subject: impl Into<String>) -> Self {
        Self {
            client,
            subject: subject.into(),
        }
    }
}

impl Events for Nats {
    async fn publish(&self, event: Greeted) -> Result<(), Undelivered> {
        self.client
            .publish(self.subject.clone(), event.encode_to_vec().into())
            .await
            .map_err(Undelivered::new)
    }
}

#[derive(Debug, Default)]
pub struct Memory {
    published: std::sync::Mutex<Vec<Greeted>>,
}

pub trait Journal {
    fn entries(&self) -> Vec<Greeted>;
}

impl Journal for Memory {
    fn entries(&self) -> Vec<Greeted> {
        self.published
            .lock()
            .map(|entries| entries.clone())
            .unwrap_or_default()
    }
}

impl Events for Memory {
    fn publish(&self, event: Greeted) -> impl Future<Output = Result<(), Undelivered>> + Send {
        std::future::ready(
            self.published
                .lock()
                .map(|mut entries| entries.push(event))
                .map_err(Undelivered::new),
        )
    }
}
