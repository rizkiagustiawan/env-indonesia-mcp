/// Rainwater Harvesting Calculator
/// Ref: Rational method, SNI 8456:2017

pub fn calculate(
    roof_area_m2: f64,
    rainfall_mm: f64,
    runoff_coeff: f64,
    demand_liters_day: f64,
) -> String {
    if roof_area_m2 <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if runoff_coeff < 0.0 || runoff_coeff > 1.0 {
        return format!("ERROR: Koefisien limpasan {} harus 0-1.", runoff_coeff);
    }

    let supply_liters = roof_area_m2 * rainfall_mm * runoff_coeff;
    let days_supply = if demand_liters_day > 0.0 {
        supply_liters / demand_liters_day
    } else {
        0.0
    };

    let mut out = String::from("=== Rainwater Harvesting Calculator ===\n");
    out.push_str(&format!("Luas atap: {:.0} m²\nCurah hujan: {:.0} mm/bulan\nKoef. runoff: {:.2}\nKebutuhan: {:.0} L/hari\n\n", roof_area_m2, rainfall_mm, runoff_coeff, demand_liters_day));
    out.push_str(&format!(
        "Supply = {:.0} liter/bulan\nCukup untuk {:.0} hari\nRekomendasi tangki: {:.0} liter\n",
        supply_liters,
        days_supply,
        supply_liters * 0.8
    ));
    out
}
