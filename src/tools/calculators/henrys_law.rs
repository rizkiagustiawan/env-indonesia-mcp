/// Henry's Law: p = KH × C
/// Ref: Sander (2015), Compilation of Henry's Law Constants

pub fn calculate(compound: &str, concentration_mgl: f64, temperature_c: f64) -> String {
    let mut out = String::from("=== Hukum Henry ===\n");
    out.push_str("Ref: Sander (2015), Compilation of Henry's Law Constants\n\n");

    if concentration_mgl < 0.0 { return "ERROR: Konsentrasi tidak boleh negatif.".into(); }
    if temperature_c < 0.0 || temperature_c > 80.0 { return "ERROR: Suhu harus antara 0-80°C.".into(); }

    let compound_lower = compound.to_lowercase();

    // KH at 25°C (atm·m³/mol) and ΔH/R (K) for temperature correction
    // KH(T) = KH(25) × exp(ΔH/R × (1/298 - 1/T))
    let (kh_25, delta_h_r, mw, name) = match compound_lower.as_str() {
        "benzene" | "benzena" => (5.55e-3, 4100.0, 78.11, "Benzena (C₆H₆)"),
        "toluene" | "toluena" => (6.64e-3, 4000.0, 92.14, "Toluena (C₇H₈)"),
        "tce" | "trikloroetilena" => (9.88e-3, 4700.0, 131.39, "Trikloroetilena (TCE)"),
        "pce" | "tetrakloroetilena" => (1.77e-2, 4900.0, 165.83, "Tetrakloroetilena (PCE)"),
        "chloroform" | "kloroform" => (3.67e-3, 4500.0, 119.38, "Kloroform (CHCl₃)"),
        "methane" | "metana" => (0.668, 1600.0, 16.04, "Metana (CH₄)"),
        "co2" => (0.034, 2400.0, 44.01, "Karbon Dioksida (CO₂)"),
        "o2" | "oksigen" => (769.0, 1700.0, 32.0, "Oksigen (O₂)"),
        "nh3" | "amonia" => (5.7e-4, 4200.0, 17.03, "Amonia (NH₃)"),
        _ => return format!("ERROR: Senyawa '{}' tidak tersedia.\nPilihan: benzene, toluene, tce, pce, chloroform, methane, co2, o2, nh3.", compound),
    };

    // Temperature correction
    let t_k = temperature_c + 273.15;
    let kh_t = kh_25 * (delta_h_r * (1.0 / 298.15 - 1.0 / t_k)).exp();

    // Convert concentration mg/L to mol/m³
    let _c_mol_m3 = concentration_mgl / mw * 1000.0; // mg/L → g/m³ → mol/m³

    // Partial pressure
    let p_atm = kh_t * (concentration_mgl / mw / 1000.0); // mol/L × atm·m³/mol × 1000 L/m³

    // Dimensionless Henry's constant Hcc = KH / (R × T)
    let r = 8.205e-5; // atm·m³/(mol·K)
    let hcc = kh_t / (r * t_k);

    out.push_str(&format!("Input:\n  Senyawa = {}\n  Konsentrasi = {:.2} mg/L\n  Suhu = {:.1}°C ({:.1} K)\n  Berat molekul = {:.2} g/mol\n\n",
        name, concentration_mgl, temperature_c, t_k, mw));

    out.push_str(&format!("Konstanta Henry:\n  KH (25°C) = {:.4e} atm·m³/mol\n  KH ({}°C) = {:.4e} atm·m³/mol\n  Hcc (dimensionless) = {:.4}\n\n",
        kh_25, temperature_c, kh_t, hcc));

    out.push_str(&format!("Tekanan parsial:\n  p = KH × C = {:.4e} atm\n  p = {:.4} Pa\n\n", p_atm, p_atm * 101325.0));

    // Air stripping feasibility
    out.push_str("Kelayakan air stripping:\n");
    if hcc > 0.1 {
        out.push_str("  ✅ Hcc > 0.1 — Sangat layak untuk air stripping\n");
        out.push_str("  Rasio udara/air rendah diperlukan\n");
    } else if hcc > 0.01 {
        out.push_str("  ⚠️ 0.01 < Hcc < 0.1 — Layak dengan packed tower\n");
        out.push_str("  Diperlukan rasio udara/air moderat\n");
    } else if hcc > 0.001 {
        out.push_str("  ⚠️ 0.001 < Hcc < 0.01 — Marginal, perlu evaluasi ekonomi\n");
    } else {
        out.push_str("  ❌ Hcc < 0.001 — Tidak layak untuk air stripping\n");
        out.push_str("  Pertimbangkan: adsorpsi karbon aktif, oksidasi, atau membran\n");
    }

    // KH reference table
    out.push_str("\nTabel KH (25°C) senyawa tersedia:\n");
    out.push_str("  Benzena      : 5.55e-3 atm·m³/mol\n");
    out.push_str("  Toluena      : 6.64e-3\n");
    out.push_str("  TCE          : 9.88e-3\n");
    out.push_str("  PCE          : 1.77e-2\n");
    out.push_str("  Kloroform    : 3.67e-3\n");
    out.push_str("  Metana       : 0.668\n");
    out.push_str("  CO₂          : 0.034\n");
    out.push_str("  O₂           : 769\n");
    out.push_str("  NH₃          : 5.7e-4\n");

    out
}
