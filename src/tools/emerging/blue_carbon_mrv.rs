/// Blue Carbon MRV — Mangrove Remote Sensing
/// Ref: Malerba et al. 2023 (Nature Sustainability); Bourgeois et al. 2024
/// Allometric equations: Clough & Scott 1989; Comley 2005; Kangkuso 2018
pub fn assess(mangrove_species: &str, area_ha: f64, avg_dbh_cm: f64, avg_height_m: f64, tree_density_ha: f64, soil_carbon_ton_ha: f64) -> String {
    let mut out = String::from("=== Blue Carbon MRV (Mangrove) ===\n");
    out.push_str("Ref: Malerba 2023; Bourgeois 2024; Clough 1989; Kangkuso 2018\n\n");
    let (agb_per_tree_kg, bgb_ratio) = match mangrove_species.to_lowercase().as_str() {
        s if s.contains("rhizophora") || s.contains("bakau") => (0.235 * avg_dbh_cm.powf(2.4) * avg_height_m.powf(0.8), 0.41),
        s if s.contains("avicennia") || s.contains("api-api") => (0.168 * avg_dbh_cm.powf(2.5) * avg_height_m.powf(1.2), 0.41),
        s if s.contains("bruguiera") || s.contains("lenggadai") => (0.025 * avg_dbh_cm.powf(2.5) * avg_height_m.powf(1.2), 0.38),
        s if s.contains("sonneratia") || s.contains("pidada") => (0.031 * avg_dbh_cm.powf(2.5) * avg_height_m.powf(1.2), 0.40),
        _ => (0.102 * avg_dbh_cm.powf(2.5) * avg_height_m.powf(1.0), 0.39),
    };
    let agb_ton_ha = agb_per_tree_kg * tree_density_ha / 1000.0;
    let bgb_ton_ha = agb_ton_ha * bgb_ratio;
    let total_biomass_ton_ha = agb_ton_ha + bgb_ton_ha;
    let carbon_ton_ha = total_biomass_ton_ha * 0.47;
    let co2_ton_ha = carbon_ton_ha * 44.0 / 12.0;
    let total_agb = agb_ton_ha * area_ha;
    let total_bgb = bgb_ton_ha * area_ha;
    let total_carbon = carbon_ton_ha * area_ha;
    let total_co2 = co2_ton_ha * area_ha;
    let total_soil_carbon = soil_carbon_ton_ha * area_ha;
    let grand_total_carbon = total_carbon + total_soil_carbon;
    let grand_total_co2 = grand_total_carbon * 44.0 / 12.0;
    out.push_str(&format!("Species: {}, Area: {:.0} ha\n", mangrove_species, area_ha));
    out.push_str(&format!("DBH: {:.1}cm, Height: {:.1}m, Density: {:.0}/ha\n\n", avg_dbh_cm, avg_height_m, tree_density_ha));
    out.push_str("-- Allometric Equation --\n");
    out.push_str("  Species-specific (Clough 1989 / Kangkuso 2018)\n\n");
    out.push_str("-- Biomass & Carbon --\n\n");
    out.push_str(&format!("  AGB: {:.1} ton/ha ({:.0} total)\n", agb_ton_ha, total_agb));
    out.push_str(&format!("  BGB: {:.1} ton/ha ({:.0} total) [R:S={:.2}]\n", bgb_ton_ha, total_bgb, bgb_ratio));
    out.push_str(&format!("  Soil C: {:.1} ton/ha ({:.0} total)\n\n", soil_carbon_ton_ha, total_soil_carbon));
    out.push_str("-- Grand Total --\n\n");
    out.push_str(&format!("  Carbon stock: {:.0} ton C ({:.0} ton CO2e)\n", grand_total_carbon, grand_total_co2));
    out.push_str(&format!("  Per ha: {:.1} ton C/ha ({:.1} ton CO2/ha)\n\n", grand_total_carbon/area_ha, grand_total_co2/area_ha));
    let folu_target = 118_000_000.0;
    out.push_str("-- FOLU Net Sink 2030 --\n");
    out.push_str(&format!("  Contribution: {:.4}% of 118 MTon target\n\n", grand_total_co2 / folu_target * 100.0));
    out.push_str("-- MRV Components --\n");
    out.push_str("  Measurement: Sentinel-1/2 + field inventory\n");
    out.push_str("  Reporting: SRN-PPI; Permen LH 10/2026\n");
    out.push_str("  Verification: VVB + satellite validation\n");
    out.push_str("  Ref: Malerba 2023; Bourgeois 2024; Clough 1989\n");
    out
}
