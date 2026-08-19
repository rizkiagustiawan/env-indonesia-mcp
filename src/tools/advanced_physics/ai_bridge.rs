use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct InferenceRequest {
    pub site_id: String,
    pub bbox: Vec<f64>,
    pub initial_h: Vec<f64>,
    pub dem: Vec<f64>,
    pub width: usize,
    pub height: usize,
    pub t_end: f64,
}

#[derive(Deserialize)]
pub struct InferenceResponse {
    pub status: String,
    pub inference_ms: f64,
    pub predicted_h: Vec<f64>,
}

pub fn call_ai_node(req: InferenceRequest) -> Result<InferenceResponse, String> {
    let client = reqwest::blocking::Client::new();
    let url = "http://127.0.0.1:3000/inference/swe"; 
    
    let resp = client.post(url).json(&req).send().map_err(|e| e.to_string())?;
    
    if resp.status().is_success() {
        let json: InferenceResponse = resp.json().map_err(|e| e.to_string())?;
        Ok(json)
    } else {
        Err(format!("API Gateway returned status: {}", resp.status()))
    }
}
