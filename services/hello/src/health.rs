use axum::Router;
use axum::routing::get;

pub trait Routes {
    fn router(&self) -> Router;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Health;

impl Routes for Health {
    fn router(&self) -> Router {
        Router::new().route("/health", get(|| async { "ok" }))
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::{Health, Routes};

    #[tokio::test]
    async fn answers_ok() {
        let response = Health
            .router()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
