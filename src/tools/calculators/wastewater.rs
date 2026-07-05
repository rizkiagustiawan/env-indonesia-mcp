/// Activated Sludge Wastewater Treatment Design
/// Ref: Metcalf & Eddy (2003), Monod kinetics, van't Hoff-Arrhenius

pub fn design(q_m3d: f64, s0_mgl: f64, s_target_mgl: f64, temp_c: f64) -> String {
    if q_m3d <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if s0_mgl <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if s_target_mgl < 0.0 { return "ERROR [E102]: Parameter tidak boleh negatif.".into(); }
    if s_target_mgl >= s0_mgl { return "ERROR: BOD target harus < BOD influent.".into(); }
    if temp_c < 4.0 || temp_c > 40.0 { return format!("ERROR: Suhu {}°C di luar rentang valid (4-40°C) untuk koreksi θ=1.047.", temp_c); }

    // Parameter kinetik (tipikal limbah domestik, 20°C)
    let mu_max_20 = 6.0_f64;  // d⁻¹
    let ks = 60.0;            // mg BOD/L
    let y = 0.6;              // mg VSS/mg BOD
    let kd_20 = 0.06;         // d⁻¹
    let theta_temp = 1.047_f64;

    // Koreksi suhu
    let kd = kd_20 * theta_temp.powf(temp_c - 20.0);
    let mu_max = mu_max_20 * theta_temp.powf(temp_c - 20.0);

    // Design SRT (θc) — pilih 10 hari (konservatif untuk tropis)
    let srt = 10.0_f64; // hari
    let x_mlss = 3500.0; // mg/L MLSS target

    // Volume reaktor
    let v = q_m3d * srt * y * (s0_mgl - s_target_mgl) / (x_mlss * (1.0 + kd * srt));

    // HRT
    let hrt = v / q_m3d * 24.0; // jam

    // Monod effluent substrate
    let s_monod = ks * (1.0 + kd * srt) / (srt * (y * mu_max - kd) - 1.0);

    // Kebutuhan oksigen
    let o2_demand = q_m3d * (s0_mgl - s_target_mgl) / 1000.0 * (1.0 / y - 1.42 * kd * srt / (1.0 + kd * srt));

    // Produksi lumpur
    let y_obs = y / (1.0 + kd * srt);
    let sludge_kg = q_m3d * y_obs * (s0_mgl - s_target_mgl) / 1000.0;

    let mut out = String::from("=== Desain IPAL Activated Sludge ===\n");
    out.push_str("Ref: Metcalf & Eddy (2003), Monod kinetics\n\n");
    out.push_str(&format!("INPUT:\n  Q = {:.0} m³/hari\n  BOD influent = {:.0} mg/L\n  BOD target effluent = {:.0} mg/L\n  Suhu operasi = {:.1}°C\n\n", q_m3d, s0_mgl, s_target_mgl, temp_c));
    out.push_str(&format!("PARAMETER KINETIK (koreksi {}°C, θ=1.047):\n  μmax = {:.2} d⁻¹\n  Ks = {:.0} mg/L\n  Y = {:.2} mg VSS/mg BOD\n  kd = {:.4} d⁻¹\n\n", temp_c, mu_max, ks, y, kd));
    out.push_str(&format!("DESAIN:\n  SRT (θc) = {:.0} hari\n  MLSS = {:.0} mg/L\n  Volume reaktor = {:.1} m³\n  HRT = {:.1} jam\n  Effluent BOD (Monod) = {:.1} mg/L\n\n", srt, x_mlss, v, hrt, s_monod.max(0.0)));
    out.push_str(&format!("OPERASIONAL:\n  Kebutuhan O₂ = {:.1} kg/hari\n  Produksi lumpur (Yobs) = {:.3}\n  Lumpur = {:.1} kg VSS/hari\n\n", o2_demand, y_obs, sludge_kg));

    // Cek baku mutu
    out.push_str("BAKU MUTU (PermenLHK 68/2016 — Limbah Domestik):\n");
    out.push_str(&format!("  BOD effluent {:.0} mg/L {} (maks 30 mg/L)\n", s_target_mgl, if s_target_mgl <= 30.0 { "✅" } else { "❌ MELEBIHI" }));
    out
}
