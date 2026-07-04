/// Bioretention / Rain Garden Design
/// Ref: PU Cipta Karya, FHWA HEC-22, Prince George's County BMP Manual

fn fmt_rp(v: f64) -> String {
    let s = format!("{:.0}", v.abs());
    let bytes: Vec<u8> = s.bytes().collect();
    let mut result = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 { result.push('.'); }
        result.push(*b as char);
    }
    if v < 0.0 { format!("-{}", result) } else { result }
}

pub fn design(
    q_design_m3s: f64,
    ksat_m_hr: f64,
    ponding_depth_m: f64,
    media_depth_m: f64,
    drain_time_hr: f64,
) -> String {
    if q_design_m3s <= 0.0 { return "ERROR: Debit desain harus > 0 m³/s.".into(); }
    if ksat_m_hr <= 0.0 { return "ERROR: Ksat harus > 0 m/hr.".into(); }
    if ponding_depth_m <= 0.0 { return "ERROR: Kedalaman genangan (ponding) harus > 0 m.".into(); }
    if media_depth_m <= 0.0 { return "ERROR: Kedalaman media harus > 0 m.".into(); }
    if drain_time_hr <= 0.0 { return "ERROR: Waktu drainase harus > 0 jam.".into(); }

    // Design storm volume (simplified: Q × duration assumed 1 hr = 3600s)
    let storm_duration_s = 3600.0; // 1-hour design storm
    let v_runoff = q_design_m3s * storm_duration_s; // m³

    // Surface area: Af = V_runoff / (Ksat × tf + dp)
    let storage_depth = ksat_m_hr * drain_time_hr + ponding_depth_m;
    let af = v_runoff / storage_depth;

    // Alternative Darcy-based: Af = Q × df / (Ksat × (hf + df) × tf)
    let af_darcy = q_design_m3s * 3600.0 * media_depth_m
        / (ksat_m_hr * (ponding_depth_m + media_depth_m) * drain_time_hr);

    // Use larger of two estimates (conservative)
    let af_design = af.max(af_darcy);

    // Media volume
    let media_volume = af_design * media_depth_m;

    // Gravel underdrain layer (typically 0.20-0.30 m)
    let gravel_depth = 0.25;
    let gravel_volume = af_design * gravel_depth;

    // Total excavation depth
    let total_depth = ponding_depth_m + media_depth_m + gravel_depth;
    let excavation_volume = af_design * total_depth;

    // Cost estimate (Indonesia, 2024)
    let cost_media_per_m3 = 350_000.0_f64; // IDR/m³ filter media
    let cost_gravel_per_m3 = 250_000.0;
    let cost_excavation_per_m3 = 80_000.0;
    let cost_plants_per_m2 = 75_000.0;
    let cost_underdrain_per_m = 150_000.0; // perforated PVC per meter

    let underdrain_length = af_design.sqrt() * 2.0; // approximate
    let total_cost = media_volume * cost_media_per_m3
        + gravel_volume * cost_gravel_per_m3
        + excavation_volume * cost_excavation_per_m3
        + af_design * cost_plants_per_m2
        + underdrain_length * cost_underdrain_per_m;

    let cost_per_m2 = total_cost / af_design;

    // Pollutant removal efficiency (typical)
    let removals: &[(&str, &str)] = &[
        ("TSS", "85-95%"),
        ("Total Phosphorus", "50-80%"),
        ("Total Nitrogen", "40-60%"),
        ("Heavy Metals (Zn, Cu, Pb)", "90-98%"),
        ("Bakteri (E. coli)", "70-90%"),
        ("BOD", "60-80%"),
        ("Minyak & Lemak", "85-95%"),
    ];

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  DESAIN BIORETENTION / RAIN GARDEN\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PU Cipta Karya, FHWA HEC-22, PG County BMP Manual\n\n");

    out.push_str(&format!(
        "INPUT:\n  Debit desain (Q)     = {:.4} m³/s ({:.2} L/s)\n  Ksat media           = {:.4} m/hr\n  Ponding depth        = {:.2} m\n  Media depth          = {:.2} m\n  Drain time           = {:.1} jam\n\n",
        q_design_m3s, q_design_m3s * 1000.0, ksat_m_hr, ponding_depth_m, media_depth_m, drain_time_hr
    ));

    out.push_str(&format!(
        "DESAIN HASIL:\n  Volume limpasan (1 jam)  = {:.2} m³\n  Luas permukaan (Af)     = {:.1} m² ({:.3} ha)\n  Volume media filter      = {:.1} m³\n  Volume gravel underdrain = {:.1} m³\n  Total kedalaman galian   = {:.2} m\n  Volume galian total      = {:.1} m³\n\n",
        v_runoff, af_design, af_design / 10000.0, media_volume, gravel_volume, total_depth, excavation_volume
    ));

    out.push_str("SPESIFIKASI MEDIA FILTER:\n");
    out.push_str("  Komposisi: 60-80% pasir, 20-30% kompos, 0-10% topsoil\n");
    out.push_str(&format!("  Kedalaman: {:.2} m\n", media_depth_m));
    out.push_str(&format!("  Ksat desain: {:.4} m/hr\n\n", ksat_m_hr));

    out.push_str("UNDERDRAIN:\n");
    out.push_str("  Pipa: PVC perforasi Ø100 mm\n");
    out.push_str(&format!("  Lapisan gravel: {:.2} m (kerikil 10-20 mm)\n", gravel_depth));
    out.push_str(&format!("  Panjang estimasi: {:.1} m\n\n", underdrain_length));

    out.push_str("TANAMAN YANG DIREKOMENDASIKAN (Indonesia):\n");
    out.push_str("  • Heliconia psittacorum — toleran genangan, estetik\n");
    out.push_str("  • Canna indica — penyerap polutan logam berat\n");
    out.push_str("  • Vetiveria zizanioides — akar dalam, stabilisasi tanah\n");
    out.push_str("  • Cymbopogon nardus — serai wangi, anti nyamuk\n");
    out.push_str("  • Typha angustifolia — cattail, wetland plant\n");
    out.push_str("  • Pandanus amaryllifolius — pandan, adaptif lokal\n\n");

    out.push_str("EFISIENSI PENYISIHAN POLUTAN (tipikal):\n");
    for (pollutant, eff) in removals {
        out.push_str(&format!("  {:30} {}\n", pollutant, eff));
    }

    out.push_str(&format!(
        "\nESTIMASI BIAYA (IDR, 2024):\n  Media filter   : Rp {}\n  Gravel         : Rp {}\n  Galian         : Rp {}\n  Tanaman        : Rp {}\n  Underdrain     : Rp {}\n  ─────────────────────────────\n  TOTAL          : Rp {}\n  Per m²         : Rp {}/m²\n",
        fmt_rp(media_volume * cost_media_per_m3),
        fmt_rp(gravel_volume * cost_gravel_per_m3),
        fmt_rp(excavation_volume * cost_excavation_per_m3),
        fmt_rp(af_design * cost_plants_per_m2),
        fmt_rp(underdrain_length * cost_underdrain_per_m),
        fmt_rp(total_cost),
        fmt_rp(cost_per_m2)
    ));

    if drain_time_hr > 48.0 {
        out.push_str("\n⚠️ Drain time > 48 jam — risiko genangan dan nyamuk. Pertimbangkan Ksat lebih tinggi.\n");
    }
    if ponding_depth_m > 0.30 {
        out.push_str("⚠️ Ponding > 30 cm — pertimbangkan keamanan (safety) di area publik.\n");
    }
    out
}
