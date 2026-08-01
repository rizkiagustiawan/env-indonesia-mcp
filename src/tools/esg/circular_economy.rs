/// Circular Economy Material Circularity Indicator (MCI)
/// Ref: Ellen MacArthur Foundation (2015), ISO 59020 (draft)

pub fn calculate(
    mass_product_kg: f64,
    virgin_feedstock_pct: f64,
    recycled_input_pct: f64,
    reused_input_pct: f64,
    recycled_output_pct: f64,
    reused_output_pct: f64,
    product_lifetime_years: f64,
    industry_avg_lifetime: f64,
) -> String {
    if mass_product_kg <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0 kg.".into();
    }
    if product_lifetime_years <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0 tahun.".into();
    }
    if industry_avg_lifetime <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0 tahun.".into();
    }

    // Validate percentages
    let total_input_pct = virgin_feedstock_pct + recycled_input_pct + reused_input_pct;
    if (total_input_pct - 100.0).abs() > 5.0 {
        return format!(
            "ERROR: Total input (virgin + recycled + reused) = {:.1}%, harus mendekati 100%.",
            total_input_pct
        );
    }
    let total_output_pct = recycled_output_pct + reused_output_pct;
    if total_output_pct > 100.0 {
        return "ERROR: Total output (recycled + reused) tidak boleh > 100%.".into();
    }

    let m = mass_product_kg;

    // Fractions
    let fr_input = (recycled_input_pct + reused_input_pct) / 100.0; // fraction from recycled/reused
    let fr_output = (recycled_output_pct + reused_output_pct) / 100.0; // fraction going to recycle/reuse

    // Virgin material mass
    let v = m * (1.0 - fr_input);

    // Unrecoverable waste
    let w0 = m * (1.0 - fr_output); // waste from end-of-life
    let _wf = v; // virgin feedstock (linear flow in)
    let _wc = m * fr_output; // collected for recycling/reuse

    // Linear Flow Index
    // LFI = (V + W) / (2M + (Wf - Wc)/2)  simplified
    // Using EMF formula: LFI = (V + W0) / (2M)
    let w = w0;
    let lfi = (v + w) / (2.0 * m);

    // Utility factor: F(X) = 0.9 / X, where X = L / Lavg
    let x = product_lifetime_years / industry_avg_lifetime;
    let f_x = 0.9 / x;

    // MCI = 1 - LFI × F(X)
    let mci_raw = 1.0 - lfi * f_x;
    let mci = mci_raw.max(0.0).min(1.0); // clamp to [0, 1]

    // Circularity class
    let (class, emoji) = if mci >= 0.8 {
        ("SANGAT CIRCULAR", "🟢")
    } else if mci >= 0.6 {
        ("CIRCULAR", "🟡")
    } else if mci >= 0.4 {
        ("TRANSISI", "🟠")
    } else if mci >= 0.2 {
        ("DOMINAN LINEAR", "🔴")
    } else {
        ("FULLY LINEAR", "⚫")
    };

    // Gap analysis
    let circularity_gap = 1.0 - mci;
    let virgin_gap = virgin_feedstock_pct;
    let eol_waste_pct = 100.0 - total_output_pct;

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  MATERIAL CIRCULARITY INDICATOR (MCI)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Ellen MacArthur Foundation (2015), ISO 59020\n\n");

    out.push_str(&format!(
        "INPUT:\n  Massa produk            = {:.2} kg\n  Virgin feedstock         = {:.1}%\n  Recycled input           = {:.1}%\n  Reused input             = {:.1}%\n  Recycled output (EOL)    = {:.1}%\n  Reused output (EOL)      = {:.1}%\n  Product lifetime         = {:.1} tahun\n  Industry avg lifetime    = {:.1} tahun\n\n",
        m, virgin_feedstock_pct, recycled_input_pct, reused_input_pct,
        recycled_output_pct, reused_output_pct,
        product_lifetime_years, industry_avg_lifetime
    ));

    out.push_str("PERHITUNGAN:\n");
    out.push_str(&format!("  Virgin material (V)      = {:.2} kg\n", v));
    out.push_str(&format!("  Unrecoverable waste (W)  = {:.2} kg\n", w));
    out.push_str(&format!("  Linear Flow Index (LFI)  = {:.4}\n", lfi));
    out.push_str(&format!("  Utility factor X = L/Lavg = {:.2}\n", x));
    out.push_str(&format!("  F(X) = 0.9/X             = {:.4}\n", f_x));
    out.push_str(&format!("  MCI = 1 - LFI × F(X)    = {:.4}\n\n", mci));

    out.push_str(&format!(
        "HASIL:\n  {} MCI Score = {:.2} → {}\n\n",
        emoji, mci, class
    ));

    // Visual bar
    let bar_len = 40;
    let filled = (mci * bar_len as f64) as usize;
    let empty = bar_len - filled;
    out.push_str(&format!(
        "  [{}{}] {:.0}%\n",
        "█".repeat(filled),
        "░".repeat(empty),
        mci * 100.0
    ));
    out.push_str("   0%        25%       50%       75%      100%\n");
    out.push_str("   LINEAR ◄─────────────────────────► CIRCULAR\n\n");

    // Gap analysis
    out.push_str("ANALISIS GAP SIRKULARITAS:\n");
    out.push_str(&format!(
        "  Circularity gap          = {:.1}% (sisa menuju fully circular)\n",
        circularity_gap * 100.0
    ));
    out.push_str(&format!(
        "  Virgin material gap      = {:.1}% (masih dari bahan baru)\n",
        virgin_gap
    ));
    out.push_str(&format!(
        "  End-of-life waste        = {:.1}% (tidak di-recycle/reuse)\n\n",
        eol_waste_pct
    ));

    // Improvement recommendations
    out.push_str("REKOMENDASI PENINGKATAN:\n");
    if virgin_feedstock_pct > 50.0 {
        out.push_str("  • Tingkatkan recycled content — gunakan bahan daur ulang/sekunder\n");
    }
    if eol_waste_pct > 30.0 {
        out.push_str("  • Desain untuk recyclability — pilih material mono-material\n");
        out.push_str("  • Bangun sistem take-back / Extended Producer Responsibility\n");
    }
    if x < 1.0 {
        out.push_str("  • Perpanjang umur produk — desain modular, bisa diperbaiki\n");
        out.push_str("  • Pertimbangkan model Product-as-a-Service (PaaS)\n");
    }
    out.push_str("  • Terapkan PP 27/2020 Pengelolaan Sampah Spesifik\n");
    out.push_str("  • Target: MCI > 0.6 untuk klaim circular economy\n");
    out
}
