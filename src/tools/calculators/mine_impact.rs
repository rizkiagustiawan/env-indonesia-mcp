/// Mining Impact Assessment — Screening
/// Impact matrix for mining operations (nickel, coal, gold, tin)
/// Ref: Pambudi 2025; Rosada 2025; Nasution 2024; Manurung 2025

pub fn assess(
    mine_type: &str,
    lat: f64,
    lon: f64,
    area_ha: f64,
    deforestation_ha: f64,
    water_pollution_level: &str,
    has_tailings: bool,
    has_amd: bool,
    social_displacement: u32,
) -> String {
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Mining Impact Assessment (Screening)\n");
    out.push_str("Ref: Pambudi 2025; Rosada 2025; Nasution 2024; Manurung 2025\n\n");

    out.push_str(&format!("Mine type: {}\n", mine_type.to_uppercase()));
    out.push_str(&format!("Location: ({:.4}, {:.4})\n", lat, lon));
    out.push_str(&format!("Mine area: {:.0} ha\n\n", area_ha));

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

    out.push_str(&format!("Mining Profile: {}\n\n", mine_profile.0));
    out.push_str(&format!("IMPACT MATRIX ({}):\n", mine_profile.0));
    out.push_str(&format!("{:<30} {:>10}\n", "Impact", "Severity"));
    out.push_str(&"-".repeat(40).to_string());
    out.push('\n');
    for (impact, severity) in &mine_profile.1 {
        out.push_str(&format!("{:<30} {:>10}\n", impact, severity));
    }

    out.push_str("\nSITE-SPECIFIC DATA:\n");
    out.push_str(&format!("  Deforestation: {:.0} ha ({:.1}% of mine area)\n", deforestation_ha, (deforestation_ha / area_ha * 100.0)));
    out.push_str(&format!("  Water pollution: {}\n", water_pollution_level));
    out.push_str(&format!("  Tailings present: {}\n", if has_tailings { "YES" } else { "NO" }));
    out.push_str(&format!("  Acid Mine Drainage: {}\n", if has_amd { "YES" } else { "NO" }));
    out.push_str(&format!("  Social displacement: {} people\n\n", social_displacement));

    let mut total_score = 0;
    for (_, severity) in &mine_profile.1 {
        total_score += match severity.to_string().as_str() {
            "CRITICAL" => 4,
            "HIGH" => 3,
            "MEDIUM" => 2,
            "LOW" => 1,
            _ => 0,
        };
    }
    let max_score = mine_profile.1.len() * 4;
    let impact_pct = (total_score as f64 / max_score as f64) * 100.0;

    let overall = if impact_pct > 75.0 { "CRITICAL — Immediate action" }
        else if impact_pct > 50.0 { "HIGH — Mitigation needed" }
        else if impact_pct > 25.0 { "MODERATE — Monitor" }
        else { "LOW — Routine" };

    out.push_str(&format!("Overall Impact Score: {}/{} ({:.0}%) → {}\n\n", total_score, max_score, impact_pct, overall));

    out.push_str("MITIGATION RECOMMENDATION:\n");
    out.push_str("  1. Revegetation/reclamation of mined area\n");
    out.push_str("  2. Tailings management (dry stacking, not river/sea disposal)\n");
    out.push_str("  3. Water treatment (settling pond, active/passive treatment)\n");
    out.push_str("  4. AMD prevention (limestone neutralization, wetland)\n");
    out.push_str("  5. Community resettlement + livelihood restoration\n");
    out.push_str("  6. Post-mining land use planning\n");
    out.push_str("  7. Biodiversity offset program\n\n");

    out.push_str("LIMITATION:\n");
    out.push_str("  - Screening tool only — not full AMDAL for mining\n");
    out.push_str("  - Impact profiles are generic — site conditions vary\n");
    out.push_str("  - Social impact assessment needs field survey\n");
    out.push_str("  - Does not replace PermenLHK 4/2021 AMDAL screening\n");
    out.push_str("  - Satellite-based deforestation is approximate\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}
