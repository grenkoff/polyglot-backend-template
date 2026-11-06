pub mod endpoint;
pub mod events;
pub mod greeting;
pub mod health;
pub mod name;
pub mod setting;

#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    reason = "generated code"
)]
pub mod proto {
    tonic::include_proto!("hello.v1");
}
