/// Bruun Rule Coastal Erosion Prediction
/// Ref: Bruun (1962), Dean (1991), IPCC AR6 WG1

pub fn calculate(sea_level_rise_m: f64, profile_length_m: f64, berm_height_m: f64, closure_depth_m: f64) -> String {
    if sea_level_rise_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0 m.".into(); }
    if profile_length_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0 m.".into(); }
    if berm_height_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0 m (di atas MSL).".into(); }
    if closure_depth_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0 m (di bawah MSL).".into(); }

    // Bruun Rule: R = SLR × L / (B + h*)
    let denominator = berm_height_m + closure_depth_m;
    let recession = sea_level_rise_m * profile_length_m / denominator;

    // Area lost per km coastline
    let area_lost_per_km = recession * 1000.0; // m² per km coastline

    // IPCC AR6 SLR scenarios (median, by 2100 relative to 1995-2014)
    let scenarios: &[(&str, f64)] = &[
        ("SSP1-2.6 (optimis)", 0.38),
        ("SSP2-4.5 (sedang)", 0.56),
        ("SSP5-8.5 (pesimis)", 0.77),
        ("SSP5-8.5 + ice sheet (ekstrem)", 1.50),
    ];

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  BRUUN RULE — Prediksi Erosi Pantai\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Bruun (1962), Dean (1991), IPCC AR6 WG1\n");
    out.push_str("Rumus: R = SLR × L / (B + h*)\n\n");

    out.push_str(&format!(
        "INPUT:\n  Sea Level Rise (SLR) = {:.3} m\n  Profile Length (L)   = {:.1} m\n  Berm Height (B)      = {:.2} m (di atas MSL)\n  Closure Depth (h*)   = {:.2} m (di bawah MSL)\n\n",
        sea_level_rise_m, profile_length_m, berm_height_m, closure_depth_m
    ));

    out.push_str(&format!(
        "HASIL:\n  Resesi garis pantai (R) = {:.2} m\n  Luas hilang per km pantai = {:.0} m²/km ({:.2} ha/km)\n\n",
        recession, area_lost_per_km, area_lost_per_km / 10000.0
    ));

    // Timeline estimate (assuming linear SLR rate ~3.6 mm/yr current)
    let current_slr_rate = 0.0036; // m/yr
    let years_to_rise = sea_level_rise_m / current_slr_rate;
    let annual_recession = recession / years_to_rise;
    out.push_str(&format!(
        "TIMELINE (asumsi laju SLR saat ini {:.1} mm/tahun):\n  Waktu mencapai SLR {:.3} m = {:.0} tahun\n  Laju resesi tahunan = {:.3} m/tahun\n\n",
        current_slr_rate * 1000.0, sea_level_rise_m, years_to_rise, annual_recession
    ));

    // Comparison across IPCC scenarios
    out.push_str("PERBANDINGAN SKENARIO IPCC AR6 (2100):\n");
    out.push_str(&format!("  {:30} {:>10} {:>12} {:>14}\n", "Skenario", "SLR (m)", "Resesi (m)", "Hilang (ha/km)"));
    out.push_str(&format!("  {:30} {:>10} {:>12} {:>14}\n", "─".repeat(30), "─".repeat(10), "─".repeat(12), "─".repeat(14)));
    for (name, slr) in scenarios {
        let r = slr * profile_length_m / denominator;
        let area_ha = r * 1000.0 / 10000.0;
        out.push_str(&format!("  {:30} {:>10.3} {:>12.2} {:>14.2}\n", name, slr, r, area_ha));
    }

    // Input scenario highlight
    out.push_str(&format!("  {:30} {:>10.3} {:>12.2} {:>14.2}  ← INPUT\n\n",
        "User scenario", sea_level_rise_m, recession, area_lost_per_km / 10000.0));

    // Warnings
    if recession > 50.0 {
        out.push_str("⚠️ Resesi > 50 m — ancaman serius untuk infrastruktur pesisir.\n");
    }
    if recession > 100.0 {
        out.push_str("⚠️ Resesi > 100 m — pertimbangkan managed retreat / relokasi.\n");
    }
    out.push_str("\nCatatan:\n");
    out.push_str("  - Bruun Rule asumsi profil pantai ekuilibrium (sandy beach).\n");
    out.push_str("  - Tidak memperhitungkan sedimen supply, hardening, atau coral reef.\n");
    out.push_str("  - Nilai tipikal Indonesia: B=1-3m, h*=5-15m.\n");
    out.push_str("  - Untuk pantai Indonesia, gunakan data profil aktual dari survey.\n");
    out
}
