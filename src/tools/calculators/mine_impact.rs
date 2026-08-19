use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Mining Impact Assessment — Screening
/// Impact matrix for mining operations (nickel, coal, gold, tin)
/// Ref: Pambudi 2025; Rosada 2025; Nasution 2024; Manurung 2025

pub fn assess(
    mine_type: &str,
    _lat: f64,
    _lon: f64,
    area_ha: f64,
    deforestation_ha: f64,
    water_pollution_level: &str,
    has_tailings: bool,
    has_amd: bool,
    social_displacement: u32,
) -> String {
    if area_ha <= 0.0 || deforestation_ha < 0.0 {
        return json!({"error": "E102", "message": "Area parameters must be valid positive values"}).to_string();
    }

    let mine_profile = match mine_type.to_lowercase().as_str() {
        "nickel" => ("Nickel (laterite)", vec![
            ("Deforestation", "HIGH"),
            ("Tailings (land/sea)", "CRITICAL"),
            ("Heavy metals (Ni, Cr, Co)", "HIGH"),
            ("Sediment runoff", "HIGH"),
            ("Biodiversity loss", "HIGH"),
            ("Social displacement", "MEDIUM"),
        ]),
        "coal" => ("Coal", vec![
            ("Deforestation", "HIGH"),
            ("Acid Mine Drainage", "CRITICAL"),
            ("Heavy metals (Fe, Mn, Al)", "HIGH"),
            ("Air quality (dust, SO2)", "HIGH"),
            ("Land degradation", "HIGH"),
            ("Social displacement", "MEDIUM"),
        ]),
        "gold" => ("Gold (ASGM)", vec![
            ("Mercury contamination", "CRITICAL"),
            ("Cyanide leakage", "HIGH"),
            ("Deforestation", "MEDIUM"),
            ("River sedimentation", "HIGH"),
            ("Health risk (Hg exposure)", "CRITICAL"),
            ("Social conflict", "HIGH"),
        ]),
        "tin" => ("Tin (alluvial)", vec![
            ("Land degradation", "CRITICAL"),
            ("Marine sediment plume", "CRITICAL"),
            ("Coral reef damage", "HIGH"),
            ("Heavy metals (Sn, Pb)", "MEDIUM"),
            ("Water turbidity", "HIGH"),
            ("Social conflict", "HIGH"),
        ]),
        _ => ("Generic mining", vec![
            ("Deforestation", "MEDIUM"),
            ("Water pollution", "MEDIUM"),
            ("Land degradation", "MEDIUM"),
            ("Social impact", "MEDIUM"),
            ("Air quality", "LOW"),
            ("Biodiversity", "MEDIUM"),
        ]),
    };

    let mut total_score = 0;
    let mut claims = vec![];
    claims.push(Claim::new("mine_profile", mine_profile.0));
    claims.push(Claim::new("deforestation_ratio", &(deforestation_ha / area_ha).to_string()));
    claims.push(Claim::new("water_pollution", water_pollution_level));
    claims.push(Claim::new("has_tailings", &has_tailings.to_string()));
    claims.push(Claim::new("has_amd", &has_amd.to_string()));
    claims.push(Claim::new("social_displacement_count", &social_displacement.to_string()));

    for (impact, severity) in &mine_profile.1 {
        total_score += match severity.to_string().as_str() {
            "CRITICAL" => 4,
            "HIGH" => 3,
            "MEDIUM" => 2,
            "LOW" => 1,
            _ => 0,
        };
        claims.push(Claim::new(&format!("profile_impact_{}", impact.replace(" ", "_").to_lowercase()), severity));
    }
    
    let max_score = mine_profile.1.len() * 4;
    let impact_pct = (total_score as f64 / max_score as f64) * 100.0;

    let overall = if impact_pct > 75.0 { "CRITICAL" }
        else if impact_pct > 50.0 { "HIGH" }
        else if impact_pct > 25.0 { "MODERATE" }
        else { "LOW" };

    let mut res = ScientificResult::new("mining_impact_score", impact_pct, "%")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("screening", "Generic_Mining_Matrix", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("overall_severity", overall))
        .with_claim(Claim::new("limitation", "Generic screening tool, does not replace formal AMDAL environmental assessment."));

    for claim in claims {
        res = res.with_claim(claim);
    }

    json!([
        serde_json::from_str::<serde_json::Value>(&res.emit_validated()).unwrap()
    ]).to_string()
}
