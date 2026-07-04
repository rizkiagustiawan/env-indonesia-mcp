pub fn index(precip: f64, avg: f64, std: f64) -> String {
    let spi = if std.abs() < 1e-10 { 0.0 } else { (precip - avg) / std };
    
    let category = match spi {
        v if v <= -2.0 => "KEKERINGAN EKSTREM (Extreme Drought)",
        v if v <= -1.5 => "KEKERINGAN PARAH (Severe Drought)",
        v if v <= -1.0 => "KEKERINGAN SEDANG (Moderate Drought)",
        v if v <= -0.5 => "KEKERINGAN RINGAN (Mild Drought)",
        v if v < 0.5 => "NORMAL",
        v if v < 1.0 => "BASAH RINGAN (Mildly Wet)",
        v if v < 1.5 => "BASAH SEDANG (Moderately Wet)",
        v if v < 2.0 => "BASAH PARAH (Severely Wet)",
        _ => "BASAH EKSTREM (Extremely Wet)",
    };

    let risk = if spi < -1.5 {
        "HIGH RISK — Pertanian terdampak serius, potensi gagal panen. Sumber air menurun signifikan."
    } else if spi < -1.0 {
        "MEDIUM RISK — Pertanian mulai terdampak. Perlu konservasi air."
    } else if spi < -0.5 {
        "LOW RISK — Curah hujan di bawah rata-rata tapi belum kritis."
    } else {
        "MINIMAL RISK — Kondisi curah hujan normal atau di atas rata-rata."
    };

    format!(
        "=== Drought Index (SPI) ===\nPrecipitation: {:.1} mm\nLong-term Average: {:.1} mm\nStandard Deviation: {:.1} mm\n\nSPI: {:.2}\nCategory: {}\n\nRisk Assessment:\n{}\n\nSPI Scale:\n  ≤ -2.0: Extreme Drought\n  -2.0 to -1.5: Severe Drought\n  -1.5 to -1.0: Moderate Drought\n  -1.0 to -0.5: Mild Drought\n  -0.5 to 0.5: Normal\n  0.5 to 1.0: Mildly Wet\n  ≥ 2.0: Extremely Wet\n\nIndonesia Context:\\n  - NTT, NTB timur, Jawa Timur = zona kering, paling rawan kekeringan\\n  - Musim kemarau (Juni-Oktober) kritis\\n  - Data historis: CHIRPS (1981-now) via GEE",
        precip, avg, std, spi, category, risk
    )
}
