/// Baku Mutu Air Permukaan — PP 22/2021 Lampiran VI
/// Kelas I (A): Air minum/mandi; Kelas II (B): Rekreasi/budidaya air tawar;
/// Kelas III (C): Peternakan/irigasi tanaman pangan; Kelas IV (D): Irigasi non-pangan/industri
/// Ref: PP 22/2021 Lampiran VI; diverifikasi dari 6 file codebase (pollution_index, coliform_decay, do_model_extended, heavy_metal_risk, asgm_mercury, water_quality_engine)

struct ParamLimit {
    min: Option<f64>,
    max: Option<f64>,
    unit: &'static str,
    notes: &'static str,
}

impl ParamLimit {
    fn max_only(max: f64, unit: &'static str) -> Self { Self { min: None, max: Some(max), unit, notes: "" } }
    fn min_only(min: f64, unit: &'static str) -> Self { Self { min: Some(min), max: None, unit, notes: "" } }
    fn range(min: f64, max: f64, unit: &'static str) -> Self { Self { min: Some(min), max: Some(max), unit, notes: "" } }
    fn with_notes(mut self, n: &'static str) -> Self { self.notes = n; self }
}

fn get_limit(parameter: &str, kelas: u8) -> Option<ParamLimit> {
    let p = parameter.to_lowercase().replace("-", "_").replace(" ", "_");
    let k = kelas.min(4).max(1);
    match p.as_str() {
        // ═══ FISIKA ═══
        "suhu" | "temperature" | "temp" => match k {
            1|2|3 => Some(ParamLimit::max_only(3.0, "°C").with_notes("Deviasi maks dari suhu alami")),
            _ => Some(ParamLimit::max_only(5.0, "°C").with_notes("Deviasi maks dari suhu alami")),
        },
        "tss" | "total_suspended_solids" | "padatan_tersuspensi" => match k {
            1|2 => Some(ParamLimit::max_only(50.0, "mg/L")),
            _ => Some(ParamLimit::max_only(400.0, "mg/L")),
        },
        "tds" | "total_dissolved_solids" | "padatan_terlarut" => match k {
            1|2|3 => Some(ParamLimit::max_only(1000.0, "mg/L")),
            _ => Some(ParamLimit::max_only(2000.0, "mg/L")),
        },
        "ph" | "ph_" => match k {
            1|2|3 => Some(ParamLimit::range(6.0, 9.0, "-")),
            _ => Some(ParamLimit::range(5.0, 9.0, "-")),
        },
        // ═══ OKSIGEN / ORGANIK ═══
        "do" | "dissolved_oxygen" | "oksigen_terlarut" => match k {
            1 => Some(ParamLimit::min_only(6.0, "mg/L")),
            2 => Some(ParamLimit::min_only(4.0, "mg/L")),
            3 => Some(ParamLimit::min_only(3.0, "mg/L")),
            _ => Some(ParamLimit::min_only(0.0, "mg/L")),
        },
        "bod" | "bod5" | "kebutuhan_oksigen_biokimiawi" => match k {
            1 => Some(ParamLimit::max_only(2.0, "mg/L")),
            2 => Some(ParamLimit::max_only(3.0, "mg/L")),
            3 => Some(ParamLimit::max_only(6.0, "mg/L")),
            _ => Some(ParamLimit::max_only(12.0, "mg/L")),
        },
        "cod" | "kebutuhan_oksigen_kimiawi" => match k {
            1 => Some(ParamLimit::max_only(10.0, "mg/L")),
            2 => Some(ParamLimit::max_only(25.0, "mg/L")),
            3 => Some(ParamLimit::max_only(50.0, "mg/L")),
            _ => Some(ParamLimit::max_only(100.0, "mg/L")),
        },
        // ═══ MIKROBIOLOGI ═══
        "total_coliform" | "coliform" | "koliform" => match k {
            1 => Some(ParamLimit::max_only(1000.0, "MPN/100mL")),
            2 => Some(ParamLimit::max_only(5000.0, "MPN/100mL")),
            3 => Some(ParamLimit::max_only(10000.0, "MPN/100mL")),
            _ => Some(ParamLimit::max_only(20000.0, "MPN/100mL")),
        },
        "fecal_coliform" | "coliform_tinja" | "e_coli_fecal" => match k {
            1 => Some(ParamLimit::max_only(100.0, "MPN/100mL")),
            2 => Some(ParamLimit::max_only(1000.0, "MPN/100mL")),
            _ => Some(ParamLimit::max_only(2000.0, "MPN/100mL")),
        },
        "e_coli" | "escherichia_coli" => match k {
            1 => Some(ParamLimit::max_only(0.0, "MPN/100mL").with_notes("Nihil")),
            2 => Some(ParamLimit::max_only(100.0, "MPN/100mL")),
            _ => Some(ParamLimit::max_only(1000.0, "MPN/100mL")),
        },
        // ═══ NUTRIEN ═══
        "nh3n" | "nh3" | "ammonia" | "amonia" | "amoniak" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.5, "mg/L").with_notes("Sebagai NH3-N")),
            _ => Some(ParamLimit::max_only(1.0, "mg/L").with_notes("Sebagai NH3-N")),
        },
        "no3n" | "no3" | "nitrat" | "nitrate" => Some(ParamLimit::max_only(10.0, "mg/L").with_notes("Sebagai NO3-N")),
        "no2n" | "no2" | "nitrit" | "nitrite" => Some(ParamLimit::max_only(0.06, "mg/L").with_notes("Sebagai NO2-N")),
        "total_n" | "totaln" | "total_nitrogen" | "nitrogen_total" => match k {
            1|2|3 => Some(ParamLimit::max_only(1.0, "mg/L")),
            _ => Some(ParamLimit::max_only(2.0, "mg/L")),
        },
        "total_p" | "totalp" | "total_phosphorus" | "fosfor_total" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.2, "mg/L").with_notes("Sebagai P")),
            _ => Some(ParamLimit::max_only(1.0, "mg/L").with_notes("Sebagai P")),
        },
        // ═══ BAHAN ORGANIK ═══
        "oil_grease" | "minyak_lemak" | "minyak" | "oil" | "lemak" => match k {
            1 => Some(ParamLimit::max_only(0.0, "mg/L").with_notes("Nihil")),
            2 => Some(ParamLimit::max_only(0.05, "mg/L")),
            3 => Some(ParamLimit::max_only(1.0, "mg/L")),
            _ => Some(ParamLimit::max_only(5.0, "mg/L")),
        },
        "detergen" | "deterjen" | "mbas" | "surfaktan" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.2, "mg/L").with_notes("Sebagai MBAS")),
            _ => Some(ParamLimit::max_only(1.0, "mg/L")),
        },
        "phenol" | "fenol" | "fenol_total" => match k {
            1 => Some(ParamLimit::max_only(0.001, "mg/L")),
            2|3 => Some(ParamLimit::max_only(0.002, "mg/L")),
            _ => Some(ParamLimit::max_only(0.01, "mg/L")),
        },
        // ═══ LOGAM BERAT ═══
        "pb" | "timbal" | "lead" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.05, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        "cd" | "kadmium" | "cadmium" => Some(ParamLimit::max_only(0.01, "mg/L")),
        "cr6" | "cr_vi" | "kromium_vi" | "chromium_vi" | "cr(vi)" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.05, "mg/L").with_notes("Sebagai Cr(VI)")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L").with_notes("Sebagai Cr(VI)")),
        },
        "cr_total" | "kromium_total" | "chromium_total" | "cr" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.05, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        "cu" | "tembaga" | "copper" => match k {
            1|2 => Some(ParamLimit::max_only(0.02, "mg/L")),
            3 => Some(ParamLimit::max_only(0.05, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        "zn" | "seng" | "zinc" => match k {
            1|2 => Some(ParamLimit::max_only(0.05, "mg/L")),
            3 => Some(ParamLimit::max_only(0.1, "mg/L")),
            _ => Some(ParamLimit::max_only(0.5, "mg/L")),
        },
        "ni" | "nikel" | "nickel" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.05, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        "hg" | "merkuri" | "mercury" => match k {
            1|2 => Some(ParamLimit::max_only(0.001, "mg/L")),
            3 => Some(ParamLimit::max_only(0.002, "mg/L")),
            _ => Some(ParamLimit::max_only(0.005, "mg/L")),
        },
        "as" | "arsenik" | "arsenic" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.05, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        "fe" | "besi" | "iron" => match k {
            1|2 => Some(ParamLimit::max_only(0.3, "mg/L")),
            _ => Some(ParamLimit::max_only(1.0, "mg/L")),
        },
        "mn" | "mangan" | "manganese" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.1, "mg/L")),
            _ => Some(ParamLimit::max_only(0.5, "mg/L")),
        },
        "sn" | "timah" | "tin" => match k {
            1|2 => Some(ParamLimit::max_only(0.002, "mg/L")),
            3 => Some(ParamLimit::max_only(0.05, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        "ag" | "perak" | "silver" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.05, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        "co" | "kobalt" | "cobalt" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.05, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        "ba" | "barium" => match k {
            1|2 => Some(ParamLimit::max_only(1.0, "mg/L")),
            3 => Some(ParamLimit::max_only(2.0, "mg/L")),
            _ => Some(ParamLimit::max_only(3.0, "mg/L")),
        },
        "be" | "berilium" | "beryllium" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.0001, "mg/L")),
            _ => Some(ParamLimit::max_only(0.0005, "mg/L")),
        },
        "se" | "selenium" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.01, "mg/L")),
            _ => Some(ParamLimit::max_only(0.05, "mg/L")),
        },
        // ═══ ANORGANIK LAIN ═══
        "cn" | "sianida" | "cyanide" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.02, "mg/L").with_notes("Sebagai CN")),
            _ => Some(ParamLimit::max_only(0.05, "mg/L")),
        },
        "f" | "fluorida" | "fluoride" => match k {
            1|2|3 => Some(ParamLimit::max_only(1.5, "mg/L")),
            _ => Some(ParamLimit::max_only(3.0, "mg/L")),
        },
        "cl" | "klorida" | "chloride" => match k {
            1|2 => Some(ParamLimit::max_only(250.0, "mg/L")),
            3 => Some(ParamLimit::max_only(600.0, "mg/L")),
            _ => Some(ParamLimit::max_only(1200.0, "mg/L")),
        },
        "so4" | "sulfat" | "sulfate" => match k {
            1|2 => Some(ParamLimit::max_only(250.0, "mg/L")),
            _ => Some(ParamLimit::max_only(400.0, "mg/L")),
        },
        "h2s" | "sulfida" | "sulfide" => match k {
            1|2 => Some(ParamLimit::max_only(0.002, "mg/L").with_notes("Sebagai H2S")),
            3 => Some(ParamLimit::max_only(0.05, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        // ═══ VOC / PESTISIDA ═══
        "benzene" | "benzena" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.01, "mg/L")),
            _ => Some(ParamLimit::max_only(0.1, "mg/L")),
        },
        "chloroform" | "kloroform" => match k {
            1|2 => Some(ParamLimit::max_only(0.0007, "mg/L")),
            3 => Some(ParamLimit::max_only(0.03, "mg/L")),
            _ => Some(ParamLimit::max_only(0.3, "mg/L")),
        },
        "carbon_tetrachloride" | "ccl4" | "karbon_tetraklorida" => match k {
            1|2 => Some(ParamLimit::max_only(0.003, "mg/L")),
            3 => Some(ParamLimit::max_only(0.06, "mg/L")),
            _ => Some(ParamLimit::max_only(0.6, "mg/L")),
        },
        "tce" | "trichloroethylene" | "trikloroetilen" => match k {
            1|2 => Some(ParamLimit::max_only(0.003, "mg/L")),
            3 => Some(ParamLimit::max_only(0.06, "mg/L")),
            _ => Some(ParamLimit::max_only(0.6, "mg/L")),
        },
        "pce" | "tetrachloroethylene" | "tetrakloroetilen" => match k {
            1|2 => Some(ParamLimit::max_only(0.003, "mg/L")),
            3 => Some(ParamLimit::max_only(0.06, "mg/L")),
            _ => Some(ParamLimit::max_only(0.6, "mg/L")),
        },
        "pestisida_organoklorin" | "organoklorin" | "organochlorine" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.0005, "mg/L")),
            _ => Some(ParamLimit::max_only(0.001, "mg/L")),
        },
        "pestisida_organofosfat" | "organofosfat" | "organophosphate" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.001, "mg/L")),
            _ => Some(ParamLimit::max_only(0.01, "mg/L")),
        },
        "pestisida_total" | "pesticide_total" | "pestisida" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.0005, "mg/L")),
            _ => Some(ParamLimit::max_only(0.001, "mg/L")),
        },
        "aldrin" | "dieldrin" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.0001, "mg/L")),
            _ => Some(ParamLimit::max_only(0.0005, "mg/L")),
        },
        // ═══ RADIOAKTIVITAS ═══
        "gross_alpha" | "alfa_total" | "radioaktif_alfa" | "gross_alfa" => match k {
            1|2|3 => Some(ParamLimit::max_only(0.1, "Bq/L")),
            _ => Some(ParamLimit::max_only(1.0, "Bq/L")),
        },
        "gross_beta" | "beta_total" | "radioaktif_beta" => match k {
            1|2|3 => Some(ParamLimit::max_only(1.0, "Bq/L")),
            _ => Some(ParamLimit::max_only(10.0, "Bq/L")),
        },
        _ => None,
    }
}

fn kelas_name(k: u8) -> &'static str {
    match k { 1 => "Kelas I (A) — Air Minum/Mandi", 2 => "Kelas II (B) — Rekreasi/Budidaya Air Tawar",
              3 => "Kelas III (C) — Peternakan/Irigasi Tanaman Pangan", _ => "Kelas IV (D) — Irigasi Non-Pangan/Industri" }
}

fn check_compliance(value: f64, limit: &ParamLimit, param: &str, _kelas: u8) -> (bool, String) {
    let (ok, status) = if let Some(min) = limit.min {
        if let Some(max) = limit.max {
            (value >= min && value <= max, format!("Range {:.3}-{:.3}", min, max))
        } else {
            (value >= min, format!(">= {:.3}", min))
        }
    } else if let Some(max) = limit.max {
        (value <= max, format!("<= {:.3}", max))
    } else {
        (true, "Tidak dibakukan".to_string())
    };

    let verdict = if ok { "MEMENUHI" } else { "MELEBIHI" };
    let detail = if limit.notes.is_empty() {
        format!("{} {} {} ({}) → {}",
            param, value, limit.unit, status, verdict)
    } else {
        format!("{} {} {} ({}) [{}] → {}",
            param, value, limit.unit, status, limit.notes, verdict)
    };
    (ok, detail)
}

pub fn assess(parameter: &str, value: f64, kelas: u8) -> String {
    let mut out = String::from("=== Baku Mutu Air Permukaan (PP 22/2021 Lampiran VI) ===\n\n");

    let limit = match get_limit(parameter, kelas) {
        Some(l) => l,
        None => return format!("Parameter '{}' tidak ditemukan di PP 22/2021 Lampiran VI.\nParameter tersedia: suhu, tss, tds, ph, do, bod, cod, total_coliform, fecal_coliform, e_coli, nh3n, no3n, no2n, total_n, total_p, oil_grease, detergen, phenol, pb, cd, cr6, cr_total, cu, zn, ni, hg, as, fe, mn, sn, ag, co, ba, be, se, cn, f, cl, so4, h2s, benzene, chloroform, ccl4, tce, pce, pestisida_organoklorin, pestisida_organofosfat, aldrin, gross_alpha, gross_beta", parameter),
    };

    out.push_str(&format!("Parameter: {}\n", parameter));
    out.push_str(&format!("Nilai: {} {}\n", value, limit.unit));
    out.push_str(&format!("Peruntukan: {}\n\n", kelas_name(kelas)));

    let (ok, detail) = check_compliance(value, &limit, parameter, kelas);

    out.push_str("─── STATUS KEPATUHAN ───\n");
    out.push_str(&format!("  {}\n", detail));

    if !ok {
        out.push_str(&format!("\n  ❌ MELEBIHI BAKU MUTU PP 22/2021 Kelas {}\n", kelas));
        let exceedance = if let Some(max) = limit.max { value - max }
                         else if let Some(min) = limit.min { min - value }
                         else { 0.0 };
        out.push_str(&format!("  Exceedance: {:.4} {}\n", exceedance, limit.unit));
    } else {
        out.push_str(&format!("\n  ✅ MEMENUHI BAKU MUTU PP 22/2021 Kelas {}\n", kelas));
    }

    // Mitigation
    out.push_str("\n─── REKOMENDASI MITIGASI ───\n");
    if !ok {
        match parameter.to_lowercase().as_str() {
            "bod"|"bod5" => { out.push_str("  1. Optimasi IPAL — tambah aerasi/oksigenasi\n  2. Reduksi beban organik di sumber\n  3. Tambah kolam stabilisasi sekunder\n"); }
            "do" => { out.push_str("  1. Aerasi mekanik (cascade, fountain)\n  2. Reduksi beban BOD upstream\n  3. kontrol eutrofifikasi (kurangi P/N)\n"); }
            "tss" => { out.push_str("  1. Sedimentasi basin / clarifier\n  2. Filter screen / sand filter\n  3. Erosion control di DAS\n"); }
            _ => { out.push_str("  1. Identifikasi sumber pencemar\n  2. Optimasi pengolahan\n  3. Monitoring berkelanjutan\n"); }
        }
    } else {
        out.push_str("  Pertahankan kualitas air — lanjutkan monitoring rutin\n");
    }

    // Monitoring (RPL)
    out.push_str("\n─── PEMANTAUAN (RPL) ───\n");
    out.push_str("  Frekuensi: Bulanan (sungai), Mingguan (effluent IPAL)\n");
    out.push_str("  Lokasi: Minimal 3 titik (hulu, titik pelepasan, hilir)\n");
    out.push_str(&format!("  Parameter: {} + parameter pendukung sesuai PP 22/2021\n", parameter));
    out.push_str("  Metode: SNI / Standard Methods sesuai parameter\n");

    // Reporting
    out.push_str("\n─── PELAPORAN ───\n");
    out.push_str("  PP 22/2021 Pasal 124-131: Laporan kinerja LH semesteran\n");
    out.push_str("  Sistem: Amdalnet + OSS (Permen LH 6/2026)\n");
    out.push_str("  Izin: Persetujuan Lingkungan (PP 28/2025)\n");

    out.push_str("\n  Ref: PP 22/2021 Lampiran VI; diverifikasi dari pollution_index.rs, coliform_decay.rs, do_model_extended.rs, heavy_metal_risk.rs, asgm_mercury.rs\n");
    out
}

/// Check multiple parameters at once
pub fn assess_multi(params_json: &str, kelas: u8) -> String {
    let params: Vec<(String, f64)> = match serde_json::from_str(params_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: {}. Format: {{\"parameter\": nilai, ...}}", e),
    };

    let mut out = format!("=== Baku Mutu Air Permukaan Multi-Parameter (PP 22/2021 Kelas {}) ===\n\n", kelas);
    out.push_str(&format!("Peruntukan: {}\n\n", kelas_name(kelas)));

    out.push_str(&format!("{:<20} {:>12} {:>12} {:>15} {:>10}\n", "Parameter", "Nilai", "Baku Mutu", "Unit", "Status"));
    out.push_str(&"-".repeat(72));
    out.push('\n');

    let mut n_ok = 0;
    let mut n_fail = 0;
    let mut n_unknown = 0;

    for (param, value) in &params {
        match get_limit(param, kelas) {
            Some(limit) => {
                let (ok, _) = check_compliance(*value, &limit, param, kelas);
                let bm_str = if let Some(m) = limit.max {
                    if m == 0.0 { "Nihil".to_string() } else { format!("≤{:.3}", m) }
                } else if let Some(mn) = limit.min {
                    format!("≥{:.3}", mn)
                } else { "—".to_string() };

                let status = if ok { "✅" } else { "❌" };
                out.push_str(&format!("{:<20} {:>12.4} {:>12} {:>15} {:>10}\n",
                    param, value, bm_str, limit.unit, status));
                if ok { n_ok += 1; } else { n_fail += 1; }
            }
            None => {
                out.push_str(&format!("{:<20} {:>12.4} {:>12} {:>15} {:>10}\n", param, value, "?", "?", "⚠️"));
                n_unknown += 1;
            }
        }
    }

    out.push_str(&format!("\n  Ringkasan: {} MEMENUHI, {} MELEBIHI, {} TIDAK DIKENAL\n", n_ok, n_fail, n_unknown));

    if n_fail > 0 {
        out.push_str("\n  ❌ Ada parameter MELEBIHI baku mutu — tindakan perbaikan diperlukan!\n");
    } else if n_ok > 0 {
        out.push_str("\n  ✅ Semua parameter MEMENUHI baku mutu\n");
    }

    out.push_str("\n─── PELAPORAN ───\n");
    out.push_str("  PP 22/2021 Pasal 124-131; Amdalnet + OSS; Permen LH 6/2026 (sanksi)\n");
    out.push_str("  Sanksi: Teguran → Paksaan → Denda (max Rp3M) → Pencabutan izin\n");

    out.push_str("\n  Ref: PP 22/2021 Lampiran VI\n");
    out
}
