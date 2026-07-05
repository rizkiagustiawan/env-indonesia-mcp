/// Peluruhan Radioaktif (Radioactive Decay)
/// Ref: IAEA Safety Standards, BAPETEN, Cember & Johnson (2009)

pub fn calculate(isotope: &str, initial_activity_bq: f64, time_elapsed: f64, time_unit: &str) -> String {
    if initial_activity_bq <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if time_elapsed < 0.0 { return "ERROR [E102]: Parameter tidak boleh negatif.".into(); }

    // Half-lives in seconds
    struct Isotope {
        name: &'static str,
        display: &'static str,
        half_life_s: f64,
        half_life_display: &'static str,
    }

    let isotopes = [
        Isotope { name: "cs137",  display: "Cesium-137",      half_life_s: 30.17 * 365.25 * 86400.0,  half_life_display: "30.17 tahun" },
        Isotope { name: "co60",   display: "Cobalt-60",       half_life_s: 5.27 * 365.25 * 86400.0,   half_life_display: "5.27 tahun" },
        Isotope { name: "i131",   display: "Iodine-131",      half_life_s: 8.02 * 86400.0,            half_life_display: "8.02 hari" },
        Isotope { name: "sr90",   display: "Strontium-90",    half_life_s: 28.8 * 365.25 * 86400.0,   half_life_display: "28.8 tahun" },
        Isotope { name: "ra226",  display: "Radium-226",      half_life_s: 1600.0 * 365.25 * 86400.0, half_life_display: "1600 tahun" },
        Isotope { name: "c14",    display: "Carbon-14",       half_life_s: 5730.0 * 365.25 * 86400.0, half_life_display: "5730 tahun" },
        Isotope { name: "h3",     display: "Tritium (H-3)",   half_life_s: 12.3 * 365.25 * 86400.0,   half_life_display: "12.3 tahun" },
        Isotope { name: "tc99m",  display: "Technetium-99m",  half_life_s: 6.0 * 3600.0,              half_life_display: "6 jam" },
        Isotope { name: "u238",   display: "Uranium-238",     half_life_s: 4.47e9 * 365.25 * 86400.0, half_life_display: "4.47×10⁹ tahun" },
        Isotope { name: "am241",  display: "Americium-241",   half_life_s: 432.2 * 365.25 * 86400.0,  half_life_display: "432.2 tahun" },
    ];

    let iso_lower = isotope.to_lowercase();
    let iso = match isotopes.iter().find(|i| i.name == iso_lower.as_str()) {
        Some(i) => i,
        None => {
            let available: Vec<&str> = isotopes.iter().map(|i| i.name).collect();
            return format!(
                "ERROR: Isotop '{}' tidak dikenal.\nIsotop tersedia: {}",
                isotope, available.join(", ")
            );
        }
    };

    // Convert time to seconds
    let tu_lower = time_unit.to_lowercase();
    let time_s = match tu_lower.as_str() {
        "s" | "detik" | "second" | "seconds" => time_elapsed,
        "min" | "menit" | "minute" | "minutes" => time_elapsed * 60.0,
        "h" | "jam" | "hour" | "hours" => time_elapsed * 3600.0,
        "d" | "hari" | "day" | "days" => time_elapsed * 86400.0,
        "yr" | "tahun" | "year" | "years" => time_elapsed * 365.25 * 86400.0,
        _ => {
            return format!(
                "ERROR: Satuan waktu '{}' tidak dikenal.\nPilihan: s/detik, min/menit, h/jam, d/hari, yr/tahun",
                time_unit
            );
        }
    };

    // λ = ln(2) / t½
    let lambda = (2.0_f64).ln() / iso.half_life_s;

    // A(t) = A₀ × exp(-λt)
    let remaining_activity = initial_activity_bq * (-lambda * time_s).exp();
    let fraction_remaining = remaining_activity / initial_activity_bq;
    let n_half_lives = time_s / iso.half_life_s;

    // Time to reach BAPETEN clearance level (10 Bq/g, assume 1g sample → 10 Bq)
    let clearance_bq = 10.0;
    let time_to_clearance_s = if initial_activity_bq > clearance_bq {
        (initial_activity_bq / clearance_bq).ln() / lambda
    } else {
        0.0
    };

    // Convert clearance time to best unit
    let (clearance_time_val, clearance_time_unit) = if time_to_clearance_s < 3600.0 {
        (time_to_clearance_s / 60.0, "menit")
    } else if time_to_clearance_s < 86400.0 {
        (time_to_clearance_s / 3600.0, "jam")
    } else if time_to_clearance_s < 365.25 * 86400.0 {
        (time_to_clearance_s / 86400.0, "hari")
    } else {
        (time_to_clearance_s / (365.25 * 86400.0), "tahun")
    };

    // Activity in human-readable units
    let (act_val, act_unit) = format_activity(remaining_activity);
    let (init_val, init_unit) = format_activity(initial_activity_bq);

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("PELURUHAN RADIOAKTIF\n");
    result.push_str("Ref: IAEA, BAPETEN, Cember & Johnson (2009)\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("FORMULA:\n");
    result.push_str("  A(t) = A₀ × exp(-λt)\n");
    result.push_str("  λ = ln(2) / t½\n\n");

    result.push_str("INPUT:\n");
    result.push_str(&format!("• Isotop               : {}\n", iso.display));
    result.push_str(&format!("• Waktu paruh (t½)     : {}\n", iso.half_life_display));
    result.push_str(&format!("• Aktivitas awal (A₀)  : {:.4} {} ({:.2e} Bq)\n", init_val, init_unit, initial_activity_bq));
    result.push_str(&format!("• Waktu berlalu        : {:.2} {}\n", time_elapsed, time_unit));
    result.push_str(&format!("• Konstanta peluruhan  : {:.6e} s⁻¹\n\n", lambda));

    result.push_str("HASIL:\n");
    result.push_str(&format!("• Jumlah waktu paruh   : {:.4}\n", n_half_lives));
    result.push_str(&format!("• Aktivitas tersisa    : {:.4} {} ({:.2e} Bq)\n", act_val, act_unit, remaining_activity));
    result.push_str(&format!("• Fraksi tersisa       : {:.6} ({:.4}%)\n\n", fraction_remaining, fraction_remaining * 100.0));

    if initial_activity_bq > clearance_bq {
        result.push_str(&format!(
            "WAKTU MENCAPAI CLEARANCE LEVEL (10 Bq, PerKa BAPETEN):\n• {:.2} {}\n\n",
            clearance_time_val, clearance_time_unit
        ));
    } else {
        result.push_str("Aktivitas awal sudah di bawah clearance level (10 Bq).\n\n");
    }

    result.push_str("CATATAN:\n");
    result.push_str("• Clearance level BAPETEN: 10 Bq/g (untuk material curah)\n");
    result.push_str("• Sumber terbungkus: perlu sertifikat BAPETEN untuk penyimpanan\n");
    result.push_str("• Sumber tidak terpakai: dikembalikan ke BATAN/produsen\n");
    result.push_str("══════════════════════════════════════════════\n");

    result
}

fn format_activity(bq: f64) -> (f64, &'static str) {
    if bq >= 1e12 {
        (bq / 1e12, "TBq")
    } else if bq >= 1e9 {
        (bq / 1e9, "GBq")
    } else if bq >= 1e6 {
        (bq / 1e6, "MBq")
    } else if bq >= 1e3 {
        (bq / 1e3, "kBq")
    } else {
        (bq, "Bq")
    }
}
