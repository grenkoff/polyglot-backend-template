use hello::endpoint::Greetings;
use hello::events::Nats;
use hello::health::{Health, Routes};
use hello::proto::hello_service_server::HelloServiceServer;
use hello::setting::{Env, Setting};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            Env::new("RUST_LOG", "info").value(),
        ))
        .init();
    let broker = async_nats::connect(Env::new("NATS_URL", "nats://localhost:4222").value()).await?;
    let grpc = Env::new("HELLO_GRPC_ADDR", "0.0.0.0:50051")
        .value()
        .parse()?;
    let http =
        tokio::net::TcpListener::bind(Env::new("HELLO_HTTP_ADDR", "0.0.0.0:8080").value()).await?;
    tracing::info!(%grpc, http = %http.local_addr()?, "hello is starting");
    let stop = || async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "cannot listen for the shutdown signal");
        }
    };
    tokio::try_join!(
        async {
            tonic::transport::Server::builder()
                .add_service(HelloServiceServer::new(Greetings::new(Nats::new(
                    broker,
                    "hello.v1.greeted",
                ))))
                .serve_with_shutdown(grpc, stop())
                .await
                .map_err(anyhow::Error::from)
        },
        async {
            axum::serve(http, Health.router())
                .with_graceful_shutdown(stop())
                .await
                .map_err(anyhow::Error::from)
        },
    )?;
    Ok(())
}
