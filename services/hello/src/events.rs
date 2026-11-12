//! Events that report greetings to other services.

use std::future::Future;

use prost::Message;

/// Greeting given to someone.
pub trait Salutation {
    /// Returns the name of the greeted person.
    fn name(&self) -> &str;
    /// Returns the words of the greeting.
    fn text(&self) -> &str;
}

/// Fact that someone was greeted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Greeted {
    /// Name of the greeted person.
    name: String,
    /// Words of the greeting.
    text: String,
}

impl Greeted {
    /// Records that the name was greeted with the text.
    #[must_use]
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            text: text.into(),
        }
    }
}

impl Salutation for Greeted {
    fn name(&self) -> &str {
        &self.name
    }

    fn text(&self) -> &str {
        &self.text
    }
}

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
        let wire = contracts::hello::v1::Greeted {
            name: event.name().to_owned(),
            text: event.text().to_owned(),
        };
        self.client
            .publish(self.subject.clone(), wire.encode_to_vec().into())
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
