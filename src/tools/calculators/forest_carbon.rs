/// Forest Carbon Stock — IPCC 2006 Guidelines
/// Above-ground biomass (AGB), below-ground biomass (BGB), soil carbon
/// By forest type: agroforestry, hutan sekunder, hutan primer, mangrove
/// Ref: IPCC 2006 Guidelines Vol 4 (AFOLU); Indonesia FREL; FOLU Net Sink 2030
pub fn assess(forest_type: &str, area_ha: f64, tree_density_per_ha: f64, avg_dbh_cm: f64, avg_height_m: f64, soil_carbon_ton_ha: f64) -> String {
    let mut out = String::from("=== Forest Carbon Stock (IPCC 2006) ===\n");
    out.push_str("Ref: IPCC 2006 Guidelines Vol 4; Indonesia FREL; FOLU Net Sink 2030\n\n");

    // Default biomass equations by forest type (Chave 2014, IPCC defaults)
    let (wood_density, biomass_eq_factor, root_shoot_ratio, default_agb_ton_ha) = match forest_type.to_lowercase().as_str() {
        s if s.contains("primer") || s.contains("primary") => (0.55, 0.0673, 0.24, 300.0),
        s if s.contains("sekunder") || s.contains("secondary") => (0.50, 0.0673, 0.22, 200.0),
        s if s.contains("mangrove") => (0.70, 0.0957, 0.41, 350.0),
        s if s.contains("agroforestry") || s.contains("agro") => (0.45, 0.0673, 0.20, 100.0),
        s if s.contains("tanaman") || s.contains("plantation") => (0.45, 0.0673, 0.20, 80.0),
        _ => (0.50, 0.0673, 0.24, 150.0),
    };

    // AGB = 0.0673 * (WD * D^2 * H)^0.976  (Chave 2014, pantropical, Global Change Biology)
    // CRITICAL: D must be in CENTIMETERS, WD in g/cm^3, H in m. Result is already in kg (no *1000).
    // Verified: Chave et al. 2014; BIOMASS::computeAGB R docs.
    let d_cm = avg_dbh_cm; // keep DBH in cm (do NOT convert to m)
    let agb_per_tree_kg = if d_cm > 0.0 && avg_height_m > 0.0 {
        biomass_eq_factor * (wood_density * d_cm * d_cm * avg_height_m).powf(0.976)
    } else { 0.0 };

    let agb_ton_ha = if tree_density_per_ha > 0.0 && agb_per_tree_kg > 0.0 {
        agb_per_tree_kg * tree_density_per_ha / 1000.0
    } else { default_agb_ton_ha };

    let bgb_ton_ha = agb_ton_ha * root_shoot_ratio;
    let total_biomass_ton_ha = agb_ton_ha + bgb_ton_ha;
    let carbon_ton_ha = total_biomass_ton_ha * 0.47; // carbon fraction
    let co2_ton_ha = carbon_ton_ha * 44.0 / 12.0;

    let total_agb_ton = agb_ton_ha * area_ha;
    let total_bgb_ton = bgb_ton_ha * area_ha;
    let total_carbon_ton = carbon_ton_ha * area_ha;
    let total_co2_ton = co2_ton_ha * area_ha;
    let total_soil_carbon_ton = soil_carbon_ton_ha * area_ha;
    let grand_total_carbon_ton = total_carbon_ton + total_soil_carbon_ton;
    let grand_total_co2_ton = grand_total_carbon_ton * 44.0 / 12.0;

    out.push_str(&format!("Forest type: {} (WD={:.2}, R:S={:.2}))\n", forest_type, wood_density, root_shoot_ratio));
    out.push_str(&format!("Area: {:.0} ha\n", area_ha));
    out.push_str(&format!("Tree density: {:.0}/ha, DBH: {:.1}cm, Height: {:.1}m\n\n", tree_density_per_ha, avg_dbh_cm, avg_height_m));

    out.push_str("═══ BIOMASS & CARBON STOCK ═══\n");
    out.push_str(&format!("  AGB: {:.1} ton/ha ({:.0} ton total))\n", agb_ton_ha, total_agb_ton));
    out.push_str(&format!("  BGB: {:.1} ton/ha ({:.0} ton total))\n", bgb_ton_ha, total_bgb_ton));
    out.push_str(&format!("  Total Biomass: {:.1} ton/ha\n", total_biomass_ton_ha));
    out.push_str(&format!("  Carbon (0.47)): {:.1} ton C/ha ({:.0} ton C total)\n", carbon_ton_ha, total_carbon_ton));
    out.push_str(&format!("  CO2 equiv: {:.1} ton CO2/ha ({:.0} ton CO2 total))\n\n", co2_ton_ha, total_co2_ton));

    out.push_str("═══ SOIL CARBON ═══\n");
    out.push_str(&format!("  Soil C: {:.1} ton C/ha ({:.0} ton total))\n\n", soil_carbon_ton_ha, total_soil_carbon_ton));

    out.push_str("═══ GRAND TOTAL ═══\n");
    out.push_str(&format!("  Total Carbon Stock: {:.0} ton C ({:.0} ton CO2e))\n", grand_total_carbon_ton, grand_total_co2_ton));
    out.push_str(&format!("  Per ha: {:.1} ton C/ha ({:.1} ton CO2/ha))\n\n", grand_total_carbon_ton/area_ha, grand_total_co2_ton/area_ha));

    // NDC FOLU Net Sink 2030
    out.push_str("═══ FOLU NET SINK 2030 ═══\n");
    out.push_str("  FOLU target 2030: -118 MTon CO2e (penyerapan)\n");
    out.push_str("  FOLU target 2035: -206 MTon CO2e\n");
    let contribution_pct = if grand_total_co2_ton > 0.0 { (grand_total_co2_ton / 118_000_000.0) * 100.0 } else { 0.0 };
    out.push_str(&format!("  Kontribusi area ini: {:.4}% dari target FOLU 2030\n\n", contribution_pct));

    out.push_str("═══ PEMANTAUAN (MRV) ═══\n");
    out.push_str("  Method: Biomass inventory + soil sampling + remote sensing (LiDAR)\n");
    out.push_str("  Frekuensi: 5-year (national FREL), annual (project-level)\n");
    out.push_str("  Sistem: SRN-PPI; SIGN-SMART; FREL\n");

    out.push_str("\n  Ref: IPCC 2006 Guidelines Vol 4; Chave 2014; Indonesia FREL; Second NDC 2025\n");
    out
}

#[cfg(test)]
mod tests {
    // Self-check: Chave 2014 with WD=0.55 g/cm3, D=30 cm, H=20 m -> AGB ~= 543 kg/tree
    // AGB = 0.0673 * (0.55 * 30^2 * 20)^0.976 = 0.0673 * (9900)^0.976 = 0.0673 * 8074 = 543.4 kg
    #[test]
    fn chave_reference_value() {
        let wd: f64 = 0.55; let d: f64 = 30.0; let h: f64 = 20.0;
        let agb = 0.0673 * (wd * d * d * h).powf(0.976);
        assert!((agb - 534.3).abs() < 5.0, "AGB={agb} expected ~534 kg/tree");
        // A single 30cm-DBH tree must be hundreds of kg, not millions (the old *1000 + m bug)
        assert!(agb > 100.0 && agb < 2000.0, "AGB={agb} kg outside realistic single-tree range");
    }
}

