//! Events that report greetings to other services.

use std::future::Future;

use contracts::hello::v1::Greeted;
use prost::Message;

/// Event that could not be published.
#[derive(Debug, thiserror::Error)]
#[error("event was not published: {0}")]
pub struct Undelivered(String);

impl Undelivered {
    /// Describes why the event was not published.
    #[must_use]
    pub fn new(reason: impl std::fmt::Display) -> Self {
        Self(reason.to_string())
    }
}

/// Destination of greeting events.
pub trait Events: Send + Sync + 'static {
    /// Publishes one greeting event.
    fn publish(&self, event: Greeted) -> impl Future<Output = Result<(), Undelivered>> + Send;
}

/// Adapter: publishes greeting events to a NATS subject.
#[derive(Debug, Clone)]
pub struct Nats {
    /// Connection to NATS.
    client: async_nats::Client,
    /// Subject the events go to.
    subject: String,
}

impl Nats {
    /// Creates a publisher to the given subject.
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

/// Greeting events kept in memory, for tests.
#[derive(Debug, Default)]
pub struct Memory {
    /// Events published so far.
    published: std::sync::Mutex<Vec<Greeted>>,
}

/// Events that were already published.
pub trait Journal {
    /// Returns the published events in order.
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
