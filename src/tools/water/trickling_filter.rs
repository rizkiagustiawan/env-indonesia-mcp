/// Trickling Filter — NRC Equation
/// E = 100 / (1 + 0.4432 × √(W/(V×F)))
/// Ref: NRC (National Research Council, 1946); Metcalf & Eddy (2003)

pub fn design(
    q_m3d: f64,
    bod_in: f64,
    bod_target: f64,
    media_depth_m: f64,
    recirculation_ratio: f64,
) -> String {
    let mut out = String::from("=== Desain Trickling Filter (NRC) ===\n");
    out.push_str("Ref: NRC (1946), Metcalf & Eddy (2003)\n\n");

    if q_m3d <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if bod_in <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if bod_target < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }
    if bod_target >= bod_in {
        return "ERROR: BOD target harus < BOD influent.".into();
    }
    if media_depth_m < 1.0 || media_depth_m > 3.0 {
        return "ERROR: Kedalaman media 1-3 m (tipikal).".into();
    }
    if recirculation_ratio < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }

    let target_efficiency = (1.0 - bod_target / bod_in) * 100.0;

    // Recirculation factor
    let r = recirculation_ratio;
    let f = (1.0 + r) / (1.0 + 0.1 * r).powi(2);

    // BOD load
    let w = q_m3d * bod_in / 1000.0; // kg BOD/day

    // Required efficiency
    let e_required = target_efficiency / 100.0;

    // From NRC: E = 1 / (1 + 0.4432 × √(W/(V×F)))
    // Solve for V: 0.4432 × √(W/(V×F)) = (1/E - 1)
    // √(W/(V×F)) = (1/E - 1) / 0.4432
    // W/(V×F) = ((1/E - 1) / 0.4432)²
    // V = W / (F × ((1/E - 1) / 0.4432)²)

    let ratio = (1.0 / e_required - 1.0) / 0.4432;
    let v_required = w / (f * ratio * ratio);

    // Dimensions
    let area = v_required / media_depth_m;
    let diameter = (4.0 * area / std::f64::consts::PI).sqrt();

    // Actual efficiency check
    let e_actual = 1.0 / (1.0 + 0.4432 * (w / (v_required * f)).sqrt());

    // Hydraulic loading rate
    let q_total = q_m3d * (1.0 + r); // including recirculation
    let hlr = q_total / area; // m³/m²/day

    // Organic loading rate
    let olr = w / v_required; // kg BOD/m³/day

    // Effluent BOD
    let bod_eff = bod_in * (1.0 - e_actual);

    out.push_str(&format!("Input:\n  Q = {:.0} m³/hari\n  BOD influent = {:.0} mg/L\n  BOD target = {:.0} mg/L\n  Kedalaman media = {:.1} m\n  Rasio resirkulasi (R) = {:.1}\n\n",
        q_m3d, bod_in, bod_target, media_depth_m, r));

    out.push_str(&format!(
        "Perhitungan:\n  Faktor resirkulasi (F) = (1+R)/(1+0.1R)² = {:.3}\n",
        f
    ));
    out.push_str(&format!("  Beban BOD (W) = {:.1} kg/hari\n", w));
    out.push_str(&format!(
        "  Efisiensi target = {:.1}%\n\n",
        target_efficiency
    ));

    out.push_str("Desain filter:\n");
    out.push_str(&format!("  Volume filter = {:.1} m³\n", v_required));
    out.push_str(&format!("  Luas permukaan = {:.1} m²\n", area));
    out.push_str(&format!("  Diameter = {:.1} m\n", diameter));
    out.push_str(&format!("  Kedalaman media = {:.1} m\n\n", media_depth_m));

    out.push_str("Parameter operasional:\n");
    out.push_str(&format!("  Hydraulic loading = {:.1} m³/m²/hari", hlr));
    if hlr > 40.0 {
        out.push_str(" (high rate)\n");
    } else if hlr > 4.0 {
        out.push_str(" (standard rate)\n");
    } else {
        out.push_str(" (low rate)\n");
    }
    out.push_str(&format!("  Organic loading = {:.2} kg BOD/m³/hari", olr));
    if olr > 1.6 {
        out.push_str(" ⚠️ tinggi\n");
    } else {
        out.push_str(" ✅\n");
    }
    out.push_str(&format!("  Efisiensi (NRC) = {:.1}%\n", e_actual * 100.0));
    out.push_str(&format!("  BOD effluent = {:.1} mg/L\n", bod_eff));

    out.push_str("\n─── STATUS KEPATUHAN EFLUEN ───\n\n");
    let bod_ok = bod_eff <= 30.0;
    out.push_str(&format!("  BOD: {:.1} mg/L → ≤30 mg/L (domestik): {}\n", bod_eff, if bod_ok {"✅ MEMENUHI"} else {"❌ MELEBIHI"}));
    
    if !bod_ok {
        out.push_str("\n─── REKOMENDASI MITIGASI ───\n");
        out.push_str("  1. Tingkatkan recirculation ratio (R)\n");
        out.push_str("  2. Tambah secondary clarifier / polishing step\n");
        out.push_str("  3. Kurangi hydraulic loading rate\n");
    }

    out.push_str("\n─── PEMANTAUAN & PELAPORAN ───\n");
    out.push_str("  Permen LH/BPLH 11/2025: Baku mutu air limbah domestik\n");
    out.push_str("  Parameter: BOD, TSS, pH\n");

    out
}
