/// Darcy's Law: Q = K × i × A, q = K × i, v = q/n
/// Ref: Freeze & Cherry (1979), Groundwater

pub fn calculate(k_ms: f64, gradient: f64, area_m2: f64, porosity: f64, distance_m: f64) -> String {
    let mut out = String::from("=== Hukum Darcy — Aliran Air Tanah ===\n");
    out.push_str("Ref: Freeze & Cherry (1979), Groundwater\n\n");

    if k_ms <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if gradient <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if area_m2 <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if porosity <= 0.0 || porosity >= 1.0 { return "ERROR: Porositas harus antara 0 dan 1 (eksklusif).".into(); }

    let q_specific = k_ms * gradient; // m/s (specific discharge / Darcy velocity)
    let q_flow = k_ms * gradient * area_m2; // m³/s
    let v_seepage = q_specific / porosity; // m/s (seepage velocity)

    out.push_str(&format!("Input:\n  K (konduktivitas hidraulik) = {:.2e} m/s\n  i (gradien hidraulik) = {:.4}\n  A (luas penampang) = {:.2} m²\n  n (porositas) = {:.2}\n  Jarak analisis = {:.1} m\n\n",
        k_ms, gradient, area_m2, porosity, distance_m));

    out.push_str("Hasil:\n");
    out.push_str(&format!("  Darcy velocity (q) = K × i = {:.2e} m/s = {:.4} m/hari\n", q_specific, q_specific * 86400.0));
    out.push_str(&format!("  Debit (Q) = K × i × A = {:.2e} m³/s = {:.4} m³/hari\n", q_flow, q_flow * 86400.0));
    out.push_str(&format!("  Seepage velocity (v) = q/n = {:.2e} m/s = {:.4} m/hari\n\n", v_seepage, v_seepage * 86400.0));

    // Travel time
    if distance_m > 0.0 {
        let travel_time_s = distance_m / v_seepage;
        let travel_time_d = travel_time_s / 86400.0;
        let travel_time_yr = travel_time_d / 365.25;
        out.push_str(&format!("Waktu tempuh kontaminan sejauh {:.1} m:\n", distance_m));
        if travel_time_yr > 1.0 {
            out.push_str(&format!("  t = {:.1} tahun\n", travel_time_yr));
        } else {
            out.push_str(&format!("  t = {:.1} hari ({:.2} tahun)\n", travel_time_d, travel_time_yr));
        }
    }

    // K lookup table
    out.push_str("\nTabel K untuk jenis tanah Indonesia:\n");
    out.push_str("  Kerikil (gravel)    : K ≈ 1e-1 m/s\n");
    out.push_str("  Pasir (sand)        : K ≈ 1e-3 m/s\n");
    out.push_str("  Pasir berlempung    : K ≈ 1e-5 m/s\n");
    out.push_str("  Lanau (silt)        : K ≈ 1e-6 m/s\n");
    out.push_str("  Lempung (clay)      : K ≈ 1e-9 m/s\n");
    out.push_str("  Gambut (peat)       : K ≈ 1e-5 m/s\n");

    out
}

/// Lookup K by Indonesian soil type name
pub fn k_lookup(soil_type: &str) -> String {
    let k = match soil_type.to_lowercase().as_str() {
        "gravel" | "kerikil" => 1e-1,
        "sand" | "pasir" => 1e-3,
        "silty_sand" | "pasir_berlempung" => 1e-5,
        "silt" | "lanau" => 1e-6,
        "clay" | "lempung" => 1e-9,
        "peat" | "gambut" => 1e-5,
        _ => return format!("ERROR: Jenis tanah '{}' tidak dikenali. Pilihan: gravel, sand, silty_sand, silt, clay, peat.", soil_type),
    };
    format!("K untuk {} = {:.2e} m/s", soil_type, k)
}
