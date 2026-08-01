/// Waste to Energy Calculator
/// Ref: EPA, Indonesia PermenLHK

pub fn calculate(waste_ton_day: f64, moisture_pct: f64, organic_pct: f64) -> String {
    if waste_ton_day <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if moisture_pct < 0.0 || moisture_pct > 100.0 {
        return "ERROR: Moisture harus 0-100%.".into();
    }

    let dry_mass = waste_ton_day * (1.0 - moisture_pct / 100.0);
    let calorific_value_mj = dry_mass * 1000.0 * 12.0; // ~12 MJ/kg dry waste (Indonesia typical)
    let electricity_mwh = calorific_value_mj / 3600.0 * 0.25; // 25% efficiency
    let co2_avoided = electricity_mwh * 0.794; // avoided grid emission

    let out = format!("=== Waste to Energy Calculator ===\n\nSampah: {:.0} ton/hari\nMoisture: {:.0}%\nOrganik: {:.0}%\n\nMassa kering: {:.1} ton/hari\nNilai kalori: {:.0} MJ/hari\nListrik (η=25%): {:.1} MWh/hari\nEmisi CO₂ terelak: {:.1} tCO₂/hari\n",
        waste_ton_day, moisture_pct, organic_pct, dry_mass, calorific_value_mj, electricity_mwh, co2_avoided);
    out
}
