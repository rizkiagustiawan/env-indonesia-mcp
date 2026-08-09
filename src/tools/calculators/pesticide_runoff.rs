/// Pesticide Runoff Risk — GUS Index + RUSLE
/// GUS = log(Koc) × √(t½_days) — Groundwater Ubiquity Score
/// GUS < 1.8 = immobile, 1.8-2.8 = transition, > 2.8 = mobile (leaching risk)
/// Ref: Gustafson 1989; USDA NRCS; PP 22/2021
pub fn assess(pesticide: &str, application_rate_kg_ha: f64, koc: f64, half_life_days: f64, rainfall_mm: f64, slope_pct: f64, soil_erodibility: f64, area_ha: f64, water_body_distance_m: f64) -> String {
    let mut out = String::from("=== Pesticide Runoff & Leaching Risk ===\n");
    out.push_str("Ref: Gustafson 1989 (GUS); USDA NRCS; PP 22/2021\n\n");

    let gus = (koc.ln() / 10.0_f64.ln()) * half_life_days.sqrt();

    out.push_str(&format!("Pesticide: {}\n", pesticide));
    out.push_str(&format!("Koc: {:.0} (adsorption)), t½: {:.0} days\n", koc, half_life_days));
    out.push_str(&format!("Application: {:.2} kg/ha, Area: {:.0} ha\n", application_rate_kg_ha, area_ha));
    out.push_str(&format!("Rainfall: {:.0}mm, Slope: {:.0}%, Distance to water: {:.0}m\n\n", rainfall_mm, slope_pct, water_body_distance_m));

    out.push_str("═══ GROUNDWATER LEACHING RISK (GUS Index) ═══\n");
    out.push_str(&format!("  GUS = log(Koc)) × √(t½) = {:.2} × {:.2} = {:.2}\n", koc.log10(), half_life_days.sqrt(), gus));

    let (leaching_risk, leaching_desc) = if gus < 1.8 {
        ("LOW", "Immobile — unlikely to leach to groundwater")
    } else if gus < 2.8 {
        ("MODERATE", "Transition — possible leaching with high recharge")
    } else {
        ("HIGH", "Mobile — significant leaching risk to groundwater")
    };
    out.push_str(&format!("  >> Leaching risk: {} — {}\n\n", leaching_risk, leaching_desc));

    // PP 22/2021 groundwater
    let gw_threshold = match pesticide.to_lowercase().as_str() {
        s if s.contains("atrazine") => 0.0005, // organoklorin
        s if s.contains("glyphosate") => 0.001,
        _ => 0.001, // generic pesticides
    };

    // Estimated leaching concentration
    let recharge_m = rainfall_mm / 1000.0 * 0.2; // 20% recharge
    let leached_mass_kg = match leaching_risk {
        "HIGH" => application_rate_kg_ha * area_ha * 0.10,
        "MODERATE" => application_rate_kg_ha * area_ha * 0.03,
        _ => application_rate_kg_ha * area_ha * 0.01,
    };
    let gw_conc_mg_l = if recharge_m > 0.0 { leached_mass_kg * 1000.0 / (area_ha * 10000.0 * recharge_m) } else { 0.0 };

    out.push_str(&format!("  Estimated GW conc: {:.6} mg/L (PP 22/2021 limit: {:.4}))\n", gw_conc_mg_l, gw_threshold));
    out.push_str(&format!("  Status: {}\n\n", if gw_conc_mg_l <= gw_threshold { "✅" } else { "❌ MELEBIHI" }));

    out.push_str("═══ SURFACE RUNOFF RISK ═══\n");
    let rusle_a = soil_erodibility * 2.0 * (slope_pct / 10.0).min(2.0) * 0.5; // simplified RUSLE
    let runoff_pct = match slope_pct {
        s if s < 2.0 => 1.0, s if s < 5.0 => 3.0, s if s < 10.0 => 8.0, _ => 15.0,
    };
    let runoff_mass_kg = application_rate_kg_ha * area_ha * (runoff_pct / 100.0);
    let water_body_conc = if water_body_distance_m > 0.0 { runoff_mass_kg * 1000.0 / (water_body_distance_m * 1000.0 * 10.0) } else { 0.0 };

    out.push_str(&format!("  RUSLE erosion: {:.2} ton/ha\n", rusle_a));
    out.push_str(&format!("  Runoff mass: {:.2} kg ({:.0}% of applied))\n", runoff_mass_kg, runoff_pct));
    out.push_str(&format!("  Est. conc at water body: {:.4} mg/L\n\n", water_body_conc));

    out.push_str("═══ MITIGATION ═══\n");
    if leaching_risk == "HIGH" || water_body_conc > 0.001 {
        out.push_str("  ❌ RISK — recommended actions:\n");
        out.push_str("  1. Buffer strip 10-30m around water body\n");
        out.push_str("  2. Reduce application rate or frequency\n");
        out.push_str("  3. Use alternative pesticide (low Koc, short t½)\n");
        out.push_str("  4. Time application (avoid rainy season)\n");
        out.push_str("  5. Integrated Pest Management (IPM)\n\n");
    } else {
        out.push_str("  ✅ Low risk — continue monitoring\n\n");
    }

    out.push_str("═══ PEMANTAUAN (RPL) ═══\n");
    out.push_str("  Parameter: pesticide residue in soil, groundwater, surface water\n");
    out.push_str("  Frekuensi: seasonal (aplikasi), monthly (water body)\n");

    out.push_str("\n  Ref: Gustafson 1989 (GUS); PP 22/2021 Annex VI; USDA NRCS\n");
    out
}
