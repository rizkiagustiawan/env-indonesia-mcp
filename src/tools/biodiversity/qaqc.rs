use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Validasi QA/QC Data Lingkungan
/// Ref: US EPA QA/QC Guidance, SNI 6989-series

pub fn validate(data_json: &str) -> String {
    let samples: Vec<serde_json::Value> = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(e) => return json!({"error": "E100", "message": format!("Gagal parsing JSON: {}", e)}).to_string(),
    };

    if samples.is_empty() {
        return json!({"error": "E102", "message": "Array data QA/QC kosong"}).to_string();
    }

    let mut total_samples = 0_usize;
    let mut rpd_pass = 0_usize;
    let mut rpd_fail = 0_usize;
    let mut spike_pass = 0_usize;
    let mut spike_fail = 0_usize;
    let mut blank_pass = 0_usize;
    let mut blank_fail = 0_usize;
    let mut flags: Vec<String> = Vec::new();

    for sample in &samples {
        total_samples += 1;

        let sample_id = sample.get("sample").and_then(|v| v.as_str()).unwrap_or("?");
        let value = sample.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);

        // RPD check (duplicate)
        if let Some(dup) = sample.get("duplicate").and_then(|v| v.as_f64()) {
            let avg = (value + dup) / 2.0;
            let rpd = if avg > 1e-12 {
                ((value - dup).abs() / avg) * 100.0
            } else {
                0.0
            };

            let rpd_limit = 20.0;
            let rpd_ok = rpd <= rpd_limit;

            if rpd_ok {
                rpd_pass += 1;
            } else {
                rpd_fail += 1;
                flags.push(format!("{}: RPD {:.1}% melebihi batas {}%", sample_id, rpd, rpd_limit));
            }
        }

        // Spike recovery
        if let (Some(spike), Some(spike_amt)) = (
            sample.get("spike").and_then(|v| v.as_f64()),
            sample.get("spike_amount").and_then(|v| v.as_f64()),
        ) {
            let recovery = if spike_amt > 1e-12 {
                ((spike - value) / spike_amt) * 100.0
            } else {
                0.0
            };

            let recovery_ok = (80.0..=120.0).contains(&recovery);

            if recovery_ok {
                spike_pass += 1;
            } else {
                spike_fail += 1;
                flags.push(format!("{}: Recovery {:.1}% di luar 80-120%", sample_id, recovery));
            }
        }

        // Blank check
        if let Some(blank) = sample.get("blank").and_then(|v| v.as_f64()) {
            let mdl_estimate = value * 0.1;
            let blank_ok = blank < mdl_estimate.max(0.01);

            if blank_ok {
                blank_pass += 1;
            } else {
                blank_fail += 1;
                flags.push(format!("{}: Blank {:.4} melebihi MDL", sample_id, blank));
            }
        }
    }

    let total_checks = rpd_pass + rpd_fail + spike_pass + spike_fail + blank_pass + blank_fail;
    let total_pass = rpd_pass + spike_pass + blank_pass;
    let pass_pct = if total_checks > 0 {
        (total_pass as f64 / total_checks as f64) * 100.0
    } else {
        100.0
    };

    let status = if flags.is_empty() {
        ResultStatus::Valid
    } else if pass_pct >= 80.0 {
        ResultStatus::ValidWithAssumptions
    } else {
        ResultStatus::ValidationFailed
    };

    let mut res = ScientificResult::new("qaqc_pass_rate", pass_pct, "%")
        .with_status(status)
        .with_provenance(Provenance::new("calculation", "EPA_QAQC_SNI6989", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("total_samples", &total_samples.to_string()))
        .with_claim(Claim::new("total_checks", &total_checks.to_string()))
        .with_claim(Claim::new("total_passed", &total_pass.to_string()));

    if !flags.is_empty() {
        res = res.with_claim(Claim::new("flags", &flags.join("; ")));
    }

    json!([
        serde_json::from_str::<serde_json::Value>(&res.emit_validated()).unwrap()
    ]).to_string()
}
