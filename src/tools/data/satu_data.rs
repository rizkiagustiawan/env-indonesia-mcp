use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use reqwest::Client;
use serde_json::json;

pub async fn search(client: &Client, query: &str, limit: u32) -> String {
    let url = format!(
        "https://data.go.id/api/3/action/package_search?q={}&rows={}&fq=groups:lingkungan-dan-sumber-daya-alam",
        urlencoding(query), limit
    );

    let mut results = vec![];

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                if let Some(ds_results) = v
                    .get("result")
                    .and_then(|r| r.get("results"))
                    .and_then(|r| r.as_array())
                {
                    let count = v
                        .get("result")
                        .and_then(|r| r.get("count"))
                        .and_then(|c| c.as_f64())
                        .unwrap_or(0.0);

                    let mut res = ScientificResult::new("satu_data_dataset_count", count, "count")
                        .with_status(ResultStatus::Valid)
                        .with_provenance(Provenance::new("database", "data.go.id", "2026-08-19T00:00:00Z"));

                    for (i, ds) in ds_results.iter().enumerate() {
                        let title = ds.get("title").and_then(|t| t.as_str()).unwrap_or("?");
                        let org = ds
                            .get("organization")
                            .and_then(|o| o.get("title"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("?");
                        let url_str = ds.get("url").and_then(|u| u.as_str()).unwrap_or("");
                        
                        res = res.with_claim(Claim::new(&format!("dataset_{}", i), &format!("{} ({})", title, org)));
                        if !url_str.is_empty() {
                            res = res.with_claim(Claim::new(&format!("url_{}", i), url_str));
                        }
                    }
                    results.push(res);
                } else {
                    let res = ScientificResult::new("satu_data_search_error", 1.0, "boolean")
                        .with_status(ResultStatus::ValidationFailed)
                        .with_provenance(Provenance::new("database", "data.go.id", "2026-08-19T00:00:00Z"))
                        .with_claim(Claim::new("error", "No results or API format changed"));
                    results.push(res);
                }
            }
            Err(e) => {
                 let res = ScientificResult::new("satu_data_search_error", 1.0, "boolean")
                    .with_status(ResultStatus::ValidationFailed)
                    .with_provenance(Provenance::new("database", "data.go.id", "2026-08-19T00:00:00Z"))
                    .with_claim(Claim::new("error", &format!("Parse error: {}", e)));
                 results.push(res);
            }
        },
        Err(e) => {
             let res = ScientificResult::new("satu_data_search_error", 1.0, "boolean")
                .with_status(ResultStatus::ValidationFailed)
                .with_provenance(Provenance::new("database", "data.go.id", "2026-08-19T00:00:00Z"))
                .with_claim(Claim::new("error", &format!("Connection error: {}", e)));
             results.push(res);
        }
    }
    
    let json_array: Vec<serde_json::Value> = results.iter()
        .map(|r| serde_json::from_str(&r.clone().emit_validated()).unwrap())
        .collect();

    json!(json_array).to_string()
}

fn urlencoding(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}
