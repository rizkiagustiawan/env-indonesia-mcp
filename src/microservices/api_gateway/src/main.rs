use axum::{
    routing::get,
    Router,
    response::Json,
};
use serde::Serialize;
use std::net::SocketAddr;
use tracing_subscriber;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    architecture: &'static str,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "stac-geospatial-lake",
        architecture: "axum-grpc-hybrid",
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
