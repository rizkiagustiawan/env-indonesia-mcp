/// Rational Method: Q = C × I × A / 360
/// Q in m³/s, I in mm/hr, A in ha
/// Ref: Kuichling (1889), Suripin (2004) Drainase Perkotaan

pub fn calculate(c_coeff: f64, i_mm_hr: f64, a_ha: f64, land_use: &str) -> String {
    let mut out = String::from("=== Metode Rasional — Debit Puncak ===\n");
    out.push_str("Ref: Kuichling (1889), Suripin (2004)\n\n");

    if i_mm_hr <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if a_ha <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    // Lookup C from land use if c_coeff is 0
    let c = if c_coeff > 0.0 {
        if c_coeff > 1.0 { return "ERROR: Koefisien limpasan (C) harus ≤ 1.".into(); }
        c_coeff
    } else {
        let land_lower = land_use.to_lowercase();
        match land_lower.as_str() {
            "hutan" | "forest" => 0.10,
            "sawah" | "paddy" => 0.15,
            "perkebunan" | "plantation" => 0.20,
            "taman" | "park" => 0.25,
            "permukiman_jarang" | "low_density" => 0.40,
            "jalan_tanah" | "dirt_road" => 0.45,
            "permukiman_padat" | "high_density" => 0.70,
            "industri" | "industrial" => 0.75,
            "komersial" | "commercial" => 0.80,
            "jalan_aspal" | "paved_road" => 0.85,
            _ => return format!("ERROR: Tata guna lahan '{}' tidak dikenali.\nPilihan: hutan, sawah, perkebunan, taman, permukiman_jarang, jalan_tanah, permukiman_padat, industri, komersial, jalan_aspal.", land_use),
        }
    };

    // Q = C × I × A / 360
    let q_m3s = c * i_mm_hr * a_ha / 360.0;
    let q_ls = q_m3s * 1000.0;

    out.push_str(&format!("Input:\n  C (koefisien limpasan) = {:.2}", c));
    if c_coeff <= 0.0 {
        out.push_str(&format!(" ({})", land_use));
    }
    out.push_str(&format!("\n  I (intensitas hujan) = {:.1} mm/jam\n  A (luas DAS) = {:.2} ha\n\n", i_mm_hr, a_ha));

    out.push_str("Perhitungan:\n");
    out.push_str(&format!("  Q = C × I × A / 360\n"));
    out.push_str(&format!("  Q = {:.2} × {:.1} × {:.2} / 360\n", c, i_mm_hr, a_ha));
    out.push_str(&format!("  Q = {:.4} m³/s ({:.1} L/s)\n\n", q_m3s, q_ls));

    out.push_str(&format!("Kapasitas drainase yang direkomendasikan: ≥ {:.1} L/s\n\n", q_ls * 1.2));

    // Reference table
    out.push_str("Koefisien limpasan (C) tata guna lahan Indonesia:\n");
    out.push_str("  Hutan              : 0.10\n");
    out.push_str("  Sawah              : 0.15\n");
    out.push_str("  Perkebunan         : 0.20\n");
    out.push_str("  Taman              : 0.25\n");
    out.push_str("  Permukiman jarang  : 0.40\n");
    out.push_str("  Jalan tanah        : 0.45\n");
    out.push_str("  Permukiman padat   : 0.70\n");
    out.push_str("  Industri           : 0.75\n");
    out.push_str("  Komersial          : 0.80\n");
    out.push_str("  Jalan aspal        : 0.85\n");

    out
}
