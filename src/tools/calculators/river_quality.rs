/// River Quality Model — QUAL2K Simplified (BOD-DO)
/// Ref: Chapra 2008 "Surface Water Quality Modeling"; QUAL2K (Chapra & Pelletier 2003)
/// dL/dx = -(kr/v)*L  (BOD decay)
/// dD/dx = (kc/v)*L - (ka/v)*D  (DO deficit: BOD deoxygenation - reaeration)
pub fn assess(
    river_length_m: f64,
    flow_m3_s: f64,
    velocity_m_s: f64,
    initial_bod_mg_l: f64,
    initial_do_mg_l: f64,
    bod_decay_rate_day: f64,
    reaeration_rate_day: f64,
    saturation_do_mg_l: f64,
    n_reaches: u32,
) -> String {
    let mut out = String::from("=== River Quality Model (QUAL2K Simplified) ===\n");
    out.push_str("Ref: Chapra 2008; QUAL2K (Chapra & Pelletier 2003)\n\n");

    if river_length_m <= 0.0 || flow_m3_s <= 0.0 {
        return "ERROR [E102]: river length and flow must be > 0.".into();
    }

    let v = velocity_m_s * 86400.0; // m/day
    let kr = bod_decay_rate_day; // BOD deoxygenation rate
    let ka = reaeration_rate_day; // reaeration
    let kc = kr; // assume kc = kr for CBOD
    let L0 = initial_bod_mg_l;
    let D0 = saturation_do_mg_l - initial_do_mg_l; // initial deficit
    let dx = river_length_m / n_reaches as f64;

    out.push_str(&format!("River: {:.0}m, {} reaches ({:.0}m each)\n", river_length_m, n_reaches, dx));
    out.push_str(&format!("Flow: {:.1} m3/s, Velocity: {:.1} m/s ({:.0} m/day)\n", flow_m3_s, velocity_m_s, v));
    out.push_str(&format!("Initial: BOD={:.1} mg/L, DO={:.1} mg/L (sat={:.1}, deficit={:.2})\n", L0, initial_do_mg_l, saturation_do_mg_l, D0));
    out.push_str(&format!("Rates: kr={:.3}/day, ka={:.3}/day\n\n", kr, ka));

    // Analytical solution (Streeter-Phelps):
    // L(x) = L0 * exp(-kr * x / v)
    // D(x) = (kc*L0)/(ka-kr) * [exp(-kr*x/v) - exp(-ka*x/v)] + D0*exp(-ka*x/v)
    out.push_str(&format!("{:>8} {:>10} {:>10} {:>10} {:>10}\n", "x (m)", "BOD", "DO", "Deficit", "Status"));
    out.push_str(&"-".repeat(52));
    out.push('\n');

    let mut min_do = saturation_do_mg_l;
    let mut critical_x = 0.0;

    for i in 0..=n_reaches {
        let x = i as f64 * dx;
        let tau = x / v.max(1e-6); // travel time in days

        let bod = L0 * (-kr * tau).exp();
        let deficit = if (ka - kr).abs() > 1e-6 {
            (kc * L0) / (ka - kr) * ((-kr * tau).exp() - (-ka * tau).exp()) + D0 * (-ka * tau).exp()
        } else {
            (kc * L0 * tau + D0) * (-ka * tau).exp()
        };
        let do_val = (saturation_do_mg_l - deficit).max(0.0);

        let status = if do_val < 4.0 { "[CRITICAL]" }
            else if do_val < 5.0 { "[WARN]" }
            else { "[OK]" };

        out.push_str(&format!("{:>8.0} {:>10.2} {:>10.2} {:>10.3} {:>10}\n", x, bod, do_val, deficit, status));

        if do_val < min_do {
            min_do = do_val;
            critical_x = x;
        }
    }

    // Critical point (minimum DO)
    // x_c = (v/(ka-kr)) * ln[(ka/k_r) * (1 - D0*(ka-kr)/(kc*L0))]
    let xc = if (ka - kr).abs() > 1e-6 && kc * L0 > 0.0 {
        let ratio = (ka / kr) * (1.0 - D0 * (ka - kr) / (kc * L0));
        if ratio > 0.0 {
            v / (ka - kr) * ratio.ln()
        } else { 0.0 }
    } else { 0.0 };

    out.push_str(&format!("\n  >> Critical point (min DO): x={:.0}m, DO={:.2} mg/L\n", critical_x, min_do));
    out.push_str(&format!("  >> Theoretical x_c = {:.0}m\n", xc));

    if min_do < 4.0 {
        out.push_str("\n  [CRITICAL] DO < 4 mg/L — fish kill risk. Reduce BOD loading.\n");
    } else if min_do < 5.0 {
        out.push_str("\n  [WARN] DO < 5 mg/L — stress for aquatic life.\n");
    } else {
        out.push_str("\n  [OK] DO adequate (>5 mg/L)\n");
    }

    // ─── PP 22/2021 COMPLIANCE FOOTER ───
    out.push_str("\n─── STATUS KEPATUHAN (PP 22/2021 Lampiran VI) ───\n\n");
    out.push_str("  BOD vs Baku Mutu:\n");
    let bod_final = L0 * (-kr * (river_length_m / v.max(1e-6))).exp();
    let do_final = saturation_do_mg_l - ((kc * L0) / (ka - kr).max(1e-6) * ((-kr * river_length_m / v.max(1e-6)).exp() - (-ka * river_length_m / v.max(1e-6)).exp()) + D0 * (-ka * river_length_m / v.max(1e-6)).exp());
    out.push_str(&format!("  BOD hilir: {:.1} mg/L → Kls I(≤2): {} | Kls II(≤3): {} | Kls III(≤6): {}\n",
        bod_final, if bod_final <= 2.0 {"✅"} else {"❌"}, if bod_final <= 3.0 {"✅"} else {"❌"}, if bod_final <= 6.0 {"✅"} else {"❌"}));
    out.push_str(&format!("  DO hilir: {:.1} mg/L → Kls I(≥6): {} | Kls II(≥4): {} | Kls III(≥3): {}\n\n",
        do_final, if do_final >= 6.0 {"✅"} else {"❌"}, if do_final >= 4.0 {"✅"} else {"❌"}, if do_final >= 3.0 {"✅"} else {"❌"}));

    out.push_str("─── REKOMENDASI MITIGASI ───\n");
    if min_do < 4.0 {
        out.push_str("  1. Reduksi beban BOD di sumber (IPAL komunitas/industri)\n");
        out.push_str("  2. Aerasi sungai (cascade, fountain, aerator)\n");
        out.push_str("  3. Tambah vegetasi riparian untuk shading + nutrient uptake\n");
    } else {
        out.push_str("  Pertahankan kualitas — monitoring rutin\n");
    }

    out.push_str("\n─── PEMANTAUAN (RPL) ───\n");
    out.push_str("  Parameter: BOD, DO, COD, TSS, NH3-N, Total P, coliform\n");
    out.push_str("  Frekuensi: Bulanan (sungai), Mingguan (effluent IPAL)\n");
    out.push_str("  Lokasi: Minimal 3 titik (hulu, titik pelepasan, hilir)\n");
    out.push_str("  Metode: SNI 6989 series; Standard Methods\n");

    out.push_str("\n─── PELAPORAN & IZIN ───\n");
    out.push_str("  PP 22/2021 Pasal 124-131; Amdalnet + OSS\n");
    out.push_str("  Permen LH 6/2026: Sanksi berbasis risiko (denda max Rp3M)\n");
    out.push_str("  Persetujuan Lingkungan (PP 28/2025)\n");

    out.push_str("\n  Ref: Chapra 2008; Streeter-Phelps 1925; QUAL2K; PP 22/2021 Lampiran VI; Permen LH 6/2026\n");
    out
}
