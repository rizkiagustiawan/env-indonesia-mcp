mod ai_client;

use axum::{
    routing::get,
    Router,
    response::Json,
};
use serde::Serialize;
use std::net::SocketAddr;
use tracing_subscriber;
use ai_client::ai_inference::inference_engine_client::InferenceEngineClient;
use ai_client::ai_inference::FloodRequest;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    architecture: &'static str,
}

#[derive(Serialize)]
struct InferenceResponse {
    status: &'static str,
    inference_ms: f32,
    predicted_depth_sample: f32,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "stac-geospatial-lake",
        architecture: "axum-grpc-hybrid",
    })
}

async fn test_inference() -> Json<InferenceResponse> {
    // Connect to Python gRPC server
    let mut client = InferenceEngineClient::connect("http://127.0.0.1:50051").await.unwrap();
    
    let request = tonic::Request::new(FloodRequest {
        site_id: "sumbawa_test".to_string(),
        bbox: vec![117.0, -8.5, 118.0, -9.0],
        initial_h: vec![1.0, 1.2, 1.1, 1.0], // 2x2 grid
        width: 2,
        height: 2,
        t_end: 3600.0,
    });

    let response = client.predict_flood_fno(request).await.unwrap().into_inner();
    
    Json(InferenceResponse {
        status: "success",
        inference_ms: response.inference_ms,
        predicted_depth_sample: response.predicted_h[0],
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/test_inference", get(test_inference));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
