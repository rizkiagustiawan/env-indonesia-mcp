/// Anaerobic Digestion / Biogas Reactor Sizing
/// Ref: Rittmann & McCarty (2001), Metcalf & Eddy (2003)
/// Aplikasi: peternakan (sapi, babi, ayam) dan POME Indonesia

pub fn design(
    q_m3d: f64,
    vs_concentration_kgm3: f64,
    vs_destruction_pct: f64,
    temperature_c: f64,
    substrate: &str,
) -> String {
    let mut out = String::from("=== Desain Reaktor Anaerobik (Biogas) ===\n");
    out.push_str("Ref: Rittmann & McCarty (2001), Metcalf & Eddy (2003)\n\n");

    if q_m3d <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if vs_concentration_kgm3 <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if vs_destruction_pct <= 0.0 || vs_destruction_pct > 100.0 {
        return "ERROR: VS destruction harus antara 0-100%.".into();
    }
    if temperature_c < 20.0 || temperature_c > 60.0 {
        return "ERROR: Suhu harus antara 20-60°C.".into();
    }

    let substrate_lower = substrate.to_lowercase();

    // Substrate-specific gas yields (m³ biogas / kg VS destroyed)
    let (gas_yield, typical_vs, desc, min_srt) = match substrate_lower.as_str() {
        "sapi" | "cow" => (0.30, 40.0, "Kotoran Sapi", 15.0),
        "babi" | "pig" => (0.40, 50.0, "Kotoran Babi", 12.0),
        "ayam" | "chicken" => (0.35, 60.0, "Kotoran Ayam", 15.0),
        "pome" => (0.50, 35.0, "Palm Oil Mill Effluent", 20.0),
        "sampah_organik" | "organic_waste" => (0.45, 80.0, "Sampah Organik", 20.0),
        _ => {
            return format!(
            "ERROR: Substrat '{}' tidak dikenali. Pilihan: sapi, babi, ayam, pome, sampah_organik.",
            substrate
        )
        }
    };

    // Temperature regime
    let (regime, design_srt) = if temperature_c >= 50.0 {
        ("Termofilik", min_srt * 0.5)
    } else if temperature_c >= 30.0 {
        ("Mesofilik", min_srt)
    } else {
        ("Psikrofilik", min_srt * 2.0)
    };

    // Reactor volume
    let v_reactor = q_m3d * design_srt;

    // VS loading
    let vs_load_kgd = q_m3d * vs_concentration_kgm3; // kg VS/day
    let olr = vs_load_kgd / v_reactor; // kg VS/m³/day

    // VS destroyed
    let vs_destroyed_kgd = vs_load_kgd * vs_destruction_pct / 100.0;

    // Gas production
    let biogas_m3d = gas_yield * vs_destroyed_kgd;
    let ch4_fraction = 0.60; // 60% CH4 typical
    let ch4_m3d = biogas_m3d * ch4_fraction;

    // Energy potential: 1 m³ CH4 = 9.97 kWh (lower heating value), 35% engine efficiency
    let energy_kwh = ch4_m3d * 9.97 * 0.35;
    let energy_mj = ch4_m3d * 35.8; // 1 m³ CH4 = 35.8 MJ

    // Digestate / sludge
    let vs_remaining_kgd = vs_load_kgd - vs_destroyed_kgd;

    // Dimensions (cylindrical, H/D ≈ 1)
    let pi = std::f64::consts::PI;
    let diameter = (4.0 * v_reactor / (pi * 1.0_f64)).powf(1.0 / 3.0); // H ≈ D
    let height = diameter;

    out.push_str(&format!("Input:\n  Q = {:.1} m³/hari\n  VS = {:.1} kg/m³\n  VS destruction = {:.0}%\n  Suhu = {:.1}°C ({})\n  Substrat = {} ({})\n\n",
        q_m3d, vs_concentration_kgm3, vs_destruction_pct, temperature_c, regime, substrate, desc));

    out.push_str(&format!("Gas yield spesifik ({}):\n  {:.2} m³ biogas / kg VS destroyed\n  VS tipikal: {:.0} kg/m³\n\n",
        substrate, gas_yield, typical_vs));

    out.push_str("Desain reaktor:\n");
    out.push_str(&format!(
        "  SRT = {:.0} hari (min {:.0} hari pada {})\n",
        design_srt, min_srt, regime
    ));
    out.push_str(&format!("  Volume = {:.0} m³\n", v_reactor));
    out.push_str(&format!("  Diameter ≈ {:.1} m\n", diameter));
    out.push_str(&format!("  Tinggi ≈ {:.1} m\n", height));
    out.push_str(&format!("  OLR = {:.2} kg VS/m³/hari", olr));
    if olr > 4.0 {
        out.push_str(" ⚠️ tinggi, risiko asidifikasi\n\n");
    } else if olr < 1.0 {
        out.push_str(" (rendah — reaktor oversized)\n\n");
    } else {
        out.push_str(" ✅\n\n");
    }

    out.push_str("Produksi gas:\n");
    out.push_str(&format!("  VS masuk = {:.1} kg/hari\n", vs_load_kgd));
    out.push_str(&format!(
        "  VS dihancurkan = {:.1} kg/hari\n",
        vs_destroyed_kgd
    ));
    out.push_str(&format!("  Biogas = {:.1} m³/hari\n", biogas_m3d));
    out.push_str(&format!("  CH₄ (60%) = {:.1} m³/hari\n", ch4_m3d));
    out.push_str(&format!(
        "  CO₂ (40%) = {:.1} m³/hari\n\n",
        biogas_m3d - ch4_m3d
    ));

    out.push_str("Potensi energi:\n");
    out.push_str(&format!("  Energi termal = {:.1} MJ/hari\n", energy_mj));
    out.push_str(&format!("  Listrik (η=35%) = {:.1} kWh/hari\n", energy_kwh));
    out.push_str(&format!(
        "  Setara rumah tangga ≈ {:.0} KK (2 kWh/hari)\n\n",
        energy_kwh / 2.0
    ));

    out.push_str(&format!(
        "Digestate:\n  VS sisa = {:.1} kg/hari (pupuk organik)\n",
        vs_remaining_kgd
    ));

    out
}
