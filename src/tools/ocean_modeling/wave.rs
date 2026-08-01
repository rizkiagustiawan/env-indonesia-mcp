/// JONSWAP Wave Height Calculator (replaces SMB)
/// Ref: Hasselmann et al. (1973), Shore Protection Manual

pub fn jonswap(wind_speed_ms: f64, fetch_m: f64, depth_m: f64) -> String {
    if wind_speed_ms < 0.28 {
        return "ERROR: Wind speed < 0.28 m/s.".into();
    }
    if fetch_m <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if depth_m <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    let g = 9.81_f64;
    let u = wind_speed_ms;

    // Deep water JONSWAP
    let x_star = g * fetch_m / (u * u);
    let hs_deep = 0.0016 * x_star.sqrt() * u * u / g;
    let tp_deep = 0.286 * x_star.powf(1.0 / 3.0) * u / g;

    // Fully developed limit
    let hs_max = 0.243 * u * u / g;
    let hs = hs_deep.min(hs_max);

    // Shallow water correction (simplified)
    let hs_shallow = hs * (0.75 * depth_m / hs).min(1.0).sqrt();
    let hs_final = hs_shallow.min(0.78 * depth_m); // Breaking limit

    // Rayleigh statistics
    let h_1_10 = 1.27 * hs_final; // 1 in 10 waves
    let h_1_100 = 1.52 * hs_final; // 1 in 100
    let h_max = 1.86 * hs_final; // 1 in 1000

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  JONSWAP Wave Height Model\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Hasselmann et al. (1973), SPM 1984\n");
    out.push_str("⚠️ Valid untuk fetch-limited wind waves, bukan swell.\n\n");
    out.push_str(&format!(
        "INPUT:\n  Wind speed = {:.1} m/s\n  Fetch = {:.0} m ({:.1} km)\n  Depth = {:.1} m\n\n",
        u,
        fetch_m,
        fetch_m / 1000.0,
        depth_m
    ));
    out.push_str(&format!("HASIL:\n  Hs (deep water) = {:.2} m\n  Hs (depth-limited) = {:.2} m\n  Tp (peak period) = {:.1} s\n\n", hs, hs_final, tp_deep));
    out.push_str(&format!("Distribusi Rayleigh:\n  H1/10 (1 in 10) = {:.2} m\n  H1/100 (1 in 100) = {:.2} m\n  Hmax (1 in 1000) = {:.2} m\n\n", h_1_10, h_1_100, h_max));

    if hs_final >= 0.78 * depth_m * 0.95 {
        out.push_str("⚠️ Gelombang mendekati breaking limit (Hb/d = 0.78).\n");
    }
    out
}

pub fn coral_bleaching_dhw(sst_weekly: &[f64], sst_max_monthly_mean: f64) -> String {
    if sst_weekly.is_empty() {
        return "ERROR: Minimal 1 data SST mingguan.".into();
    }

    let mut dhw = 0.0_f64;
    let mut hotspots: Vec<(usize, f64)> = Vec::new();

    for (i, &sst) in sst_weekly.iter().enumerate() {
        let hs = sst - sst_max_monthly_mean;
        if hs > 1.0 {
            dhw += hs;
            hotspots.push((i + 1, hs));
        }
    }

    let risk = if dhw < 4.0 {
        "Watch — Waspada, bleaching belum terjadi"
    } else if dhw < 8.0 {
        "Warning — Bleaching kemungkinan besar terjadi"
    } else {
        "Alert Level 2 — Bleaching parah dan kematian coral"
    };

    let mut out = String::from(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Coral Bleaching DHW\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    out.push_str("Ref: NOAA Coral Reef Watch\n\n");
    out.push_str(&format!(
        "SST Max Monthly Mean = {:.1}°C\nData SST: {} minggu\n\n",
        sst_max_monthly_mean,
        sst_weekly.len()
    ));

    if !hotspots.is_empty() {
        out.push_str("Hotspot Weeks (SST > MMM + 1°C):\n");
        for (week, hs) in &hotspots {
            out.push_str(&format!("  Minggu {}: +{:.1}°C\n", week, hs));
        }
    }

    out.push_str(&format!("\nDHW = {:.1} °C-weeks\nRisiko: {}\n", dhw, risk));
    if dhw >= 4.0 {
        out.push_str("\n⚠️ Terumbu karang Indonesia (perairan Indonesia) terancam bleaching.\n");
    }
    out
}
