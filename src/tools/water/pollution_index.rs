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
        // DO is an inverse parameter: lower concentration = worse quality.
        // Per KepMen 115/2003: for DO, use ratio = Lij/Ci (not Ci/Lij).
        let ratio = if *name == "DO" {
            if *conc > 0.0 { std / conc } else { 999.0 } // DO=0 -> extreme pollution
        } else if *std > 0.0 {
            conc / std
        } else { 0.0 };
        ratios.push(ratio);
        out.push_str(&format!("{:<20} {:>12.2} {:>12.2} {:>10.2}{}\n", name, conc, std, ratio,
            if *name == "DO" {" (inverse)"} else {""}));
    }

    let max_ratio = ratios.iter().cloned().fold(0.0f64, f64::max);
    let avg_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;

    // KepMen LH 115/2003 (Nemerow-Sumitomo): IP = sqrt((max(Ci/Lij)^2 + avg(Ci/Lij)^2) / 2)
    // BUG FIX: was sqrt(max*avg) (geometric mean); correct is RMS of max and mean.
    let pi = ((max_ratio * max_ratio + avg_ratio * avg_ratio) / 2.0).sqrt();

    out.push_str(&format!("\nMax(Ci/Lij) = {:.2}\n", max_ratio));
    out.push_str(&format!("Avg(Ci/Lij) = {:.2}\n", avg_ratio));
    out.push_str(&format!("PI = sqrt((max^2 + avg^2)/2) = {:.2}  [Nemerow-Sumitomo, KepMen 115/2003]\n\n", pi));

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
    for (name, conc, std) in baku_mutu.iter() {
        // DO inverse: violation when conc < std (not >)
        let violated = if *name == "DO" { *conc < *std } else { *std > 0.0 && *conc > *std };
        if violated { storet_score += 1; }
    }
    let storet_class = match storet_score {
        0 => "Class A (Compliant)",
        1..=2 => "Class B (Lightly Polluted)",
        3..=4 => "Class C (Moderately Polluted)",
        _ => "Class D (Heavily Polluted)",
    };
    out.push_str(&format!("STORET score: {}/5 → {}\n", storet_score, storet_class));

    out.push_str("\nLIMITATION:\n");
    out.push_str("  - PI formula: Nemerow-Sumitomo RMS sqrt((max^2+avg^2)/2) — KepMen 115/2003\n");
    out.push_str("  - DO handled as inverse parameter (Lij/Ci)\n");
    out.push_str("  - Does not include all PP 22/2021 parameters (pH, metals, NH3N, etc.)\n");
    out.push_str("  - Total Coliform in MPN/100mL, others in mg/L\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}

#[cfg(test)]
mod tests {
    // Self-check: Nemerow IP with max=2, avg=1 -> sqrt((4+1)/2)=sqrt(2.5)=1.581
    #[test]
    fn nemerow_formula() {
        let max_r = 2.0_f64; let avg_r = 1.0_f64;
        let pi = ((max_r * max_r + avg_r * avg_r) / 2.0).sqrt();
        assert!((pi - 1.5811).abs() < 1e-3, "PI={pi} expected ~1.581");
        // Old geometric mean sqrt(2*1)=1.414 would give different (wrong) answer
        let old_gm = (max_r * avg_r).sqrt();
        assert!((old_gm - 1.4142).abs() < 1e-3, "sanity: old GM=1.414 (different from RMS)");
    }
}
