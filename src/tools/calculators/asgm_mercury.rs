use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// ASGM Mercury Assessment
/// Hg mass balance + health risk for Artisanal Small-scale Gold Mining
/// Ref: Agustiani et al. 2025 (Sukabumi); Desmaiani et al. 2026 (W. Kalimantan)

pub fn assess(
    hg_conc_water: f64,
    hg_conc_sediment: f64,
    gold_production_kg_yr: f64,
    population_exposed: u32,
) -> String {
    if hg_conc_water < 0.0 || hg_conc_sediment < 0.0 || gold_production_kg_yr < 0.0 {
        return json!({"error": "E102", "message": "Parameter tidak boleh negatif"}).to_string();
    }

    let hg_ratio = 1.5;
    let hg_total = gold_production_kg_yr * hg_ratio;
    let hg_recovered = hg_total * 0.20;
    let hg_atmosphere = hg_total * 0.60;
    let hg_tailings = hg_total * 0.20;

    let bm_hg_water = 0.002;
    let bm_hg_class4 = 0.005;

    let exceed_ratio = if bm_hg_water > 0.0 { hg_conc_water / bm_hg_water } else { 0.0 };

    let bw = 60.0;
    let ir = 2.0;
    let ef = 365.0;
    let ed = 30.0;
    let at = 70.0 * 365.0;
    let rfd_mehg = 1.0e-4;
    let sf_mehg = 1.0e-1;

    let cdi = (hg_conc_water * ir * ef * ed) / (bw * at);
    let hq = cdi / rfd_mehg;
    let ilcr = (hg_conc_water * ir * ef * ed * sf_mehg) / (bw * at);

    let at_risk = (population_exposed as f64) * (ilcr / 1e-4).max(0.0).min(1.0);

    let res_hg_total = ScientificResult::new("total_hg_used", hg_total, "kg/yr")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "UNEP_ASGM_Ratio", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("assumption", &format!("Hg:Au ratio of {}", hg_ratio)));

    let claims_distribution = vec![
        Claim::new("recovered", &hg_recovered.to_string()),
        Claim::new("atmosphere", &hg_atmosphere.to_string()),
        Claim::new("tailings", &hg_tailings.to_string()),
    ];

    let mut res_distribution = ScientificResult::new("hg_distribution", 1.0, "boolean")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "UNEP_2013", "2026-08-19T00:00:00Z"));

    for claim in claims_distribution {
        res_distribution = res_distribution.with_claim(claim);
    }

    let _status_water = if hg_conc_water > bm_hg_class4 {
        ResultStatus::ValidationFailed
    } else if hg_conc_water > bm_hg_water {
        ResultStatus::ScreeningOnly
    } else {
        ResultStatus::Valid
    };

    let mut res_water_hq = ScientificResult::new("hg_water_hq", hq, "dimensionless")
        .with_status(if hq <= 1.0 { ResultStatus::Valid } else { ResultStatus::ValidationFailed })
        .with_provenance(Provenance::new("calculation", "EPA_HHRA", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("ilcr", &ilcr.to_string()))
        .with_claim(Claim::new("population_at_risk_cancer", &at_risk.to_string()));

    if hg_conc_water > bm_hg_water {
        res_water_hq = res_water_hq.with_claim(Claim::new("warning", &format!("Water Hg exceeds Class III standard by {:.1}x", exceed_ratio)));
    }

    let status_sed = if hg_conc_sediment > 0.3 { ResultStatus::ValidationFailed } else { ResultStatus::Valid };

    let mut res_sed = ScientificResult::new("hg_sediment", hg_conc_sediment, "mg/kg")
        .with_status(status_sed)
        .with_provenance(Provenance::new("observation", "User_Input", "2026-08-19T00:00:00Z"));

    if hg_conc_sediment > 0.3 {
        res_sed = res_sed.with_claim(Claim::new("warning", "Exceeds sediment quality (0.3 mg/kg, NOAA SQG)"));
    }

    json!([
        serde_json::from_str::<serde_json::Value>(&res_hg_total.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_distribution.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_water_hq.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_sed.emit_validated()).unwrap()
    ]).to_string()
}
