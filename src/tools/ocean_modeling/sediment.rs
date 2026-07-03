/// CERC Longshore Sediment Transport
/// Ref: Shore Protection Manual (1984), USACE

pub fn cerc_transport(hs_m: f64, wave_angle_deg: f64, beach_slope_deg: f64) -> String {
    if hs_m <= 0.0 { return "ERROR: Hs harus > 0.".into(); }
    if wave_angle_deg.abs() > 90.0 { return "ERROR: Wave angle harus -90 to 90 deg.".into(); }

    let g = 9.81_f64;
    let k = 0.39; // SPM coefficient
    let rho_s = 2650.0; // sand density kg/m3
    let rho_w = 1025.0; // seawater density kg/m3
    let porosity = 0.4;

    let alpha_b = wave_angle_deg.to_radians();
    let gamma_b = 0.78; // breaking criterion Hb/db

    // Breaking wave height (shallow water)
    let db = hs_m / gamma_b;

    // Group velocity at breaking (shallow water)
    let cg_b = (g * db).sqrt();

    // Longshore energy flux
    let p_ls = rho_w * g * hs_m * hs_m * cg_b * (2.0 * alpha_b).sin() / 16.0;

    // Volumetric transport rate (m3/year)
    let qs = k * p_ls / ((rho_s - rho_w) * g * (1.0 - porosity));
    let qs_annual = qs * 365.25 * 24.0 * 3600.0; // m3/year

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  CERC Sediment Transport\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Shore Protection Manual (1984), USACE\n");
    out.push_str("⚠️ Simplified 1D longshore transport. Untuk detail gunakan GENESIS/LITPACK.\n\n");
    out.push_str(&format!("INPUT:\n  Hs = {:.2} m\n  Wave angle = {:.1}°\n  Breaking depth = {:.2} m\n\n", hs_m, wave_angle_deg, db));
    out.push_str(&format!("HASIL:\n  Longshore energy flux (Pls) = {:.1} W/m\n  Transport rate = {:.4} m³/s\n  Transport rate = {:.0} m³/year\n\n", p_ls, qs, qs_annual));

    let direction = if wave_angle_deg > 0.0 { "ke KANAN (melihat ke laut)" } else { "ke KIRI (melihat ke laut)" };
    out.push_str(&format!("Arah transport: {}\n", direction));
    if qs_annual.abs() > 100000.0 {
        out.push_str("⚠️ Transport > 100,000 m³/tahun — risiko erosi pantai TINGGI.\n");
    }
    out
}
