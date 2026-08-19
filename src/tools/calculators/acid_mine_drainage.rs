use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Acid Mine Drainage (AMD) Calculator
/// Ref: PermenLH 113/2003, Acid Base Accounting (ABA)

pub fn calculate(sulfur_pct: f64, anc_kg_h2so4_t: f64, nag_ph: Option<f64>) -> String {
    if sulfur_pct < 0.0 {
        return json!({"error": "E102", "message": "Parameter tidak boleh negatif"}).to_string();
    }

    let mpa = sulfur_pct * 30.6; // kg H2SO4/ton
    let napp = mpa - anc_kg_h2so4_t;

    let status = if napp > 0.0 {
        if let Some(ph) = nag_ph {
            if ph < 4.5 {
                "PAF"
            } else {
                "Uncertain_PAF"
            }
        } else {
            "PAF"
        }
    } else if napp < 0.0 {
        if let Some(ph) = nag_ph {
            if ph >= 4.5 {
                "NAF"
            } else {
                "Uncertain_NAF"
            }
        } else {
            "NAF"
        }
    } else {
        "Uncertain"
    };

    let mut res_napp = ScientificResult::new("NAPP", napp, "kg H2SO4/t")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("calculation", "ABA_Static_Test", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("classification", status));
    
    if status == "PAF" {
        res_napp = res_napp.with_claim(Claim::new("mitigation", "Enkapsulasi NAF, Wet Cover, atau Active Treatment"));
    }

    let res_mpa = ScientificResult::new("MPA", mpa, "kg H2SO4/t")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("calculation", "ABA_Static_Test", "2026-08-19T00:00:00Z"));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_napp.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_mpa.emit_validated()).unwrap()
    ]).to_string()
}
