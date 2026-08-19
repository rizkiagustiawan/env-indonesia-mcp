mod ai_client;

use axum::{
    routing::{get, post},
    Router,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing_subscriber;
use ai_client::ai_inference::inference_engine_client::InferenceEngineClient;
use ai_client::ai_inference::FloodRequest;
use ai_client::ai_inference::ShallowWaterRequest;

#[derive(Deserialize)]
struct SweInferenceRequest {
    site_id: String,
    bbox: Vec<f32>,
    initial_h: Vec<f32>,
    dem: Vec<f32>,
    width: usize,
    height: usize,
    t_end: f32,
}

#[derive(Serialize)]
struct SweInferenceResponse {
    status: &'static str,
    inference_ms: f32,
    predicted_h: Vec<f32>,
}

async fn inference_swe(axum::extract::Json(payload): axum::extract::Json<SweInferenceRequest>) -> Json<SweInferenceResponse> {
    let client_res = InferenceEngineClient::connect("http://127.0.0.1:50051").await;
    if client_res.is_err() {
        return Json(SweInferenceResponse { status: "error_grpc_connection", inference_ms: 0.0, predicted_h: vec![] });
    }
    let mut client = client_res.unwrap();
    
    let request = tonic::Request::new(FloodRequest {
        site_id: payload.site_id,
        bbox: payload.bbox,
        initial_h: payload.initial_h,
        dem: payload.dem,
        width: payload.width as i32,
        height: payload.height as i32,
        t_end: payload.t_end,
    });

    match client.predict_flood_fno(request).await {
        Ok(resp) => {
            let response = resp.into_inner();
            Json(SweInferenceResponse {
                status: "success",
                inference_ms: response.inference_ms,
                predicted_h: response.predicted_h,
            })
        }
        Err(_) => Json(SweInferenceResponse { status: "error_inference", inference_ms: 0.0, predicted_h: vec![] })
    }
}

#[derive(Serialize)]
struct ShallowWaterInferenceResponse {
    status: &'static str,
    inference_ms: f32,
    predicted_h_sample: f32,
    predicted_u_sample: f32,
    predicted_v_sample: f32,
}

async fn test_pino_inference() -> Json<ShallowWaterInferenceResponse> {
    // Connect to Python gRPC server
    let mut client = InferenceEngineClient::connect("http://127.0.0.1:50051").await.unwrap();
    
    // Simulate Banjarmasin Grid (10x10 for testing)
    let grid_size = 10 * 10;
    
    let request = tonic::Request::new(ShallowWaterRequest {
        site_id: "banjarmasin_test".to_string(),
        bbox: vec![114.49, -3.40, 114.69, -3.20],
        initial_h: vec![2.0; grid_size], // Simulate 2m initial water depth
        initial_u: vec![0.5; grid_size], // Simulate initial x-velocity
        initial_v: vec![0.1; grid_size], // Simulate initial y-velocity
        width: 10,
        height: 10,
        t_end: 3600.0,
    });

    let response = client.predict_shallow_water_pino(request).await.unwrap().into_inner();
    
    Json(ShallowWaterInferenceResponse {
        status: "success",
        inference_ms: response.inference_ms,
        predicted_h_sample: response.predicted_h[0],
        predicted_u_sample: response.predicted_u[0],
        predicted_v_sample: response.predicted_v[0],
    })
}
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
        .route("/inference/swe", post(inference_swe))
        .route("/test_inference", get(test_inference))
        .route("/test_pino", get(test_pino_inference));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
