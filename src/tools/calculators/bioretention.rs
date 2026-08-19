use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Bioretention / Rain Garden Design
/// Ref: PU Cipta Karya, FHWA HEC-22, Prince George's County BMP Manual

pub fn design(
    q_design_m3s: f64,
    ksat_m_hr: f64,
    ponding_depth_m: f64,
    media_depth_m: f64,
    drain_time_hr: f64,
) -> String {
    if q_design_m3s <= 0.0 || ksat_m_hr <= 0.0 || ponding_depth_m <= 0.0 || media_depth_m <= 0.0 || drain_time_hr <= 0.0 {
        return json!({"error": "E102", "message": "Semua parameter desain harus > 0"}).to_string();
    }

    // Design storm volume (simplified: Q × duration assumed 1 hr = 3600s)
    let storm_duration_s = 3600.0; // 1-hour design storm
    let v_runoff = q_design_m3s * storm_duration_s; // m³

    // Surface area: Af = V_runoff / (Ksat × tf + dp)
    let storage_depth = ksat_m_hr * drain_time_hr + ponding_depth_m;
    let af = v_runoff / storage_depth;

    // Alternative Darcy-based: Af = Q × df / (Ksat × (hf + df) × tf)
    let af_darcy = q_design_m3s * 3600.0 * media_depth_m
        / (ksat_m_hr * (ponding_depth_m + media_depth_m) * drain_time_hr);

    // Use larger of two estimates (conservative)
    let af_design = af.max(af_darcy);

    // Media volume
    let media_volume = af_design * media_depth_m;

    // Gravel underdrain layer (typically 0.20-0.30 m)
    let gravel_depth = 0.25;
    let gravel_volume = af_design * gravel_depth;

    // Total excavation depth
    let total_depth = ponding_depth_m + media_depth_m + gravel_depth;
    let excavation_volume = af_design * total_depth;

    // Cost estimate (Indonesia, 2024)
    let cost_media_per_m3 = 350_000.0_f64; // IDR/m³ filter media
    let cost_gravel_per_m3 = 250_000.0;
    let cost_excavation_per_m3 = 80_000.0;
    let cost_plants_per_m2 = 75_000.0;
    let cost_underdrain_per_m = 150_000.0; // perforated PVC per meter

    let underdrain_length = af_design.sqrt() * 2.0; // approximate
    let total_cost = media_volume * cost_media_per_m3
        + gravel_volume * cost_gravel_per_m3
        + excavation_volume * cost_excavation_per_m3
        + af_design * cost_plants_per_m2
        + underdrain_length * cost_underdrain_per_m;

    let cost_per_m2 = total_cost / af_design;

    let res_area = ScientificResult::new("surface_area", af_design, "m2")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "FHWA_HEC_22", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("storm_duration_s", &storm_duration_s.to_string()))
        .with_claim(Claim::new("runoff_volume_m3", &v_runoff.to_string()));

    let res_excavation = ScientificResult::new("total_excavation_volume", excavation_volume, "m3")
        .with_status(ResultStatus::Valid)
        .with_provenance(Provenance::new("calculation", "FHWA_HEC_22", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("total_depth_m", &total_depth.to_string()));

    let mut claims_cost = vec![
        Claim::new("total_cost_idr", &total_cost.to_string()),
        Claim::new("cost_per_m2_idr", &cost_per_m2.to_string()),
    ];

    if drain_time_hr > 48.0 {
        claims_cost.push(Claim::new("warning", "Drain time > 48 hours increases mosquito and flooding risk. Consider higher Ksat."));
    }
    if ponding_depth_m > 0.30 {
        claims_cost.push(Claim::new("warning", "Ponding > 30 cm poses safety risks in public areas."));
    }

    let mut res_cost = ScientificResult::new("total_cost_estimate", total_cost, "IDR")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("calculation", "PU_CiptaKarya_2024", "2026-08-19T00:00:00Z"));
        
    for claim in claims_cost {
        res_cost = res_cost.with_claim(claim);
    }

    json!([
        serde_json::from_str::<serde_json::Value>(&res_area.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_excavation.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_cost.emit_validated()).unwrap()
    ]).to_string()
}
