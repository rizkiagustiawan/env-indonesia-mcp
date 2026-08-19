use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Heavy Metal Risk Assessment
/// HPI (Heavy metal Pollution Index) + US EPA RAGS health risk
/// Ref: Mohsen & Bhatt 1989 (HPI); US EPA RAGS (health risk)
/// Baku mutu: PP 22/2021

pub fn assess(
    pb: f64, cd: f64, hg: f64, as_: f64, cr: f64,
    body_weight_kg: f64,
    intake_l_per_day: f64,
    exposure_years: f64,
) -> String {
    if pb < 0.0 || cd < 0.0 || hg < 0.0 || as_ < 0.0 || cr < 0.0 || body_weight_kg <= 0.0 || intake_l_per_day <= 0.0 || exposure_years <= 0.0 {
        return json!({"error": "E102", "message": "Parameter konsentrasi tidak boleh negatif; BW, IR, dan ED harus > 0"}).to_string();
    }

    let standards: [(&str, f64, f64, f64, f64); 5] = [
        ("Pb", pb, 0.05, 0.0035, 0.5),
        ("Cd", cd, 0.01, 0.0005, 0.5),
        ("Hg", hg, 0.001, 0.00003, 0.3),
        ("As", as_, 0.05, 0.01, 0.0003),
        ("Cr", cr, 0.05, 0.05, 0.003),
    ];

    let mut total_w: f64 = 0.0;
    let mut total_wq: f64 = 0.0;
    
    for &(_name, conc, std, _rfd, _sf) in standards.iter() {
        let qi = (conc / std) * 100.0;
        let wi = 1.0 / std;
        total_w += wi;
        total_wq += wi * qi;
    }

    let hpi = total_wq / total_w;
    let hpi_class = if hpi < 100.0 { "Good" } else if hpi < 500.0 { "Lightly-Moderately Polluted" } else { "Heavily Polluted" };

    let bw = body_weight_kg;
    let ir = intake_l_per_day;
    let ed = exposure_years;
    let ef = 365.0;
    let at_nocarc = ed * 365.0;
    let at_carc = 70.0 * 365.0;

    let mut total_hq = 0.0;
    let mut total_ilcr = 0.0;
    
    let mut hq_claims = vec![];

    for &(name, conc, _std, rfd, sf) in standards.iter() {
        let cdi = (conc * ir * ef * ed) / (bw * at_nocarc);
        let hq = cdi / rfd;
        let ilcr = (conc * ir * ef * ed * sf) / (bw * at_carc);
        total_hq += hq;
        total_ilcr += ilcr;
        hq_claims.push(Claim::new(&format!("{}_hq", name), &hq.to_string()));
        hq_claims.push(Claim::new(&format!("{}_ilcr", name), &ilcr.to_string()));
    }

    let res_hpi = ScientificResult::new("heavy_metal_pollution_index", hpi, "dimensionless")
        .with_status(if hpi < 100.0 { ResultStatus::Valid } else { ResultStatus::ValidationFailed })
        .with_provenance(Provenance::new("calculation", "Mohsen_Bhatt_1989", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("classification", hpi_class));

    let mut res_hq = ScientificResult::new("total_hazard_quotient", total_hq, "dimensionless")
        .with_status(if total_hq < 1.0 { ResultStatus::Valid } else { ResultStatus::ValidationFailed })
        .with_provenance(Provenance::new("calculation", "EPA_RAGS", "2026-08-19T00:00:00Z"));
        
    for claim in hq_claims {
        res_hq = res_hq.with_claim(claim);
    }
        
    let res_ilcr = ScientificResult::new("total_ilcr", total_ilcr, "probability")
        .with_status(if total_ilcr < 1e-4 { ResultStatus::Valid } else { ResultStatus::ValidationFailed })
        .with_provenance(Provenance::new("calculation", "EPA_RAGS", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("limitation", "Assumes oral ingestion only (no inhalation/dermal)"));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_hpi.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_hq.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_ilcr.emit_validated()).unwrap()
    ]).to_string()
}
