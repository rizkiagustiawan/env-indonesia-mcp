/// Fugitive Dust Emission Factors (EPA AP-42 Ch.13)
/// Ref: EPA AP-42; WRAP Fugitive Dust Handbook 2006
pub fn assess(road_type: &str, silt_loading_g_m2: f64, silt_content_pct: f64, avg_vehicle_weight_ton: f64, precip_days: u32, vehicle_count: u32, road_length_m: f64) -> String {
    let mut out = String::from("=== Fugitive Dust (EPA AP-42 Ch.13) ===\n");
    out.push_str("Ref: EPA AP-42; WRAP 2006\n\n");
    let (k_pm10, k_pm25) = match road_type.to_lowercase().as_str() {
        "paved" => (4.6, 0.78),
        "unpaved_industrial" => (1.5, 0.15),
        "unpaved_public" => (0.72, 0.072),
        _ => (4.6, 0.78),
    };
    let P = precip_days as f64;
    let N = 365.0;
    let ef_pm10 = if road_type.contains("paved") && !road_type.contains("unpaved") {
        k_pm10 * silt_loading_g_m2.powf(0.91) * avg_vehicle_weight_ton.powf(1.02) * (1.0 - P/(4.0*N)).max(0.0)
    } else if road_type.contains("unpaved_industrial") {
        1.5 * k_pm10 * (silt_content_pct/12.0).powf(0.9) * (avg_vehicle_weight_ton/3.0).powf(0.45)
    } else {
        (k_pm10/0.7) * (silt_content_pct/12.0).powf(0.9) * (avg_vehicle_weight_ton/3.0).powf(0.45) * ((365.0-P)/365.0).max(0.0)
    };
    let ef_pm25 = ef_pm10 * (k_pm25 / k_pm10);
    let daily_pm10 = ef_pm10 * vehicle_count as f64 * road_length_m / 1000.0; // g/m -> kg
    let daily_pm25 = ef_pm25 * vehicle_count as f64 * road_length_m / 1000.0;
    out.push_str(&format!("Road: {} (k_PM10={:.1}, k_PM2.5={:.2})\n", road_type, k_pm10, k_pm25));
    out.push_str(&format!("Silt loading: {:.1} g/m2, Silt content: {:.1}%\n", silt_loading_g_m2, silt_content_pct));
    out.push_str(&format!("Avg weight: {:.1} ton, Precip: {}/365 days\n\n", avg_vehicle_weight_ton, precip_days));
    out.push_str(&format!("  EF PM10: {:.2} g/VKT (g per vehicle-km)\n", ef_pm10));
    out.push_str(&format!("  EF PM2.5: {:.2} g/VKT\n\n", ef_pm25));
    out.push_str(&format!("  >> Daily PM10: {:.1} g/day ({} veh x {:.0}m)\n", daily_pm10, vehicle_count, road_length_m));
    out.push_str(&format!("  >> Daily PM2.5: {:.1} g/day\n\n", daily_pm25));

    // ─── PP 41/1999 / PP 22/2021 AMBIENT COMPLIANCE ───
    out.push_str("─── STATUS KEPATUHAN (PP 22/2021 Lampiran VII — Udara Ambien) ───\n\n");
    let daily_pm10_ug_m3 = daily_pm10 * 1e6 / (road_length_m * 10.0 * 10.0 * 10.0); // approx concentration in 10m×10m×10m volume
    out.push_str(&format!("  PM10 ambien (est): {:.1} µg/m3 → ≤75 µg/m3 (24 jam): {}\n", daily_pm10_ug_m3, if daily_pm10_ug_m3 <= 75.0 {"✅"} else {"❌"}));
    out.push_str(&format!("  PM2.5 ambien (est): {:.1} µg/m3 → ≤55 µg/m3 (24 jam): {}\n\n", daily_pm10_ug_m3 * (k_pm25/k_pm10), if daily_pm10_ug_m3 * (k_pm25/k_pm10) <= 55.0 {"✅"} else {"❌"}));

    out.push_str("─── REKOMENDASI MITIGASI ───\n");
    out.push_str("  1. Watering (penyiraman) — reduksi 70%\n");
    out.push_str("  2. Chemical suppressant — reduksi 90%\n");
    out.push_str("  3. Paving / concrete road — reduksi 95%\n");
    out.push_str("  4. Speed limit ≤ 20 km/jam — reduksi 40%\n");
    out.push_str("  5. Vegetative buffer di roadside\n\n");

    out.push_str("─── PEMANTAUAN (RPL) ───\n");
    out.push_str("  Parameter: PM10, PM2.5, TSP\n");
    out.push_str("  Frekuensi: Bulanan (active road), seasonal (dry season peak)\n");
    out.push_str("  Lokasi: Roadside (≤10m from edge) + receptor (50-100m)\n");
    out.push_str("  Metode: High Volume Sampler (PM10), Beta Attenuation Monitor\n");

    out.push_str("\n─── PELAPORAN & IZIN ───\n");
    out.push_str("  PP 22/2021 Lampiran VII (udara ambien); Pasal 124-131\n");
    out.push_str("  Permen LH 5/2026: Perencanaan mutu udara\n");
    out.push_str("  Amdalnet + OSS; Permen LH 6/2026 (sanksi)\n");

    out.push_str("\n  Ref: EPA AP-42 Ch.13; WRAP 2006; PP 22/2021 Lampiran VII; Permen LH 5/2026\n");
    out
}
