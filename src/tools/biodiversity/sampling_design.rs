use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Desain Sampling — Penentuan Jumlah Sampel
/// Ref: US EPA Guidance on Choosing a Sampling Design, RKL-RPL (PP 22/2021)

pub fn calculate(
    confidence_pct: f64,
    margin_error_pct: f64,
    std_deviation: f64,
    population_size: Option<u64>,
) -> String {
    if confidence_pct <= 0.0 || confidence_pct >= 100.0 {
        return json!({"error": "E102", "message": "Tingkat kepercayaan harus antara 0 dan 100 (eksklusif)"}).to_string();
    }
    if margin_error_pct <= 0.0 || std_deviation <= 0.0 {
        return json!({"error": "E102", "message": "Parameter harus > 0"}).to_string();
    }

    // Z-value lookup
    let z = if confidence_pct >= 99.0 {
        2.576
    } else if confidence_pct >= 95.0 {
        1.96
    } else if confidence_pct >= 90.0 {
        1.645
    } else if confidence_pct >= 85.0 {
        1.44
    } else if confidence_pct >= 80.0 {
        1.28
    } else {
        1.96 // fallback to 95%
    };

    let e = margin_error_pct / 100.0 * std_deviation; // absolute margin of error

    // n = (z² × s²) / e²
    let n_infinite = (z * z * std_deviation * std_deviation) / (e * e);
    
    // Finite population correction
    let n_adjusted = if let Some(n_pop) = population_size {
        if n_pop == 0 {
            return json!({"error": "E102", "message": "Ukuran populasi tidak boleh 0"}).to_string();
        }
        n_infinite / (1.0 + (n_infinite - 1.0) / n_pop as f64)
    } else {
        n_infinite
    };

    let n_adj_rounded = n_adjusted.ceil() as u64;

    let res_n = ScientificResult::new("minimum_sample_size", n_adj_rounded as f64, "count")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "EPA_Sampling_Design", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("methodology", "n = (z² × s²) / e²"))
        .with_claim(Claim::new("confidence_pct", &confidence_pct.to_string()))
        .with_claim(Claim::new("z_value", &z.to_string()));

    let mut res_n_mut = res_n;
    if population_size.is_some() {
        res_n_mut = res_n_mut.with_claim(Claim::new("correction", "Finite Population Correction (FPC) applied"));
    }

    json!([
        serde_json::from_str::<serde_json::Value>(&res_n_mut.emit_validated()).unwrap()
    ]).to_string()
}
