/// Tsunami Travel Time & Run-up Calculator
/// Ref: Shallow water wave theory, Synolakis (1987)

pub fn travel_time(depth_m: f64, distance_km: f64) -> String {
    if depth_m <= 0.0 { return "ERROR: Kedalaman laut harus > 0.".into(); }
    if distance_km <= 0.0 { return "ERROR: Jarak harus > 0.".into(); }

    let g = 9.81_f64;
    let c = (g * depth_m).sqrt(); // m/s
    let c_kmh = c * 3.6;
    let time_hours = distance_km / c_kmh;
    let time_minutes = time_hours * 60.0;

    let mut out = String::from("=== Tsunami Travel Time Calculator ===\n");
    out.push_str("Ref: Shallow water wave theory, c = √(g×d)\n\n");
    out.push_str(&format!("INPUT:\n  Kedalaman laut = {:.0} m\n  Jarak = {:.0} km\n\n", depth_m, distance_km));
    out.push_str(&format!("HASIL:\n  Kecepatan tsunami = {:.1} m/s = {:.0} km/jam\n  Waktu tempuh = {:.1} jam = {:.0} menit\n\n", c, c_kmh, time_hours, time_minutes));

    if time_minutes < 30.0 {
        out.push_str("⚠️ PERINGATAN: Waktu tempuh < 30 menit. Zona SANGAT RAWAN. Evakuasi harus segera setelah gempa.\n");
    }
    out
}

pub fn runup(wave_height_m: f64, depth_m: f64, slope_deg: f64) -> String {
    if wave_height_m <= 0.0 { return "ERROR: Tinggi gelombang harus > 0.".into(); }
    if depth_m <= 0.0 { return "ERROR: Kedalaman harus > 0.".into(); }
    if slope_deg <= 0.0 || slope_deg >= 90.0 { return "ERROR: Kemiringan pantai harus 0-90 derajat.".into(); }

    let beta_rad = slope_deg.to_radians();
    let cot_beta = 1.0 / beta_rad.tan();
    let h_d = wave_height_m / depth_m;

    // Synolakis (1987)
    let r = depth_m * 2.831 * cot_beta.sqrt() * h_d.powf(1.25);

    let mut out = String::from("=== Tsunami Run-up (Synolakis 1987) ===\n");
    out.push_str("⚠️ Untuk pantai landai uniform. Pantai riil lebih kompleks.\n\n");
    out.push_str(&format!("INPUT:\n  H (offshore) = {:.2} m\n  d (kedalaman) = {:.0} m\n  β (slope) = {:.1}°\n\n", wave_height_m, depth_m, slope_deg));
    out.push_str(&format!("HASIL:\n  Run-up (R) ≈ {:.2} m\n", r));
    if r > 5.0 { out.push_str("  ⚠️ Run-up > 5m: Zona inundasi SANGAT LUAS. Evakuasi vertikal diperlukan.\n"); }
    out
}
