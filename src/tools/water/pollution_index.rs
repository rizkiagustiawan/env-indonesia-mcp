/// Water Pollution Index — KepMen LH 115/2003 + STORET
/// Ref: KepMen LH 115/2003; PP 22/2021 (baku mutu)

pub fn calculate(
    bod: f64, cod: f64, do_: f64, tss: f64,
    total_coliform: Option<f64>,
    class: u8,
) -> String {
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Water Pollution Index (PI)\n");
    out.push_str("Ref: KepMen LH 115/2003; PP 22/2021\n\n");

    let baku_mutu = match class {
        1 => [
            ("BOD", bod, 2.0),
            ("COD", cod, 10.0),
            ("DO", do_, 6.0),
            ("TSS", tss, 50.0),
            ("Total Coliform", total_coliform.unwrap_or(0.0), 100.0),
        ],
        2 => [
            ("BOD", bod, 3.0),
            ("COD", cod, 25.0),
            ("DO", do_, 4.0),
            ("TSS", tss, 50.0),
            ("Total Coliform", total_coliform.unwrap_or(0.0), 5000.0),
        ],
        3 => [
            ("BOD", bod, 6.0),
            ("COD", cod, 50.0),
            ("DO", do_, 3.0),
            ("TSS", tss, 400.0),
            ("Total Coliform", total_coliform.unwrap_or(0.0), 10000.0),
        ],
        4 => [
            ("BOD", bod, 12.0),
            ("COD", cod, 100.0),
            ("DO", do_, 0.0),
            ("TSS", tss, 400.0),
            ("Total Coliform", total_coliform.unwrap_or(0.0), 20000.0),
        ],
        _ => [
            ("BOD", bod, 3.0),
            ("COD", cod, 25.0),
            ("DO", do_, 4.0),
            ("TSS", tss, 50.0),
            ("Total Coliform", total_coliform.unwrap_or(0.0), 5000.0),
        ],
    };

    let class_names = ["I (Air Minum)", "II (Sarana Air)", "III (Perikanan/Irigasi)", "IV (Pertanian)"];
    out.push_str(&format!("Baku Mutu: Kelas {}\n\n", class_names[(class as usize - 1).min(3)]));

    let mut ratios: Vec<f64> = Vec::new();
    out.push_str(&format!("{:<20} {:>12} {:>12} {:>10}\n", "Parameter", "Conc(mg/L)", "Std(mg/L)", "Ci/Lij"));
    out.push_str(&"-".repeat(54).to_string());
    out.push('\n');

    for (name, conc, std) in baku_mutu.iter() {
        let ratio = if *std > 0.0 { conc / std } else { 0.0 };
        ratios.push(ratio);
        out.push_str(&format!("{:<20} {:>12.2} {:>12.2} {:>10.2}\n", name, conc, std, ratio));
    }

    let max_ratio = ratios.iter().cloned().fold(0.0f64, f64::max);
    let avg_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;

    let pi = (max_ratio * avg_ratio).sqrt();

    out.push_str(&format!("\nMax(Ci/Lij) = {:.2}\n", max_ratio));
    out.push_str(&format!("Avg(Ci/Lij) = {:.2}\n", avg_ratio));
    out.push_str(&format!("PI = sqrt(max x avg) = {:.2}\n\n", pi));

    let pi_class = if pi <= 1.0 {
        "0 - 1.0: Good (Memenuhi Baku Mutu)"
    } else if pi <= 5.0 {
        "1.0 - 5.0: Lightly Polluted (Cemar Ringan)"
    } else if pi <= 10.0 {
        "5.0 - 10.0: Moderately Polluted (Cemar Sedang)"
    } else {
        "> 10: Heavily Polluted (Cemar Berat)"
    };

    out.push_str(&format!("POLLUTION STATUS: {}\n\n", pi_class));

    let violations: Vec<&str> = baku_mutu.iter()
        .zip(ratios.iter())
        .filter(|(_, r)| **r > 1.0)
        .map(|((name, _, _), _)| *name)
        .collect();

    if violations.is_empty() {
        out.push_str("All parameters within baku mutu.\n");
    } else {
        out.push_str(&format!("Exceedance: {} exceed standard\n", violations.join(", ")));
    }

    out.push_str("\nSTORET Method (alternative):\n");
    let mut storet_score = 0;
    for (_, conc, std) in baku_mutu.iter() {
        if *std > 0.0 && *conc > *std {
            storet_score += 1;
        }
    }
    let storet_class = match storet_score {
        0 => "Class A (Compliant)",
        1..=2 => "Class B (Lightly Polluted)",
        3..=4 => "Class C (Moderately Polluted)",
        _ => "Class D (Heavily Polluted)",
    };
    out.push_str(&format!("STORET score: {}/5 → {}\n", storet_score, storet_class));

    out.push_str("\nLIMITATION:\n");
    out.push_str("  - PI formula: sqrt(max(Ci/Lij) × avg(Ci/Lij)) — KepMen 115/2003\n");
    out.push_str("  - Does not include all PP 22/2021 parameters (pH, metals, NH3N, etc.)\n");
    out.push_str("  - DO is inverse (lower = worse) — formula treats all same\n");
    out.push_str("  - Total Coliform in MPN/100mL, others in mg/L\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}
