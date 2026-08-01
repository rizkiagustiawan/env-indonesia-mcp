/// UASB (Upflow Anaerobic Sludge Blanket) Reactor Design
/// OLR = Q × COD_in / V, HRT = V / Q
/// Ref: Lettinga et al. (1980), van Haandel & Lettinga (1994)

pub fn design(
    q_m3d: f64,
    cod_in_mgl: f64,
    cod_eff_target: f64,
    temperature_c: f64,
    waste_type: &str,
) -> String {
    let mut out = String::from("=== Desain Reaktor UASB ===\n");
    out.push_str("Ref: Lettinga et al. (1980), van Haandel & Lettinga (1994)\n\n");

    if q_m3d <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if cod_in_mgl <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if cod_eff_target < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }
    if cod_eff_target >= cod_in_mgl {
        return "ERROR: COD target harus < COD influent.".into();
    }
    if temperature_c < 15.0 || temperature_c > 40.0 {
        return "ERROR: Suhu harus antara 15-40°C untuk UASB.".into();
    }

    let waste_lower = waste_type.to_lowercase();

    // Waste-specific parameters: (typical COD mg/L, design OLR kg COD/m³/d, HRT hours)
    let (typical_cod, design_olr, design_hrt_hr, desc) = match waste_lower.as_str() {
        "pome" => (50000.0, 12.0, 16.0, "Palm Oil Mill Effluent"),
        "tapioka" | "tapioca" => (15000.0, 8.0, 12.0, "Limbah Tapioka"),
        "karet" | "rubber" => (5000.0, 6.0, 8.0, "Limbah Karet"),
        "domestik" | "domestic" => (500.0, 4.0, 6.0, "Limbah Domestik"),
        "tahu" | "tofu" => (8000.0, 7.0, 10.0, "Limbah Tahu"),
        _ => (cod_in_mgl, 6.0, 8.0, "Custom"),
    };

    // Reactor volume based on OLR
    let cod_load_kgd = q_m3d * cod_in_mgl / 1000.0; // kg COD/day
    let v_olr = cod_load_kgd / design_olr;

    // Reactor volume based on HRT
    let v_hrt = q_m3d * design_hrt_hr / 24.0;

    // Use the larger volume
    let v_design = v_olr.max(v_hrt);
    let actual_olr = cod_load_kgd / v_design;
    let actual_hrt = v_design / q_m3d * 24.0;

    // Dimensions (height 4-6m)
    let height = 5.0; // m
    let area = v_design / height;
    let diameter = (4.0 * area / std::f64::consts::PI).sqrt();

    // Upflow velocity
    let v_up = q_m3d / (area * 24.0); // m/hr

    // COD removal
    let cod_removed = cod_in_mgl - cod_eff_target;
    let removal_efficiency = cod_removed / cod_in_mgl * 100.0;
    let cod_removed_kgd = q_m3d * cod_removed / 1000.0;

    // Biogas production: 0.35 m³ CH4 / kg COD removed (at STP)
    let ch4_production = 0.35 * cod_removed_kgd; // m³ CH4/day
    let biogas_production = ch4_production / 0.65; // biogas ≈ 65% CH4

    // Temperature correction for gas volume
    let temp_factor = (273.15 + temperature_c) / 273.15;
    let ch4_actual = ch4_production * temp_factor;

    // Sludge production (0.05-0.15 kg VSS/kg COD removed for anaerobic)
    let sludge_kgd = 0.10 * cod_removed_kgd;

    out.push_str(&format!("Input:\n  Q = {:.0} m³/hari\n  COD influent = {:.0} mg/L\n  COD effluent target = {:.0} mg/L\n  Suhu = {:.1}°C\n  Jenis limbah = {} ({})\n",
        q_m3d, cod_in_mgl, cod_eff_target, temperature_c, waste_type, desc));
    if typical_cod != cod_in_mgl {
        out.push_str(&format!(
            "  COD tipikal untuk {} = {:.0} mg/L\n",
            waste_type, typical_cod
        ));
    }
    out.push_str("\n");

    out.push_str(&format!("Beban COD:\n  Beban harian = {:.1} kg COD/hari\n  COD removal = {:.0} mg/L ({:.1}%)\n  COD removal = {:.1} kg/hari\n\n",
        cod_load_kgd, cod_removed, removal_efficiency, cod_removed_kgd));

    out.push_str("Desain reaktor:\n");
    out.push_str(&format!("  Volume reaktor = {:.1} m³\n", v_design));
    out.push_str(&format!(
        "  OLR aktual = {:.1} kg COD/m³/hari (desain: ≤{:.0})\n",
        actual_olr, design_olr
    ));
    out.push_str(&format!(
        "  HRT aktual = {:.1} jam (desain: ≥{:.0})\n",
        actual_hrt, design_hrt_hr
    ));
    out.push_str(&format!("  Tinggi = {:.1} m\n", height));
    out.push_str(&format!("  Luas penampang = {:.1} m²\n", area));
    out.push_str(&format!("  Diameter (jika silinder) = {:.1} m\n", diameter));
    out.push_str(&format!(
        "  Upflow velocity = {:.2} m/jam {}\n\n",
        v_up,
        if v_up <= 0.7 {
            "✅"
        } else {
            "❌ > 0.7 m/jam — risiko washout!"
        }
    ));

    out.push_str("Produksi biogas:\n");
    out.push_str(&format!("  CH₄ = {:.1} m³/hari (STP)\n", ch4_production));
    out.push_str(&format!(
        "  CH₄ = {:.1} m³/hari (pada {}°C)\n",
        ch4_actual, temperature_c
    ));
    out.push_str(&format!(
        "  Biogas total (65% CH₄) = {:.1} m³/hari\n\n",
        biogas_production
    ));

    out.push_str(&format!(
        "Produksi lumpur:\n  Lumpur = {:.1} kg VSS/hari (Yobs = 0.10)\n\n",
        sludge_kgd
    ));

    // Energy potential
    let energy_kwh = ch4_production * 9.97 * 0.35; // 1 m³ CH4 = 9.97 kWh, 35% efficiency
    out.push_str(&format!(
        "Potensi energi listrik:\n  Energi = {:.1} kWh/hari (efisiensi genset 35%)\n",
        energy_kwh
    ));

    out
}
