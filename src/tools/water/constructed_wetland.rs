/// Constructed Wetland — k-C* Model
/// Ce = C* + (Ci - C*) × exp(-k × t_hyd)
/// Ref: Kadlec & Knight (1996), Treatment Wetlands

pub fn design(q_m3d: f64, parameter: &str, ci_mgl: f64, ce_target: f64, temp_c: f64, wetland_type: &str) -> String {
    let mut out = String::from("=== Desain Constructed Wetland (k-C* Model) ===\n");
    out.push_str("Ref: Kadlec & Knight (1996), Treatment Wetlands\n\n");

    if q_m3d <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if ci_mgl <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if ce_target < 0.0 { return "ERROR [E102]: Parameter tidak boleh negatif.".into(); }
    if temp_c < 5.0 || temp_c > 40.0 { return "ERROR: Suhu harus antara 5-40°C.".into(); }

    let param_lower = parameter.to_lowercase();
    let type_lower = wetland_type.to_lowercase();

    // k values at 20°C (day⁻¹) and C* background concentration
    let (k20, c_star, param_name) = match (param_lower.as_str(), type_lower.as_str()) {
        ("bod", "fws") => (0.678, 3.5 + 0.053 * ci_mgl, "BOD"),
        ("bod", "hssf") => (1.104, 3.5 + 0.053 * ci_mgl, "BOD"),
        ("tss", "fws") => (2.5, 5.1 + 0.16 * ci_mgl, "TSS"),
        ("tss", "hssf") => (5.1, 5.1 + 0.16 * ci_mgl, "TSS"),
        ("nh4n" | "nh4-n" | "amonia", "fws") => (0.2218, 0.0, "NH₄-N"),
        ("nh4n" | "nh4-n" | "amonia", "hssf") => (0.4107, 0.0, "NH₄-N"),
        _ => return format!("ERROR: Kombinasi parameter '{}' dan tipe '{}' tidak dikenali.\nParameter: bod, tss, nh4n\nTipe: fws (Free Water Surface), hssf (Horizontal Subsurface Flow)", parameter, wetland_type),
    };

    // Temperature correction: kT = k20 × θ^(T-20)
    let theta = 1.06_f64;
    let k_t = k20 * theta.powf(temp_c - 20.0);

    // Check if target is achievable (must be > C*)
    if ce_target <= c_star {
        return format!("ERROR [E102]: Parameter harus > C*. target={}, c_star={}", ce_target, c_star);
    }

    // Area sizing: A = Q × ln((Ci-C*)/(Ce-C*)) / k
    // From Ce = C* + (Ci-C*) × exp(-k×t), where t = V/Q and V = A × depth
    let depth = match type_lower.as_str() {
        "fws" => 0.3, // m typical FWS
        "hssf" => 0.6, // m typical HSSF
        _ => 0.4,
    };

    let ln_ratio = ((ci_mgl - c_star) / (ce_target - c_star)).ln();
    let hrt_required = ln_ratio / k_t; // days
    let volume = q_m3d * hrt_required;
    let area = volume / depth;

    // Aspect ratio (L:W typically 3:1 to 4:1)
    let aspect = 3.0;
    let width = (area / aspect).sqrt();
    let length = area / width;

    // Actual effluent
    let ce_actual = c_star + (ci_mgl - c_star) * (-k_t * hrt_required).exp();
    let removal_pct = (1.0 - ce_actual / ci_mgl) * 100.0;

    out.push_str(&format!("Input:\n  Q = {:.0} m³/hari\n  Parameter = {}\n  Ci (influent) = {:.1} mg/L\n  Ce (target) = {:.1} mg/L\n  Suhu = {:.1}°C\n  Tipe wetland = {} ({})\n\n",
        q_m3d, param_name, ci_mgl, ce_target, temp_c,
        wetland_type.to_uppercase(),
        if type_lower == "fws" { "Free Water Surface" } else { "Horizontal Subsurface Flow" }));

    out.push_str(&format!("Parameter model:\n  k₂₀ = {:.4} /hari\n  kT ({}°C) = {:.4} /hari (θ = 1.06)\n  C* = {:.1} mg/L\n\n",
        k20, temp_c, k_t, c_star));

    out.push_str("Desain wetland:\n");
    out.push_str(&format!("  Luas = {:.0} m² ({:.2} ha)\n", area, area / 10000.0));
    out.push_str(&format!("  Kedalaman = {:.1} m\n", depth));
    out.push_str(&format!("  Volume = {:.0} m³\n", volume));
    out.push_str(&format!("  HRT = {:.1} hari\n", hrt_required));
    out.push_str(&format!("  Panjang = {:.1} m\n", length));
    out.push_str(&format!("  Lebar = {:.1} m\n", width));
    out.push_str(&format!("  Rasio L:W = {:.1}:1\n\n", aspect));

    out.push_str(&format!("Effluen prediksi:\n  {} effluent = {:.1} mg/L\n  Removal = {:.1}%\n\n", param_name, ce_actual, removal_pct));

    // Plant species recommendation
    out.push_str("Rekomendasi tanaman (Indonesia tropis):\n");
    out.push_str("  • Typha angustifolia (Cattail) — toleran beban tinggi\n");
    out.push_str("  • Cyperus alternifolius (Payung) — estetis, BOD removal baik\n");
    out.push_str("  • Phragmites karka (Karka) — adaptif di Indonesia\n");
    out.push_str("  • Scirpus grossus (Mensiang) — lokal Indonesia tropis, removal N tinggi\n");

    out
}
