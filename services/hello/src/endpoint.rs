//! gRPC endpoint of the hello service.

use contracts::hello::v1::hello_service_server::HelloService;
use contracts::hello::v1::{GreetRequest, GreetResponse};
use tonic::{Request, Response, Status};

use crate::domain::events::{Events, Greeted};
use crate::domain::greeting::{Greeting, Plain, Polite};
use crate::domain::name::{BlankName, Name, Raw, Trimmed};

/// Greets over gRPC and publishes a fact about every greeting.
#[derive(Debug)]
pub struct Greetings<E> {
    /// Where greeting events are published.
    events: E,
}

impl<E: Events> Greetings<E> {
    /// Creates the endpoint that publishes to the given events.
    #[must_use]
    pub const fn new(events: E) -> Self {
        Self { events }
    }
}

#[tonic::async_trait]
impl<E: Events> HelloService for Greetings<E> {
    async fn greet(
        &self,
        request: Request<GreetRequest>,
    ) -> Result<Response<GreetResponse>, Status> {
        let message = request.into_inner();
        let invalid = |error: BlankName| Status::invalid_argument(error.to_string());
        let name = Trimmed::new(Raw::new(message.name));
        let who = name.text().map_err(invalid)?;
        let plain = Plain::new(name);
        let text = if message.polite {
            Polite::new(plain).text()
        } else {
            plain.text()
        }
        .map_err(invalid)?;
        self.events
            .publish(Greeted::new(who, text.clone()))
            .await
            .map_err(|error| Status::unavailable(error.to_string()))?;
        Ok(Response::new(GreetResponse { text }))
    }
}

#[cfg(test)]
mod tests {
    use tonic::{Code, Request};

    use super::Greetings;
    use crate::domain::events::{Events, Greeted, Journal, Memory, Undelivered};
    use contracts::hello::v1::GreetRequest;
    use contracts::hello::v1::hello_service_server::HelloService;

    #[tokio::test]
    async fn greets_and_reports() {
        let endpoint = Greetings::new(Memory::default());
        let response = endpoint
            .greet(Request::new(GreetRequest {
                name: String::from(" Ann "),
                polite: true,
            }))
            .await
            .unwrap();
        assert_eq!(response.into_inner().text, "Hello, Ann! Nice to meet you.");
        assert_eq!(
            endpoint.events.entries(),
            vec![Greeted::new("Ann", "Hello, Ann! Nice to meet you.")]
        );
    }

    #[tokio::test]
    async fn refuses_blank_names() {
        let endpoint = Greetings::new(Memory::default());
        let status = endpoint
            .greet(Request::new(GreetRequest {
                name: String::from("  "),
                polite: false,
            }))
            .await
            .unwrap_err();
        assert_eq!(status.code(), Code::InvalidArgument);
        assert_eq!(endpoint.events.entries(), Vec::new());
    }

    struct Broken;

    impl Events for Broken {
        fn publish(&self, _event: Greeted) -> impl Future<Output = Result<(), Undelivered>> + Send {
            std::future::ready(Err(Undelivered::new("broker is down")))
        }
    }

    #[tokio::test]
    async fn reports_unavailable_broker() {
        let status = Greetings::new(Broken)
            .greet(Request::new(GreetRequest {
                name: String::from("Ann"),
                polite: false,
            }))
            .await
            .unwrap_err();
        assert_eq!(status.code(), Code::Unavailable);
    }
}
