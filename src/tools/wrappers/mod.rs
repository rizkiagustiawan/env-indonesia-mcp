use reqwest::Client;
use serde_json::json;

// Port mapping berdasarkan project yang sudah ada
const GEO_ESG_PORT: &str = "8000";
const FLOOD_AI_PORT: &str = "8001"; 
const METHANE_PORT: &str = "8002";
const GROUNDWATER_PORT: &str = "8003";
const AIR_QUALITY_PORT: &str = "8004";

pub async fn trigger_esg_audit(client: &Client, site_id: &str) -> String {
    let url = format!("http://localhost:{}/generate-esg-report", GEO_ESG_PORT);
    match client.post(&url).json(&json!({"site_id": site_id})).send().await {
        Ok(resp) => match resp.text().await {
            Ok(body) => format!("=== GeoESG-Final Audit ===\n{}", &body[..body.len().min(4000)]),
            Err(e) => format!("Error reading response: {}", e),
        },
        Err(e) => format!("Service offline or error: {}. Pastikan GeoESG-Final berjalan di port {}.", e, GEO_ESG_PORT),
    }
}

pub async fn predict_flood(client: &Client, lat: f64, lon: f64) -> String {
    let url = format!("http://localhost:{}/predict/at?lat={}&lon={}", FLOOD_AI_PORT, lat, lon);
    match client.get(&url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(body) => format!("=== Flood AI Prediction ===\n{}", &body[..body.len().min(4000)]),
            Err(e) => format!("Error reading response: {}", e),
        },
        Err(e) => format!("Service offline or error: {}. Pastikan geo-ntb-flood-ai berjalan di port {}.", e, FLOOD_AI_PORT),
    }
}

pub async fn get_methane_plumes(client: &Client) -> String {
    let url = format!("http://localhost:{}/api/methane/plumes", METHANE_PORT);
    match client.get(&url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(body) => format!("=== Methane Plumes Data ===\n{}", &body[..body.len().min(4000)]),
            Err(e) => format!("Error reading response: {}", e),
        },
        Err(e) => format!("Service offline or error: {}. Pastikan Gas-Metana-NTB berjalan di port {}.", e, METHANE_PORT),
    }
}

pub async fn get_groundwater_status(client: &Client) -> String {
    // API endpoint contoh, sesuaikan dengan endpoint riil
    let url = format!("http://localhost:{}/api/status", GROUNDWATER_PORT);
    match client.get(&url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(body) => format!("=== Groundwater Status ===\n{}", &body[..body.len().min(4000)]),
            Err(e) => format!("Error reading response: {}", e),
        },
        Err(e) => format!("Service offline or error: {}. Pastikan ntb-groundwater-monitor berjalan di port {}.", e, GROUNDWATER_PORT),
    }
}

pub async fn get_air_quality(client: &Client) -> String {
    let url = format!("http://localhost:{}/api/health", AIR_QUALITY_PORT); // Health or dashboard endpoint
    match client.get(&url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(body) => format!("=== Air Quality Monitor ===\n{}", &body[..body.len().min(4000)]),
            Err(e) => format!("Error reading response: {}", e),
        },
        Err(e) => format!("Service offline or error: {}. Pastikan airtestingquality berjalan di port {}.", e, AIR_QUALITY_PORT),
    }
}
