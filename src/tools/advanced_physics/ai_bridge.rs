use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct InferenceRequest {
    pub site_id: String,
    pub bbox: Vec<f64>,
    pub initial_h: Vec<f64>,
    pub width: usize,
    pub height: usize,
    pub t_end: f64,
}

#[derive(Deserialize)]
pub struct InferenceResponse {
    pub status: String,
    pub inference_ms: f64,
    pub predicted_depth_sample: f64,
}

pub fn call_ai_node(req: InferenceRequest) -> Result<InferenceResponse, String> {
    // Synchronous HTTP request to Axum Gateway which acts as gRPC bridge to Python
    // We use reqwest blocking client since the MCP tool runs in a synchronous thread pool context
    let client = reqwest::blocking::Client::new();
    let url = "http://127.0.0.1:3000/test_inference"; 
    // In reality this should be a POST to /inference but we reuse our existing endpoint for now
    
    let resp = client.get(url).send().map_err(|e| e.to_string())?;
    
    if resp.status().is_success() {
        let json: InferenceResponse = resp.json().map_err(|e| e.to_string())?;
        Ok(json)
    } else {
        Err(format!("API Gateway returned status: {}", resp.status()))
    }
}
