/// Desain Sampling — Penentuan Jumlah Sampel
/// Ref: US EPA Guidance on Choosing a Sampling Design, RKL-RPL (PP 22/2021)

pub fn calculate(confidence_pct: f64, margin_error_pct: f64, std_deviation: f64, population_size: Option<u64>) -> String {
    if confidence_pct <= 0.0 || confidence_pct >= 100.0 {
        return "ERROR: Tingkat kepercayaan harus antara 0 dan 100 (eksklusif).".into();
    }
    if margin_error_pct <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if std_deviation <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    // Z-value lookup
    let z = if confidence_pct >= 99.0 {
        2.576
    } else if confidence_pct >= 95.0 {
        1.96
    } else if confidence_pct >= 90.0 {
        1.645
    } else if confidence_pct >= 85.0 {
        1.44
    } else if confidence_pct >= 80.0 {
        1.28
    } else {
        // General: z = norminv((1+p)/2) — approximate
        1.96 // fallback to 95%
    };

    let e = margin_error_pct / 100.0 * std_deviation; // absolute margin of error

    // n = (z² × s²) / e²
    let n_infinite = (z * z * std_deviation * std_deviation) / (e * e);
    let n_infinite_rounded = n_infinite.ceil() as u64;

    // Finite population correction
    let (n_adjusted, n_adj_rounded, pop_str) = if let Some(n_pop) = population_size {
        if n_pop == 0 {
            return "ERROR: Ukuran populasi tidak boleh 0.".into();
        }
        let n_adj = n_infinite / (1.0 + (n_infinite - 1.0) / n_pop as f64);
        (n_adj, n_adj.ceil() as u64, format!("{}", n_pop))
    } else {
        (n_infinite, n_infinite_rounded, "∞ (tidak terbatas)".to_string())
    };

    // Sampling frequency recommendation per RKL-RPL
    let freq_recommendation = if n_adj_rounded <= 10 {
        "Bulanan (12×/tahun) — jumlah sampel sedikit, frekuensi tinggi"
    } else if n_adj_rounded <= 30 {
        "Triwulanan (4×/tahun) — jumlah sampel sedang"
    } else {
        "Semesteran (2×/tahun) — jumlah sampel banyak, efisiensi biaya"
    };

    // Cost estimate (typical Indonesian lab costs 2024)
    let cost_per_sample_water = 2_500_000.0_f64; // IDR, typical for water quality panel
    let cost_per_sample_soil = 3_500_000.0;
    let cost_per_sample_air = 4_000_000.0;
    let total_cost_water = n_adj_rounded as f64 * cost_per_sample_water;
    let total_cost_soil = n_adj_rounded as f64 * cost_per_sample_soil;
    let total_cost_air = n_adj_rounded as f64 * cost_per_sample_air;

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("DESAIN SAMPLING — PENENTUAN JUMLAH SAMPEL\n");
    result.push_str("Ref: US EPA Sampling Design, PP 22/2021 (RKL-RPL)\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("FORMULA: n = (z² × s²) / e²\n");
    result.push_str("Koreksi populasi terbatas: n_adj = n / (1 + (n-1)/N)\n\n");

    result.push_str("INPUT:\n");
    result.push_str(&format!("• Tingkat kepercayaan  : {:.1}%\n", confidence_pct));
    result.push_str(&format!("• Nilai z              : {:.3}\n", z));
    result.push_str(&format!("• Margin error         : {:.1}%\n", margin_error_pct));
    result.push_str(&format!("• Standar deviasi (s)  : {:.4}\n", std_deviation));
    result.push_str(&format!("• Ukuran populasi (N)  : {}\n\n", pop_str));

    result.push_str("HASIL:\n");
    result.push_str(&format!("• n (populasi tak terbatas) : {:.2} → {} sampel\n", n_infinite, n_infinite_rounded));
    if population_size.is_some() {
        result.push_str(&format!("• n (terkoreksi FPC)        : {:.2} → {} sampel\n", n_adjusted, n_adj_rounded));
    }
    result.push_str(&format!("\nJUMLAH SAMPEL MINIMUM: {} sampel\n\n", n_adj_rounded));

    result.push_str("STRATEGI SAMPLING:\n");
    result.push_str("• Random       : Setiap lokasi peluang sama, unbiased\n");
    result.push_str("• Sistematik   : Grid/transek teratur, cakupan merata\n");
    result.push_str("• Stratifikasi : Bagi area per zona, proporsional\n");
    result.push_str("• Purposive    : Fokus pada area sensitif/terdampak\n\n");

    result.push_str(&format!("FREKUENSI REKOMENDASI (RKL-RPL): {}\n\n", freq_recommendation));

    result.push_str("ESTIMASI BIAYA (laboratorium Indonesia, 2024):\n");
    result.push_str(&format!("• Air ({} sampel × Rp 2.500.000)  : Rp {:>14.0}\n", n_adj_rounded, total_cost_water));
    result.push_str(&format!("• Tanah ({} sampel × Rp 3.500.000): Rp {:>14.0}\n", n_adj_rounded, total_cost_soil));
    result.push_str(&format!("• Udara ({} sampel × Rp 4.000.000): Rp {:>14.0}\n", n_adj_rounded, total_cost_air));
    result.push_str("══════════════════════════════════════════════\n");

    result
}
