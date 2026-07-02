/// Ecological Footprint Calculator
/// Ref: Global Footprint Network methodology

pub fn calculate(electricity_kwh: f64, vehicle_km: f64, meat_kg_week: f64, waste_kg_day: f64) -> String {
    // Simplified footprint (global hectares)
    let energy_gha = electricity_kwh * 12.0 * 0.000353; // per year
    let transport_gha = vehicle_km * 52.0 * 0.0000425;
    let food_gha = meat_kg_week * 52.0 * 0.0072;
    let waste_gha = waste_kg_day * 365.0 * 0.000185;
    let total = energy_gha + transport_gha + food_gha + waste_gha;

    let earth_needed = total / 1.63; // biocapacity rata-rata per kapita global

    let mut out = String::from("=== Ecological Footprint ===\n");
    out.push_str(&format!("Energi: {:.3} gha\nTransportasi: {:.3} gha\nPangan: {:.3} gha\nLimbah: {:.3} gha\n\nTOTAL: {:.2} gha/tahun\nButuh {:.1} Bumi\n", energy_gha, transport_gha, food_gha, waste_gha, total, earth_needed));
    if earth_needed > 1.0 { out.push_str("\n⚠️ Gaya hidup ini TIDAK BERKELANJUTAN.\n"); }
    out
}
