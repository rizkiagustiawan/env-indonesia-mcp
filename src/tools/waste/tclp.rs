use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Skrining TCLP (Toxicity Characteristic Leaching Procedure)
/// Ref: PP 101/2014 tentang Pengelolaan Limbah Bahan Berbahaya dan Beracun

pub fn screen(parameters_json: &str) -> String {
    let params: Vec<serde_json::Value> = match serde_json::from_str(parameters_json) {
        Ok(v) => v,
        Err(e) => return json!({"error": "E100", "message": format!("Gagal parsing JSON: {}", e)}).to_string(),
    };

    if params.is_empty() {
        return json!({"error": "E102", "message": "Array parameter TCLP kosong"}).to_string();
    }

    // TCLP regulatory limits per PP 101/2014 Lampiran (mg/L)
    struct TclpLimit {
        name: &'static str,
        _cas: &'static str,
        limit_mgl: f64,
    }

    let limits = [
        TclpLimit {
            name: "As",
            _cas: "7440-38-2",
            limit_mgl: 5.0,
        },
        TclpLimit {
            name: "Ba",
            _cas: "7440-39-3",
            limit_mgl: 100.0,
        },
        TclpLimit {
            name: "Cd",
            _cas: "7440-43-9",
            limit_mgl: 1.0,
        },
        TclpLimit {
            name: "Cr",
            _cas: "7440-47-3",
            limit_mgl: 5.0,
        },
        TclpLimit {
            name: "Pb",
            _cas: "7439-92-1",
            limit_mgl: 5.0,
        },
        TclpLimit {
            name: "Hg",
            _cas: "7439-97-6",
            limit_mgl: 0.2,
        },
        TclpLimit {
            name: "Se",
            _cas: "7782-49-2",
            limit_mgl: 1.0,
        },
        TclpLimit {
            name: "Ag",
            _cas: "7440-22-4",
            limit_mgl: 5.0,
        },
        TclpLimit {
            name: "F",
            _cas: "16984-48-8",
            limit_mgl: 150.0,
        },
    ];

    let mut any_fail = false;
    let mut fail_params = Vec::new();
    let mut results = Vec::new();

    for param in &params {
        let name = match param.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => {
                return json!({"error": "E100", "message": "field 'name' tidak ada"}).to_string();
            }
        };
        let concentration = match param.get("concentration_mgl").and_then(|v| v.as_f64()) {
            Some(c) => c,
            None => {
                return json!({"error": "E100", "message": format!("field 'concentration_mgl' tidak ada untuk {}", name)}).to_string();
            }
        };

        let name_upper = name.to_uppercase();
        let name_trimmed = name_upper.trim();

        // Find matching limit
        let matched_limit = limits
            .iter()
            .find(|l| l.name.to_uppercase() == name_trimmed);

        let (status, is_fail, limit_val) = match matched_limit {
            Some(lim) => {
                if concentration > lim.limit_mgl {
                    (ResultStatus::ValidationFailed, true, lim.limit_mgl)
                } else {
                    (ResultStatus::Valid, false, lim.limit_mgl)
                }
            }
            None => (ResultStatus::OutOfDomain, false, f64::NAN),
        };

        if is_fail {
            any_fail = true;
            fail_params.push(name.to_string());
        }

        let mut sr = ScientificResult::new(&format!("tclp_conc_{}", name_trimmed.to_lowercase()), concentration, "mg/L")
            .with_status(status)
            .with_provenance(Provenance::new("regulatory_limit", "PP_101_2014", "2026-08-19T00:00:00Z"));
            
        if !limit_val.is_nan() {
             sr = sr.with_claim(Claim::new("regulatory_limit_mgl", &limit_val.to_string()));
        } else {
             sr = sr.with_claim(Claim::new("warning", "No regulatory limit found for parameter"));
        }

        results.push(sr);
    }

    let classification_status = if any_fail {
        ResultStatus::ValidationFailed
    } else {
        ResultStatus::Valid
    };

    let mut classification_result = ScientificResult::new("tclp_classification", if any_fail { 1.0 } else { 0.0 }, "boolean_fail")
        .with_status(classification_status)
        .with_provenance(Provenance::new("screening", "PP_101_2014", "2026-08-19T00:00:00Z"));

    if any_fail {
        classification_result = classification_result
            .with_claim(Claim::new("classification", "LIMBAH B3"))
            .with_claim(Claim::new("failed_parameters", &fail_params.join(", ")));
    } else {
        classification_result = classification_result
            .with_claim(Claim::new("classification", "NON-B3 (TCLP Passed)"))
            .with_claim(Claim::new("limitation", "Other B3 characteristics (corrosive, reactive, etc.) must still be verified"));
    }

    results.push(classification_result);

    let json_results: Vec<serde_json::Value> = results.iter()
        .map(|r| serde_json::from_str(&r.clone().emit_validated()).unwrap())
        .collect();

    json!(json_results).to_string()
}
