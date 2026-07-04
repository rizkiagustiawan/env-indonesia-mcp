/// Nernst Equation: E = E° - (RT/nF) × ln(Q)
/// Ref: Stumm & Morgan (1996), Aquatic Chemistry

pub fn calculate(half_reaction: &str, temperature_c: f64, log_q: f64, n_electrons: u32) -> String {
    let mut out = String::from("=== Persamaan Nernst — Potensial Redoks ===\n");
    out.push_str("Ref: Stumm & Morgan (1996), Aquatic Chemistry\n\n");

    if temperature_c < 0.0 || temperature_c > 100.0 { return "ERROR: Suhu harus antara 0-100°C.".into(); }
    if n_electrons == 0 { return "ERROR: Jumlah elektron (n) harus > 0.".into(); }

    let reaction_lower = half_reaction.to_lowercase();

    // Standard potentials E° (V) and descriptions
    let (e_standard, n_default, description) = match reaction_lower.as_str() {
        "o2/h2o" | "oksigen" => (1.23, 4, "O₂ + 4H⁺ + 4e⁻ → 2H₂O"),
        "fe3+/fe2+" | "besi" => (0.77, 1, "Fe³⁺ + e⁻ → Fe²⁺"),
        "mno4-/mn2+" | "permanganat" => (1.51, 5, "MnO₄⁻ + 8H⁺ + 5e⁻ → Mn²⁺ + 4H₂O"),
        "cr2o72-/cr3+" | "dikromat" => (1.33, 6, "Cr₂O₇²⁻ + 14H⁺ + 6e⁻ → 2Cr³⁺ + 7H₂O"),
        "no3-/n2" | "nitrat" => (0.74, 10, "2NO₃⁻ + 12H⁺ + 10e⁻ → N₂ + 6H₂O"),
        "no3-/nh4+" | "nitrat_ammonium" => (0.88, 8, "NO₃⁻ + 10H⁺ + 8e⁻ → NH₄⁺ + 3H₂O"),
        "so42-/h2s" | "sulfat" => (-0.22, 8, "SO₄²⁻ + 10H⁺ + 8e⁻ → H₂S + 4H₂O"),
        "co2/ch4" | "metanogenesis" => (-0.24, 8, "CO₂ + 8H⁺ + 8e⁻ → CH₄ + 2H₂O"),
        _ => return format!("ERROR: Reaksi '{}' tidak dikenali.\nPilihan: o2/h2o, fe3+/fe2+, mno4-/mn2+, cr2o72-/cr3+, no3-/n2, no3-/nh4+, so42-/h2s, co2/ch4", half_reaction),
    };

    let n = if n_electrons > 0 { n_electrons } else { n_default as u32 };

    // Nernst equation: E = E° - (RT/nF) × ln(Q) = E° - (2.303RT/nF) × log(Q)
    let r = 8.314; // J/(mol·K)
    let f = 96485.0; // C/mol
    let t_k = temperature_c + 273.15;

    let nernst_factor = 2.303 * r * t_k / (n as f64 * f);
    let e = e_standard - nernst_factor * log_q;

    // Gibbs free energy
    let delta_g = -(n as f64) * f * e / 1000.0; // kJ/mol

    out.push_str(&format!("Setengah reaksi: {}\n", description));
    out.push_str(&format!("E° = {:.3} V (standar)\n\n", e_standard));

    out.push_str(&format!("Input:\n  Suhu = {:.1}°C ({:.1} K)\n  log(Q) = {:.2}\n  n (elektron) = {}\n\n", temperature_c, t_k, log_q, n));

    out.push_str("Perhitungan:\n");
    out.push_str(&format!("  Faktor Nernst (2.303RT/nF) = {:.4} V\n", nernst_factor));
    out.push_str(&format!("  E = E° - (2.303RT/nF) × log(Q)\n"));
    out.push_str(&format!("  E = {:.3} - {:.4} × {:.2}\n", e_standard, nernst_factor, log_q));
    out.push_str(&format!("  E = {:.4} V\n\n", e));

    out.push_str(&format!("Energi bebas Gibbs:\n  ΔG = -nFE = {:.2} kJ/mol\n", delta_g));
    if delta_g < 0.0 {
        out.push_str("  → Reaksi spontan (ΔG < 0) ✅\n\n");
    } else {
        out.push_str("  → Reaksi tidak spontan (ΔG > 0) — perlu input energi\n\n");
    }

    // Reference table
    out.push_str("Tabel potensial standar (E°):\n");
    out.push_str("  O₂/H₂O         : +1.23 V (oksik)\n");
    out.push_str("  MnO₄⁻/Mn²⁺     : +1.51 V\n");
    out.push_str("  Cr₂O₇²⁻/Cr³⁺   : +1.33 V\n");
    out.push_str("  NO₃⁻/NH₄⁺      : +0.88 V\n");
    out.push_str("  Fe³⁺/Fe²⁺      : +0.77 V\n");
    out.push_str("  NO₃⁻/N₂        : +0.74 V\n");
    out.push_str("  SO₄²⁻/H₂S      : -0.22 V (anoksik)\n");
    out.push_str("  CO₂/CH₄        : -0.24 V (metanogenik)\n");

    out
}
