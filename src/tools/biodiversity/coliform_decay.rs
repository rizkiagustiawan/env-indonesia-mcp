/// Peluruhan Coliform — Model Mancini
/// Ref: Mancini (1978), PP 22/2021 tentang Baku Mutu Air

pub fn calculate(initial_count_per_100ml: f64, temperature_c: f64, time_hours: f64, water_type: &str) -> String {
    if initial_count_per_100ml <= 0.0 { return "ERROR: Jumlah coliform awal harus > 0.".into(); }
    if temperature_c < 0.0 || temperature_c > 45.0 {
        return "ERROR: Suhu harus antara 0 dan 45 °C.".into();
    }
    if time_hours < 0.0 { return "ERROR: Waktu tidak boleh negatif.".into(); }

    let wt_lower = water_type.to_lowercase();

    // Mancini model: k = k_base × θ^(T-20)
    // k in day⁻¹, θ = 1.07 (temperature coefficient)
    let (k_base, _t90_base_hr, water_name) = match wt_lower.as_str() {
        "freshwater" | "air_tawar" | "sungai" => (0.8, 60.0, "Air Tawar (Freshwater)"),
        "seawater" | "air_laut" | "laut" => (2.0, 36.0, "Air Laut (Seawater)"),
        "tropical" | "tropis" | "pantai_tropis" => (3.0, 18.0, "Perairan Tropis (Tropical)"),
        "estuari" | "estuary" | "muara" => (1.5, 48.0, "Estuari (Estuary)"),
        _ => {
            return format!(
                "ERROR: Tipe perairan '{}' tidak dikenal.\nPilihan: freshwater/air_tawar, seawater/air_laut, tropical/tropis, estuari/estuary",
                water_type
            );
        }
    };

    let theta = 1.07_f64;
    let k_day = k_base * theta.powf(temperature_c - 20.0);
    let k_hour = k_day / 24.0;

    // T90 at temperature (hours for 90% die-off = 1 log removal)
    let t90_hr = (2.303 / k_day) * 24.0; // T90 = ln(10)/k, convert to hours

    // N(t) = N₀ × 10^(-t/T90) = N₀ × exp(-k×t)
    let remaining = initial_count_per_100ml * (-k_hour * time_hours).exp();
    let log_removal = if remaining > 0.0 {
        (initial_count_per_100ml / remaining).log10()
    } else {
        f64::INFINITY
    };

    // PP 22/2021 coliform limits
    let limit_class1 = 1000.0;  // per 100 mL
    let limit_class2 = 5000.0;
    let limit_class3 = 10000.0;
    let _limit_class4 = 10000.0; // same or no limit

    let comply_class1 = remaining <= limit_class1;
    let comply_class2 = remaining <= limit_class2;
    let comply_class3 = remaining <= limit_class3;

    // Time needed to reach Class I compliance
    let time_to_class1 = if initial_count_per_100ml > limit_class1 {
        (initial_count_per_100ml / limit_class1).ln() / k_hour
    } else {
        0.0
    };

    let time_to_class2 = if initial_count_per_100ml > limit_class2 {
        (initial_count_per_100ml / limit_class2).ln() / k_hour
    } else {
        0.0
    };

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("PELURUHAN COLIFORM — MODEL MANCINI\n");
    result.push_str("Ref: Mancini (1978), PP 22/2021\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("FORMULA:\n");
    result.push_str("  N(t) = N₀ × exp(-k × t)\n");
    result.push_str("  k = k_base × 1.07^(T-20)\n");
    result.push_str("  T90 = ln(10) / k\n\n");

    result.push_str("INPUT:\n");
    result.push_str(&format!("• Coliform awal (N₀)   : {:.0}/100mL\n", initial_count_per_100ml));
    result.push_str(&format!("• Suhu air             : {:.1} °C\n", temperature_c));
    result.push_str(&format!("• Waktu kontak         : {:.1} jam\n", time_hours));
    result.push_str(&format!("• Tipe perairan        : {}\n\n", water_name));

    result.push_str("PARAMETER PELURUHAN:\n");
    result.push_str(&format!("• k_base               : {:.2} hari⁻¹\n", k_base));
    result.push_str(&format!("• k pada {:.1}°C         : {:.4} hari⁻¹ ({:.6} jam⁻¹)\n", temperature_c, k_day, k_hour));
    result.push_str(&format!("• T90                  : {:.1} jam ({:.1} hari)\n\n", t90_hr, t90_hr / 24.0));

    result.push_str("HASIL:\n");
    result.push_str(&format!("• Coliform tersisa     : {:.0}/100mL\n", remaining));
    result.push_str(&format!("• Log removal          : {:.2}-log\n", log_removal));
    result.push_str(&format!("• Penurunan            : {:.1}%\n\n", (1.0 - remaining / initial_count_per_100ml) * 100.0));

    result.push_str("KEPATUHAN PP 22/2021 (Baku Mutu Coliform):\n");
    result.push_str(&format!("• Kelas I  (1.000/100mL)  : {}\n",
        if comply_class1 { "MEMENUHI ✓" } else { "TIDAK MEMENUHI ✗" }));
    result.push_str(&format!("• Kelas II (5.000/100mL)  : {}\n",
        if comply_class2 { "MEMENUHI ✓" } else { "TIDAK MEMENUHI ✗" }));
    result.push_str(&format!("• Kelas III (10.000/100mL) : {}\n\n",
        if comply_class3 { "MEMENUHI ✓" } else { "TIDAK MEMENUHI ✗" }));

    if !comply_class1 {
        result.push_str("WAKTU YANG DIPERLUKAN:\n");
        result.push_str(&format!("• Untuk Kelas I  : {:.1} jam ({:.1} hari)\n", time_to_class1, time_to_class1 / 24.0));
        if !comply_class2 {
            result.push_str(&format!("• Untuk Kelas II : {:.1} jam ({:.1} hari)\n", time_to_class2, time_to_class2 / 24.0));
        }
        result.push('\n');
    }

    result.push_str("T90 TIPIKAL (20°C):\n");
    result.push_str("  Air tawar      : 48–72 jam\n");
    result.push_str("  Air laut       : 24–48 jam (efek salinitas)\n");
    result.push_str("  Perairan tropis: 12–24 jam (efek UV + suhu)\n");
    result.push_str("  Estuari        : 36–60 jam\n");
    result.push_str("══════════════════════════════════════════════\n");

    result
}
