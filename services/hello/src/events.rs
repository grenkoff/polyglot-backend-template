//! Greeting events published to NATS.

use prost::Message;

use crate::domain::events::{Events, Greeted, Salutation, Undelivered};

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
