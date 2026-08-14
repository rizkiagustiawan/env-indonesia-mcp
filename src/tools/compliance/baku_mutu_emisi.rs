/// Baku Mutu Emisi Sumber Tidak Bergerak
/// Ref: Permen LH 5/2026 (Perencanaan Perlindungan & Pengelolaan Mutu Udara)
/// Update: PermenLHK 15/2019 tetap berlaku untuk nilai baku mutu per industri
/// Permen LH 5/2026 menambah: inventarisasi udara, kriteria tanggap darurat

pub fn check(industry: &str, parameter: &str, concentration: f64) -> String {
    let ind = industry.to_lowercase();
    let par = parameter.to_uppercase();

    if concentration < 0.0 {
        return format!(
            "ERROR [E102]: Parameter tidak boleh negatif. {}",
            concentration
        );
    }

    // (industry_key, parameter) -> limit in mg/Nm³ (opacity in %)
    // Ref: PermenLHK 15/2019 (nilai baku mutu per industri)
    let limit: Option<(f64, &str)> = match (ind.as_str(), par.as_str()) {
        // PLTU Batubara (Generalized/strictest limit, >100MW post-2019)
        ("pltu_batubara" | "pltu", "TSP") => Some((50.0, "mg/Nm³")),
        ("pltu_batubara" | "pltu", "SO2") => Some((200.0, "mg/Nm³")),
        ("pltu_batubara" | "pltu", "NOX") => Some((200.0, "mg/Nm³")), // Changed from NO2 to NOx per PermenLHK 15/2019
        ("pltu_batubara" | "pltu", "OPACITY") => Some((10.0, "%")),
        // Semen
        ("semen" | "cement", "TSP") => Some((50.0, "mg/Nm³")),
        ("semen" | "cement", "SO2") => Some((150.0, "mg/Nm³")),
        ("semen" | "cement", "NO2") => Some((400.0, "mg/Nm³")),
        // Smelter
        ("smelter", "TSP") => Some((50.0, "mg/Nm³")),
        ("smelter", "SO2") => Some((400.0, "mg/Nm³")),
        // Kimia
        ("kimia" | "chemical", "TSP") => Some((50.0, "mg/Nm³")),
        ("kimia" | "chemical", "SO2") => Some((200.0, "mg/Nm³")),
        ("kimia" | "chemical", "NO2") => Some((300.0, "mg/Nm³")),
        // Pembangkit Gas
        ("pembangkit_gas" | "gas", "TSP") => Some((30.0, "mg/Nm³")),
        ("pembangkit_gas" | "gas", "SO2") => Some((35.0, "mg/Nm³")),
        ("pembangkit_gas" | "gas", "NO2") => Some((320.0, "mg/Nm³")),
        // Incinerator
        ("incinerator", "TSP") => Some((20.0, "mg/Nm³")),
        ("incinerator", "SO2") => Some((80.0, "mg/Nm³")),
        ("incinerator", "NO2") => Some((250.0, "mg/Nm³")),
        ("incinerator", "CO") => Some((50.0, "mg/Nm³")),
        ("incinerator", "HCL") => Some((35.0, "mg/Nm³")),
        // Tekstil (Permen LH 12/2025 — baku mutu emisi cerobong)
        ("tekstil" | "textile", "TSP") => Some((50.0, "mg/Nm³")),
        ("tekstil" | "textile", "SO2") => Some((150.0, "mg/Nm³")),
        ("tekstil" | "textile", "NO2") => Some((200.0, "mg/Nm³")),
        // Pulp & Paper
        ("pulp" | "pulp_kertas", "TSP") => Some((100.0, "mg/Nm³")),
        ("pulp" | "pulp_kertas", "SO2") => Some((150.0, "mg/Nm³")),
        ("pulp" | "pulp_kertas", "TRS") => Some((2.0, "mg/Nm³")),
        // Sawit (PKS — Pabrik Kelapa Sawit)
        ("pks" | "sawit" | "kelapa_sawit", "TSP") => Some((75.0, "mg/Nm³")),
        ("pks" | "sawit" | "kelapa_sawit", "SO2") => Some((200.0, "mg/Nm³")),
        _ => None,
    };

    let (lim, unit) = match limit {
        Some(v) => v,
        None => {
            return format!(
                "ERROR: Kombinasi industri '{}' dan parameter '{}' tidak ditemukan.\n\
             Industri: pltu_batubara, semen, smelter, kimia, pembangkit_gas, incinerator,\n\
             tekstil (BARU), pulp_kertas, sawit/pks (BARU)\n\
             Parameter: TSP, SO2, NO2, CO, opacity, HCl, TRS\n\n\
             Ref: PermenLHK 15/2019 (nilai); Permen LH 5/2026 (perencanaan mutu udara)",
                industry, parameter
            )
        }
    };

    let pct = (concentration / lim) * 100.0;
    let status = if concentration <= lim {
        "Memenuhi Baku Mutu ✅"
    } else {
        "Melebihi Baku Mutu ❌"
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Baku Mutu Emisi Sumber Tidak Bergerak\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PermenLHK 15/2019 (nilai baku mutu); Permen LH 5/2026 (perencanaan mutu udara)\n\n");
    out.push_str(&format!("Industri    : {}\n", industry));
    out.push_str(&format!("Parameter   : {}\n", parameter));
    out.push_str(&format!("Konsentrasi : {:.2} {}\n", concentration, unit));
    out.push_str(&format!("Baku Mutu   : {:.2} {}\n", lim, unit));
    out.push_str(&format!("Persentase  : {:.1}% dari baku mutu\n\n", pct));
    out.push_str(&format!("Status: {}\n\n", status));

    // Compliance footer
    out.push_str("─── REKOMENDASI MITIGASI ───\n");
    if concentration > lim {
        out.push_str("  1. Optimasi pembakaran (excess air, temperature)\n");
        out.push_str("  2. Tambah/upgrade pengendali polutan (ESP, baghouse, scrubber)\n");
        out.push_str("  3. Bahan bakar lebih bersih (low sulfur coal, natural gas)\n");
        out.push_str("  4. Continuous Emission Monitoring System (CEMS)\n\n");
    } else {
        out.push_str("  Pertahankan kinerja — monitoring berkala\n\n");
    }

    out.push_str("─── PEMANTAUAN (RPL) ───\n");
    out.push_str("  Parameter: TSP, SO2, NO2, CO, HCl (sesuai industri)\n");
    out.push_str("  Frekuensi: Continuous (CEMS untuk PLTU/semen), Bulanan (manual)\n");
    out.push_str("  Metode: SNI / EPA Method / Standard Methods\n\n");

    out.push_str("─── PELAPORAN & IZIN ───\n");
    out.push_str("  Permen LH 5/2026: Perencanaan perlindungan & pengelolaan mutu udara\n");
    out.push_str("  PP 22/2021 Lampiran VII (udara ambien) + Pasal 124-131\n");
    out.push_str("  Amdalnet + OSS; Permen LH 6/2026 (sanksi berbasis risiko)\n");
    out
}
