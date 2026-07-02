/// Penman-Monteith Reference ET0 (FAO-56, Allen et al. 1998)
/// Simplified daily calculation

pub fn calculate(t_mean: f64, rh_mean: f64, u2: f64, rn: f64) -> String {
    let mut out = String::from("=== Penman-Monteith ET0 (FAO-56) ===\n\n");

    if u2 < 0.0 { return "ERROR: Kecepatan angin tidak boleh negatif.".into(); }
    if rh_mean < 0.0 || rh_mean > 100.0 { return format!("ERROR: RH ({}) harus 0-100%.", rh_mean); }

    let gamma = 0.0665; // kPa/°C at sea level
    let es = 0.6108 * ((17.27 * t_mean) / (t_mean + 237.3)).exp();
    let ea = es * (rh_mean / 100.0);
    let delta = (4098.0 * es) / (t_mean + 237.3).powi(2);
    let g = 0.0; // soil heat flux ≈ 0 for daily

    let et0_num = 0.408 * delta * (rn - g) + gamma * (900.0 / (t_mean + 273.0)) * u2 * (es - ea);
    let et0_den = delta + gamma * (1.0 + 0.34 * u2);
    let et0 = et0_num / et0_den;

    out.push_str(&format!("Input:\n  T mean = {:.1} °C\n  RH mean = {:.1} %\n  u2 (angin 2m) = {:.1} m/s\n  Rn (radiasi netto) = {:.2} MJ/m²/hari\n\n", t_mean, rh_mean, u2, rn));
    out.push_str(&format!("Perhitungan:\n  es (tekanan uap jenuh) = {:.4} kPa\n  ea (tekanan uap aktual) = {:.4} kPa\n  Δ (slope) = {:.4} kPa/°C\n  γ = {:.4} kPa/°C\n\n", es, ea, delta, gamma));
    out.push_str(&format!("ET0 = {:.2} mm/hari\n\n", et0));

    if et0 < 0.0 { out.push_str("⚠️ ET0 negatif — kemungkinan kondisi malam / embun. Set ET0 = 0.\n"); }
    else if et0 > 15.0 { out.push_str("⚠️ ET0 > 15 mm/hari — sangat tinggi, cek input Rn dan u2.\n"); }

    let kategori = if et0 < 3.0 { "Rendah (dataran tinggi/humid)" } else if et0 < 5.5 { "Sedang (tropis humid)" } else if et0 < 8.0 { "Tinggi (tropis kering)" } else { "Sangat Tinggi" };
    out.push_str(&format!("Kategori untuk Indonesia: {}\n", kategori));
    out
}
