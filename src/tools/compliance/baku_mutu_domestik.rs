/// Baku Mutu Air Limbah Domestik
/// Ref: Permen LH/BPLH 11/2025 (menggantikan PermenLHK 68/2016)
/// Perubahan: volume-based (≤3 m³, >3 m³, >50 m³), parameter baru: detergen total
/// Pendekatan fleksibel: standar teknologi atau verifikasi teknologi baru

pub fn check(parameter: &str, concentration: f64) -> String {
    let par = parameter.to_lowercase();

    // Permen LH 11/2025 limits (same values, updated regulation)
    let result: Option<(f64, f64, &str, bool)> = match par.as_str() {
        "ph" => Some((9.0, 6.0, "", true)),
        "bod" => Some((30.0, 0.0, "mg/L", false)),
        "cod" => Some((100.0, 0.0, "mg/L", false)),
        "tss" => Some((30.0, 0.0, "mg/L", false)),
        "oil_grease" | "minyak_lemak" => Some((5.0, 0.0, "mg/L", false)),
        "ammonia" | "nh3n" | "amonia" => Some((10.0, 0.0, "mg/L", false)),
        "total_coliform" | "coliform" => Some((3000.0, 0.0, "jumlah/100mL", false)),
        "detergen" | "deterjen" | "mbas" => Some((1.0, 0.0, "mg/L", false)), // BARU di 11/2025
        _ => None,
    };

    let (max_lim, min_lim, unit, is_range) = match result {
        Some(v) => v,
        None => {
            return format!(
                "ERROR: Parameter '{}' tidak ditemukan dalam Permen LH 11/2025.\n\
             Parameter valid: pH, BOD, COD, TSS, oil_grease, ammonia, total_coliform, detergen (BARU)\n\
             Note: Permen LH 11/2025 menggantikan PermenLHK 68/2016 (Nov 2025)\n\
             Pendekatan volume-based: ≤3 m³, >3 m³, >50 m³ (kewajiban kajian teknis)",
                parameter
            )
        }
    };

    let (status, detail) = if is_range {
        let ok = concentration >= min_lim && concentration <= max_lim;
        (
            if ok {
                "Memenuhi Baku Mutu ✅"
            } else {
                "Melebihi Baku Mutu ❌"
            },
            format!("Range: {}-{}", min_lim, max_lim),
        )
    } else {
        let pct = (concentration / max_lim) * 100.0;
        (
            if concentration <= max_lim {
                "Memenuhi Baku Mutu ✅"
            } else {
                "Melebihi Baku Mutu ❌"
            },
            format!("{:.1}% dari baku mutu", pct),
        )
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Air Limbah Domestik\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Permen LH/BPLH 11/2025 (menggantikan PermenLHK 68/2016)\n\n");
    out.push_str(&format!("Parameter   : {}\n", parameter));
    if is_range {
        out.push_str(&format!("Nilai       : {:.2}\n", concentration));
        out.push_str(&format!("Baku Mutu   : {:.1} - {:.1}\n", min_lim, max_lim));
    } else {
        out.push_str(&format!("Konsentrasi : {:.2} {}\n", concentration, unit));
        out.push_str(&format!("Baku Mutu   : {:.2} {}\n", max_lim, unit));
    }
    out.push_str(&format!("Persentase  : {}\n\n", detail));
    out.push_str(&format!("Status: {}\n\n", status));
    out.push_str("Daftar Baku Mutu Permen LH 11/2025:\n");
    out.push_str("  pH           : 6 - 9\n");
    out.push_str("  BOD          : 30 mg/L\n");
    out.push_str("  COD          : 100 mg/L\n");
    out.push_str("  TSS          : 30 mg/L\n");
    out.push_str("  Minyak&Lemak : 5 mg/L\n");
    out.push_str("  Amonia       : 10 mg/L\n");
    out.push_str("  Total Coliform: 3000 jumlah/100mL\n");
    out.push_str("  Detergen (BARU): 1 mg/L MBAS\n\n");
    out.push_str("  Perubahan kunci 11/2025 vs 68/2016:\n");
    out.push_str("  - Volume-based: ≤3 m³, >3 m³, >50 m³ (kewajiban kajian teknis)\n");
    out.push_str("  - Parameter baru: Detergen Total (MBAS)\n");
    out.push_str("  - Pendekatan fleksibel: standar teknologi atau verifikasi\n");
    out.push_str("  - Masa transisi 2 tahun\n");
    out
}
