/// Peatland Subsidence & CO2 Emission Calculator
/// Ref: Hooijer et al. (2012) Biogeosciences

pub fn calculate(water_table_depth_cm: f64, area_ha: f64, years: u32) -> String {
    if water_table_depth_cm < 0.0 { return "ERROR: Kedalaman muka air tanah tidak boleh negatif.".into(); }
    if area_ha <= 0.0 { return "ERROR: Luas area harus > 0.".into(); }

    // Hooijer (2012) model
    let subsidence_rate_cm_yr = if water_table_depth_cm <= 40.0 { water_table_depth_cm * 0.04 + 1.0 } else { 0.9 * (water_table_depth_cm / 10.0) };
    let co2_per_ha_yr = 0.91 * water_table_depth_cm; // tCO2/ha/tahun per cm WTD
    let total_co2_yr = co2_per_ha_yr * area_ha;
    let total_co2 = total_co2_yr * (years as f64);
    let carbon_value_idr = total_co2 * 465000.0;

    let mut out = String::from("=== Peatland Subsidence & CO₂ Emission ===\n");
    out.push_str("Ref: Hooijer et al. (2012), Biogeosciences\n");
    out.push_str("⚠️ Model dikalibrasi untuk gambut Riau/Kalimantan.\n\n");
    out.push_str(&format!("INPUT:\n  Kedalaman muka air tanah (WTD) = {:.0} cm\n  Luas area = {:.1} ha\n  Periode = {} tahun\n\n", water_table_depth_cm, area_ha, years));
    out.push_str(&format!("HASIL:\n  Laju subsidensi ≈ {:.1} cm/tahun\n  Emisi CO₂ = {:.1} tCO₂/ha/tahun\n  Total emisi/tahun = {:.0} tCO₂/tahun\n  Total emisi ({} tahun) = {:.0} tCO₂\n  Nilai karbon (NEK Rp465.000/tCO₂) = Rp {:.0}\n\n", subsidence_rate_cm_yr, co2_per_ha_yr, total_co2_yr, years, total_co2, carbon_value_idr));
    out.push_str("REKOMENDASI:\n");
    if water_table_depth_cm > 40.0 {
        out.push_str("  ⚠️ WTD > 40 cm: Risiko TINGGI subsidensi dan emisi.\n");
        out.push_str("  → Rewetting (menaikkan muka air tanah) ke WTD < 40 cm.\n");
        out.push_str("  → Paludikultur (pertanian lahan basah) sebagai alternatif drainase.\n");
    }
    out
}
