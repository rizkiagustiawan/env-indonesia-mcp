/// Analisis Risiko Kesehatan Lingkungan (ARKL)
/// Ref: Pedoman ARKL Kemenkes 2012, US EPA RAGS, PP 101/2014

// ══════════════════════════════════════════════════════════════
// Indonesian ARKL defaults (Pedoman Kemenkes 2012)
// ══════════════════════════════════════════════════════════════
const ARKL_BW_ADULT: f64 = 55.0; // kg (Indonesian adult average)
const ARKL_BW_CHILD: f64 = 15.0; // kg
const ARKL_IR_INHAL_ADULT: f64 = 20.0; // m³/day
const ARKL_IR_INHAL_CHILD: f64 = 12.0; // m³/day
const ARKL_IR_ORAL_ADULT: f64 = 2.0; // L/day (water ingestion)
const ARKL_IR_ORAL_CHILD: f64 = 1.0; // L/day
const ARKL_IR_SOIL_ADULT: f64 = 50.0; // mg/day (soil ingestion)
const ARKL_IR_SOIL_CHILD: f64 = 200.0; // mg/day
const ARKL_SA_ADULT: f64 = 5000.0; // cm² (skin surface area)
const ARKL_SA_CHILD: f64 = 2800.0; // cm²
const ARKL_FE_RESIDENTIAL: f64 = 350.0; // days/year
const ARKL_FE_OCCUPATIONAL: f64 = 250.0;
const ARKL_FE_SCHOOL: f64 = 240.0;
const ARKL_DT_RESIDENTIAL: f64 = 30.0; // years
const ARKL_DT_OCCUPATIONAL: f64 = 25.0;
const ARKL_DT_CHILD: f64 = 6.0;
const ARKL_AT_CANCER: f64 = 70.0; // years (WHO convention)
const US_EPA_BW_ADULT: f64 = 70.0; // kg (US EPA default)

// ══════════════════════════════════════════════════════════════
// RfD / RfC lookup (US EPA IRIS, verified 2024-2025)
// ══════════════════════════════════════════════════════════════

/// Returns (rfd_value, source_note). Units: mg/kg/day for oral, mg/m³ for inhalation RfC.
fn get_rfd(contaminant: &str, route: &str) -> Option<(f64, &'static str)> {
    match (
        contaminant.to_lowercase().as_str(),
        route.to_lowercase().as_str(),
    ) {
        ("arsenic" | "as", "oral") => Some((6e-5, "Arsenic (inorganic) - IRIS 2025")),
        ("chromium_vi" | "cr6" | "cr(vi)", "oral") => Some((9e-4, "Chromium(VI) - IRIS 2024")),
        ("chromium_vi" | "cr6" | "cr(vi)", "inhalation" | "inhalasi") => {
            Some((3e-5, "Chromium(VI) RfC - IRIS 2024"))
        }
        ("cadmium" | "cd", "oral") => Some((5e-4, "Cadmium - IRIS")),
        ("mercury" | "hg", "inhalation" | "inhalasi") => {
            Some((3e-4, "Mercury (elemental) RfC - IRIS"))
        }
        ("methylmercury" | "mehg", "oral") => Some((1e-4, "Methylmercury - IRIS")),
        ("benzene", "oral") => Some((4e-3, "Benzene - IRIS")),
        ("benzene", "inhalation" | "inhalasi") => Some((3e-2, "Benzene RfC - IRIS")),
        ("toluene", "oral") => Some((8e-2, "Toluene - IRIS")),
        ("toluene", "inhalation" | "inhalasi") => Some((5.0, "Toluene RfC - IRIS")),
        ("xylene" | "xylenes", "oral") => Some((2e-1, "Xylenes - IRIS")),
        ("xylene" | "xylenes", "inhalation" | "inhalasi") => Some((1e-1, "Xylenes RfC - IRIS")),
        ("phenol" | "fenol", "oral") => Some((3e-1, "Phenol - IRIS")),
        ("formaldehyde" | "hcho", "inhalation" | "inhalasi") => {
            Some((7e-3, "Formaldehyde RfC - IRIS 2024"))
        }
        ("formaldehyde" | "hcho", "oral") => Some((2e-1, "Formaldehyde - IRIS")),
        ("ammonia" | "nh3", "inhalation" | "inhalasi") => Some((5e-1, "Ammonia RfC - IRIS")),
        ("vinyl_chloride", "oral") => Some((3e-3, "Vinyl chloride - IRIS")),
        ("vinyl_chloride", "inhalation" | "inhalasi") => Some((1e-1, "Vinyl chloride RfC - IRIS")),
        ("lead" | "pb", _) => None, // No RfD established by IRIS
        _ => None,
    }
}

/// CSF lookup (per (mg/kg/day)). Returns (csf_value, source_note).
fn get_csf(contaminant: &str, route: &str) -> Option<(f64, &'static str)> {
    match (
        contaminant.to_lowercase().as_str(),
        route.to_lowercase().as_str(),
    ) {
        ("arsenic" | "as", "oral") => Some((1.5, "Arsenic - IRIS")),
        ("benzene", "inhalation" | "inhalasi") => Some((0.029, "Benzene - IRIS")),
        ("chromium_vi" | "cr6" | "cr(vi)", "inhalation" | "inhalasi") => {
            Some((42.0, "Chromium(VI) - IRIS"))
        }
        ("vinyl_chloride", "oral") => Some((1.4, "Vinyl chloride (oral) - IRIS")),
        ("vinyl_chloride", "inhalation" | "inhalasi") => {
            Some((0.72, "Vinyl chloride (inhal) - IRIS"))
        }
        ("benzo_a_pyrene" | "bap", "oral") => Some((7.3, "Benzo(a)pyrene - IRIS")),
        _ => None,
    }
}

// ══════════════════════════════════════════════════════════════
// 1) ILCR — Incremental Lifetime Cancer Risk (backward compatible)
// ══════════════════════════════════════════════════════════════

pub fn calculate_ilcr(
    exposure_route: &str,
    concentration: f64,
    intake_rate: f64,
    exposure_freq_days: f64,
    exposure_dur_years: f64,
    body_weight_kg: f64,
    avg_time_years: f64,
    csf: f64,
) -> String {
    if concentration < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }
    if intake_rate <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if exposure_freq_days <= 0.0 || exposure_freq_days > 365.0 {
        return "ERROR: Frekuensi paparan harus antara 0 dan 365 hari/tahun.".into();
    }
    if exposure_dur_years <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if body_weight_kg <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if avg_time_years <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if csf <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    let route_lower = exposure_route.to_lowercase();
    let (route_name, route_unit) = match route_lower.as_str() {
        "inhalation" | "inhalasi" => ("Inhalasi", "mg/m³ untuk C, m³/hari untuk IR"),
        "ingestion" | "oral" => ("Ingesti (oral)", "mg/kg untuk C, kg/hari untuk IR"),
        "dermal" => ("Dermal", "mg/cm² untuk C, cm² untuk IR"),
        _ => {
            return format!(
                "ERROR: Jalur paparan '{}' tidak dikenal.\nPilihan: inhalation/inhalasi, ingestion/oral, dermal",
                exposure_route
            );
        }
    };

    // CDI = (C × IR × EF × ED) / (BW × AT × 365)
    let at_days = avg_time_years * 365.0;
    let cdi = (concentration * intake_rate * exposure_freq_days * exposure_dur_years)
        / (body_weight_kg * at_days);

    // ILCR = CDI × CSF
    let ilcr = cdi * csf;

    // Compare with Indonesian BW=55 if user used different BW
    let cdi_id = (concentration * intake_rate * exposure_freq_days * exposure_dur_years)
        / (ARKL_BW_ADULT * at_days);
    let ilcr_id = cdi_id * csf;

    let (risk_level, risk_desc) = if ilcr < 1e-6 {
        (
            "DAPAT DITERIMA",
            "Risiko kanker < 10⁻⁶ — dapat diterima (de minimis)",
        )
    } else if ilcr < 1e-4 {
        (
            "DAPAT DIKELOLA",
            "Risiko kanker 10⁻⁶ – 10⁻⁴ — perlu pengelolaan dan pemantauan",
        )
    } else {
        (
            "TIDAK DAPAT DITERIMA",
            "Risiko kanker > 10⁻⁴ — tindakan remediasi diperlukan",
        )
    };

    let mut r = String::new();
    r.push_str("══════════════════════════════════════════════\n");
    r.push_str("ANALISIS RISIKO KESEHATAN MANUSIA (HHRA)\n");
    r.push_str("INCREMENTAL LIFETIME CANCER RISK (ILCR)\n");
    r.push_str("Ref: US EPA RAGS, PP 101/2014, Pedoman ARKL Kemenkes 2012\n");
    r.push_str("══════════════════════════════════════════════\n\n");

    r.push_str("FORMULA:\n");
    r.push_str("  CDI = (C × IR × EF × ED) / (BW × AT × 365)\n");
    r.push_str("  ILCR = CDI × CSF\n\n");

    r.push_str("INPUT:\n");
    r.push_str(&format!("• Jalur paparan (route)    : {}\n", route_name));
    r.push_str(&format!(
        "• Konsentrasi (C)          : {:.6}\n",
        concentration
    ));
    r.push_str(&format!(
        "• Laju intake (IR)         : {:.4}\n",
        intake_rate
    ));
    r.push_str(&format!(
        "• Frekuensi paparan (EF)   : {:.0} hari/tahun\n",
        exposure_freq_days
    ));
    r.push_str(&format!(
        "• Durasi paparan (ED)      : {:.1} tahun\n",
        exposure_dur_years
    ));
    r.push_str(&format!(
        "• Berat badan (BW)         : {:.1} kg\n",
        body_weight_kg
    ));
    r.push_str(&format!(
        "• Averaging time (AT)      : {:.1} tahun\n",
        avg_time_years
    ));
    r.push_str(&format!(
        "• Cancer Slope Factor (CSF): {:.4} (mg/kg/hari)⁻¹\n",
        csf
    ));
    r.push_str(&format!("  Satuan: {}\n\n", route_unit));

    r.push_str("HASIL:\n");
    r.push_str(&format!(
        "• CDI (Chronic Daily Intake) : {:.6e} mg/kg/hari\n",
        cdi
    ));
    r.push_str(&format!("• ILCR                       : {:.6e}\n\n", ilcr));

    r.push_str(&format!("TINGKAT RISIKO: {}\n", risk_level));
    r.push_str(&format!("  {}\n\n", risk_desc));

    // Indonesian BW comparison
    if (body_weight_kg - ARKL_BW_ADULT).abs() > 0.5 {
        r.push_str("PERBANDINGAN BW INDONESIA vs INPUT:\n");
        r.push_str(&format!(
            "  BW={:.0} kg (input)   → CDI={:.6e}, ILCR={:.6e}\n",
            body_weight_kg, cdi, ilcr
        ));
        r.push_str(&format!(
            "  BW={:.0} kg (ARKL ID) → CDI={:.6e}, ILCR={:.6e}\n",
            ARKL_BW_ADULT, cdi_id, ilcr_id
        ));
        r.push_str(&format!(
            "  Catatan: BW Indonesia ({:.0} kg) vs US EPA ({:.0} kg) — perbedaan CDI ~{:.0}%\n\n",
            ARKL_BW_ADULT,
            US_EPA_BW_ADULT,
            ((US_EPA_BW_ADULT / ARKL_BW_ADULT) - 1.0) * 100.0
        ));
    }

    r.push_str("SKALA RISIKO:\n");
    r.push_str("  < 10⁻⁶  : Dapat diterima (de minimis)\n");
    r.push_str("  10⁻⁶–10⁻⁴: Dapat dikelola (manageable)\n");
    r.push_str("  > 10⁻⁴  : Tidak dapat diterima (unacceptable)\n\n");

    r.push_str("CSF REFERENSI (US EPA IRIS):\n");
    r.push_str("  Benzene      : 0.029  (inhalasi)\n");
    r.push_str("  Arsenic      : 1.5    (oral)\n");
    r.push_str("  Cr(VI)       : 42     (inhalasi)\n");
    r.push_str("  Vinyl chlor. : 0.72   (inhalasi)\n");
    r.push_str("  Benzo(a)pyr. : 7.3    (oral)\n");
    r.push_str("══════════════════════════════════════════════\n");

    r
}

// ══════════════════════════════════════════════════════════════
// 2) HQ — Non-cancer Hazard Quotient
// ══════════════════════════════════════════════════════════════

pub fn calculate_hq(
    contaminant: &str,
    route: &str,
    concentration: f64,
    intake_rate: f64,
    exposure_freq_days: f64,
    exposure_dur_years: f64,
    body_weight_kg: f64,
) -> String {
    if concentration < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }
    if intake_rate <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if exposure_freq_days <= 0.0 || exposure_freq_days > 365.0 {
        return "ERROR: Frekuensi paparan harus 0–365 hari/tahun.".into();
    }
    if exposure_dur_years <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if body_weight_kg <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    let route_lower = route.to_lowercase();
    let route_name = match route_lower.as_str() {
        "inhalation" | "inhalasi" => "Inhalasi",
        "ingestion" | "oral" => "Ingesti (oral)",
        "dermal" => "Dermal",
        _ => {
            return format!(
                "ERROR: Jalur paparan '{}' tidak dikenal.\nPilihan: inhalation/inhalasi, ingestion/oral, dermal",
                route
            );
        }
    };

    // Look up RfD from IRIS database
    let rfd_lookup = get_rfd(contaminant, &route_lower);
    let (rfd, rfd_source) = match rfd_lookup {
        Some((v, s)) => (v, s.to_string()),
        None => {
            return format!(
                "ERROR: RfD tidak ditemukan untuk kontaminan '{}' jalur '{}'.\n\
                 Kontaminan tersedia: arsenic/as, chromium_vi/cr6, cadmium/cd, mercury/hg,\n\
                 methylmercury/mehg, benzene, toluene, xylene/xylenes, phenol/fenol,\n\
                 formaldehyde/hcho, ammonia/nh3, vinyl_chloride\n\
                 Catatan: Lead/Pb tidak memiliki RfD dari IRIS.",
                contaminant, route
            );
        }
    };

    // CDI = (C × IR × EF × ED) / (BW × AT_nc × 365)
    // AT_nc = ED for non-cancer
    let at_nc_days = exposure_dur_years * 365.0;
    let cdi = (concentration * intake_rate * exposure_freq_days * exposure_dur_years)
        / (body_weight_kg * at_nc_days);

    // HQ = CDI / RfD
    let hq = cdi / rfd;

    let (risk_level, risk_desc) = if hq <= 1.0 {
        ("AMAN", "HQ ≤ 1 — risiko non-karsinogen dapat diterima")
    } else if hq <= 4.0 {
        (
            "PERLU PERHATIAN",
            "HQ 1–4 — risiko non-karsinogen moderat, perlu pengelolaan",
        )
    } else {
        (
            "TIDAK AMAN",
            "HQ > 4 — risiko non-karsinogen tinggi, tindakan segera diperlukan",
        )
    };

    // Indonesian BW comparison
    let cdi_id = (concentration * intake_rate * exposure_freq_days * exposure_dur_years)
        / (ARKL_BW_ADULT * at_nc_days);
    let hq_id = cdi_id / rfd;

    let mut r = String::new();
    r.push_str("══════════════════════════════════════════════\n");
    r.push_str("HAZARD QUOTIENT (HQ) — Risiko Non-Karsinogen\n");
    r.push_str("Ref: US EPA RAGS, Pedoman ARKL Kemenkes 2012\n");
    r.push_str("══════════════════════════════════════════════\n\n");

    r.push_str("FORMULA:\n");
    r.push_str("  CDI = (C × IR × EF × ED) / (BW × AT_nc × 365)\n");
    r.push_str("  AT_nc = ED (averaging time = durasi paparan untuk non-cancer)\n");
    r.push_str("  HQ = CDI / RfD\n\n");

    r.push_str("INPUT:\n");
    r.push_str(&format!("• Kontaminan               : {}\n", contaminant));
    r.push_str(&format!("• Jalur paparan            : {}\n", route_name));
    r.push_str(&format!(
        "• Konsentrasi (C)          : {:.6e}\n",
        concentration
    ));
    r.push_str(&format!(
        "• Laju intake (IR)         : {:.4}\n",
        intake_rate
    ));
    r.push_str(&format!(
        "• Frekuensi paparan (EF)   : {:.0} hari/tahun\n",
        exposure_freq_days
    ));
    r.push_str(&format!(
        "• Durasi paparan (ED)      : {:.1} tahun\n",
        exposure_dur_years
    ));
    r.push_str(&format!(
        "• Berat badan (BW)         : {:.1} kg\n",
        body_weight_kg
    ));
    r.push_str(&format!(
        "• Averaging time (AT_nc)   : {:.1} tahun (= ED)\n\n",
        exposure_dur_years
    ));

    r.push_str("RfD DATABASE:\n");
    r.push_str(&format!("• RfD                      : {:.6e}\n", rfd));
    r.push_str(&format!("• Sumber                   : {}\n\n", rfd_source));

    r.push_str("HASIL:\n");
    r.push_str(&format!(
        "• CDI (Chronic Daily Intake) : {:.6e} mg/kg/hari\n",
        cdi
    ));
    r.push_str(&format!("• HQ (Hazard Quotient)       : {:.4}\n\n", hq));

    r.push_str(&format!("TINGKAT RISIKO: {}\n", risk_level));
    r.push_str(&format!("  {}\n\n", risk_desc));

    // BW comparison
    if (body_weight_kg - ARKL_BW_ADULT).abs() > 0.5 {
        r.push_str("PERBANDINGAN BW:\n");
        r.push_str(&format!(
            "  BW={:.0} kg (input)   → HQ={:.4}\n",
            body_weight_kg, hq
        ));
        r.push_str(&format!(
            "  BW={:.0} kg (ARKL ID) → HQ={:.4}\n",
            ARKL_BW_ADULT, hq_id
        ));
        r.push_str(&format!(
            "  Catatan: BW Indonesia ({:.0} kg) menghasilkan CDI lebih tinggi daripada US EPA ({:.0} kg)\n\n",
            ARKL_BW_ADULT, US_EPA_BW_ADULT
        ));
    }

    r.push_str("INTERPRETASI HQ:\n");
    r.push_str("  HQ ≤ 1  : Aman — risiko dapat diterima\n");
    r.push_str("  HQ 1–4  : Perlu perhatian — monitoring intensif\n");
    r.push_str("  HQ > 4  : Tidak aman — tindakan segera diperlukan\n");
    r.push_str("══════════════════════════════════════════════\n");

    r
}

// ══════════════════════════════════════════════════════════════
// 3) ARKL — Full Indonesian Risk Assessment Calculator
// ══════════════════════════════════════════════════════════════

pub fn calculate_arkl(
    contaminant: &str,
    route: &str,
    concentration: f64,
    population_type: &str,
    exposure_scenario: &str,
) -> String {
    if concentration < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }

    let pop = population_type.to_lowercase();
    let scenario = exposure_scenario.to_lowercase();
    let route_lower = route.to_lowercase();

    // Select population defaults
    let (pop_label, bw, ir_inhal, ir_oral, ir_soil, sa) = match pop.as_str() {
        "dewasa" | "adult" => (
            "Dewasa",
            ARKL_BW_ADULT,
            ARKL_IR_INHAL_ADULT,
            ARKL_IR_ORAL_ADULT,
            ARKL_IR_SOIL_ADULT,
            ARKL_SA_ADULT,
        ),
        "anak" | "child" => (
            "Anak",
            ARKL_BW_CHILD,
            ARKL_IR_INHAL_CHILD,
            ARKL_IR_ORAL_CHILD,
            ARKL_IR_SOIL_CHILD,
            ARKL_SA_CHILD,
        ),
        _ => {
            return format!(
                "ERROR: Tipe populasi '{}' tidak dikenal.\nPilihan: dewasa/adult, anak/child",
                population_type
            );
        }
    };

    // Select exposure scenario defaults
    let (scenario_label, fe, dt) = match scenario.as_str() {
        "residensial" | "residential" => ("Residensial", ARKL_FE_RESIDENTIAL, ARKL_DT_RESIDENTIAL),
        "okupasional" | "occupational" => {
            ("Okupasional", ARKL_FE_OCCUPATIONAL, ARKL_DT_OCCUPATIONAL)
        }
        "sekolah" | "school" => ("Sekolah", ARKL_FE_SCHOOL, ARKL_DT_CHILD),
        _ => {
            return format!(
                "ERROR: Skenario paparan '{}' tidak dikenal.\nPilihan: residensial, okupasional, sekolah",
                exposure_scenario
            );
        }
    };

    // Override Dt for child population
    let dt_actual = if pop.as_str() == "anak" || pop.as_str() == "child" {
        ARKL_DT_CHILD
    } else {
        dt
    };

    // Select intake rate based on route
    let (route_name, ir) = match route_lower.as_str() {
        "inhalation" | "inhalasi" => ("Inhalasi", ir_inhal),
        "ingestion" | "oral" => ("Ingesti (oral/air minum)", ir_oral),
        "dermal" => ("Dermal", sa),
        "soil" | "tanah" => ("Ingesti tanah", ir_soil / 1_000_000.0), // convert mg to kg
        _ => {
            return format!(
                "ERROR: Jalur paparan '{}' tidak dikenal.\nPilihan: inhalation/inhalasi, ingestion/oral, dermal, soil/tanah",
                route
            );
        }
    };

    // ── CDI Calculation ──
    // Non-cancer: AT_nc = Dt (years)
    let at_nc_days = dt_actual * 365.0;
    let cdi_nc = (concentration * ir * fe * dt_actual) / (bw * at_nc_days);

    // Cancer: AT_cancer = 70 years
    let at_cancer_days = ARKL_AT_CANCER * 365.0;
    let cdi_cancer = (concentration * ir * fe * dt_actual) / (bw * at_cancer_days);

    // ── RfD lookup ──
    let rfd_result = get_rfd(contaminant, &route_lower);
    let (rfd_val, rfd_source, hq) = match rfd_result {
        Some((v, s)) => (Some(v), s.to_string(), Some(cdi_nc / v)),
        None => (None, "Tidak tersedia di database".to_string(), None),
    };

    // ── CSF lookup ──
    let csf_result = get_csf(contaminant, &route_lower);
    let (csf_val, csf_source, ilcr) = match csf_result {
        Some((v, s)) => (Some(v), s.to_string(), Some(cdi_cancer * v)),
        None => (None, "Tidak tersedia di database".to_string(), None),
    };

    // ── US EPA BW comparison ──
    let at_nc_us = dt_actual * 365.0;
    let cdi_nc_us = (concentration * ir * fe * dt_actual) / (US_EPA_BW_ADULT * at_nc_us);
    let hq_us = rfd_val.map(|r| cdi_nc_us / r);

    // ── Build output ──
    let mut r = String::new();
    r.push_str("══════════════════════════════════════════════════════════\n");
    r.push_str("ANALISIS RISIKO KESEHATAN LINGKUNGAN (ARKL)\n");
    r.push_str("Ref: Pedoman ARKL Kemenkes RI 2012, US EPA RAGS, PP 101/2014\n");
    r.push_str("══════════════════════════════════════════════════════════\n\n");

    // 1) Parameter pajanan
    r.push_str("1. PARAMETER PAJANAN (Default Indonesia — Pedoman Kemenkes 2012):\n");
    r.push_str(&format!("   Kontaminan              : {}\n", contaminant));
    r.push_str(&format!("   Jalur paparan           : {}\n", route_name));
    r.push_str(&format!("   Populasi                : {}\n", pop_label));
    r.push_str(&format!(
        "   Skenario                : {}\n",
        scenario_label
    ));
    r.push_str(&format!(
        "   Konsentrasi (C)         : {:.6e}\n",
        concentration
    ));
    r.push_str(&format!("   Laju intake (IR)        : {:.4}\n", ir));
    r.push_str(&format!(
        "   Frekuensi paparan (fE)  : {:.0} hari/tahun\n",
        fe
    ));
    r.push_str(&format!(
        "   Durasi paparan (Dt)     : {:.0} tahun\n",
        dt_actual
    ));
    r.push_str(&format!("   Berat badan (BW)        : {:.0} kg\n", bw));
    r.push_str(&format!(
        "   AT non-karsinogen       : {:.0} tahun (= Dt)\n",
        dt_actual
    ));
    r.push_str(&format!(
        "   AT karsinogen           : {:.0} tahun (WHO)\n\n",
        ARKL_AT_CANCER
    ));

    // 2) CDI
    r.push_str("2. PERHITUNGAN CDI:\n");
    r.push_str("   CDI = (C × IR × fE × Dt) / (BW × AT × 365)\n");
    r.push_str(&format!(
        "   CDI non-karsinogen      : {:.6e} mg/kg/hari\n",
        cdi_nc
    ));
    r.push_str(&format!(
        "   CDI karsinogen          : {:.6e} mg/kg/hari\n\n",
        cdi_cancer
    ));

    // 3) HQ (non-cancer)
    r.push_str("3. HAZARD QUOTIENT (Non-Karsinogen):\n");
    match (rfd_val, hq) {
        (Some(rfd), Some(hq_val)) => {
            r.push_str(&format!("   RfD                     : {:.6e}\n", rfd));
            r.push_str(&format!("   Sumber RfD              : {}\n", rfd_source));
            r.push_str(&format!("   HQ = CDI / RfD          : {:.4}\n", hq_val));
            if hq_val <= 1.0 {
                r.push_str("   Status                  : ✅ AMAN (HQ ≤ 1)\n\n");
            } else if hq_val <= 4.0 {
                r.push_str("   Status                  : ⚠️ PERLU PERHATIAN (HQ 1–4)\n\n");
            } else {
                r.push_str("   Status                  : ❌ TIDAK AMAN (HQ > 4)\n\n");
            }
        }
        _ => {
            r.push_str(&format!(
                "   RfD tidak tersedia untuk '{}' jalur '{}'\n",
                contaminant, route_name
            ));
            r.push_str("   HQ tidak dapat dihitung\n\n");
        }
    }

    // 4) ILCR (cancer)
    r.push_str("4. ILCR (Karsinogen):\n");
    match (csf_val, ilcr) {
        (Some(csf), Some(ilcr_val)) => {
            r.push_str(&format!("   CSF                     : {:.4}\n", csf));
            r.push_str(&format!("   Sumber CSF              : {}\n", csf_source));
            r.push_str(&format!("   ILCR = CDI × CSF        : {:.6e}\n", ilcr_val));
            if ilcr_val < 1e-6 {
                r.push_str("   Status                  : ✅ DAPAT DITERIMA (< 10⁻⁶)\n\n");
            } else if ilcr_val < 1e-4 {
                r.push_str("   Status                  : ⚠️ DAPAT DIKELOLA (10⁻⁶ – 10⁻⁴)\n\n");
            } else {
                r.push_str("   Status                  : ❌ TIDAK DAPAT DITERIMA (> 10⁻⁴)\n\n");
            }
        }
        _ => {
            r.push_str(&format!(
                "   CSF tidak tersedia untuk '{}' jalur '{}'\n",
                contaminant, route_name
            ));
            r.push_str(
                "   ILCR tidak dapat dihitung — kontaminan bukan karsinogen via jalur ini\n\n",
            );
        }
    }

    // 5) Comparison Indonesia vs US EPA
    r.push_str("5. PERBANDINGAN BW INDONESIA vs US EPA:\n");
    r.push_str(&format!(
        "   BW Indonesia (ARKL)     : {:.0} kg\n",
        ARKL_BW_ADULT
    ));
    r.push_str(&format!(
        "   BW US EPA               : {:.0} kg\n",
        US_EPA_BW_ADULT
    ));
    r.push_str(&format!(
        "   CDI (ID, BW={:.0})       : {:.6e}\n",
        ARKL_BW_ADULT, cdi_nc
    ));
    r.push_str(&format!(
        "   CDI (US, BW={:.0})       : {:.6e}\n",
        US_EPA_BW_ADULT, cdi_nc_us
    ));
    if let (Some(hq_val), Some(hq_us_val)) = (hq, hq_us) {
        r.push_str(&format!(
            "   HQ  (ID, BW={:.0})       : {:.4}\n",
            ARKL_BW_ADULT, hq_val
        ));
        r.push_str(&format!(
            "   HQ  (US, BW={:.0})       : {:.4}\n",
            US_EPA_BW_ADULT, hq_us_val
        ));
    }
    r.push_str("   ⚠ Menggunakan BW US EPA (70 kg) akan meremehkan CDI sebesar ~27%\n");
    r.push_str("     dan proporsional meremehkan risiko bagi populasi Indonesia.\n\n");

    // 6) Risk management recommendation
    r.push_str("6. REKOMENDASI PENGELOLAAN RISIKO (per Pedoman ARKL):\n");
    let has_risk = hq.map_or(false, |v| v > 1.0) || ilcr.map_or(false, |v| v >= 1e-6);
    if has_risk {
        r.push_str("   → Risiko teridentifikasi. Rekomendasi:\n");
        r.push_str("     a) Identifikasi sumber pencemar dan jalur pajanan utama\n");
        r.push_str("     b) Lakukan pengendalian sumber (source control)\n");
        r.push_str("     c) Kurangi frekuensi/durasi paparan (administrative control)\n");
        r.push_str("     d) Gunakan APD jika pajanan tidak dapat dihindari\n");
        r.push_str("     e) Monitoring berkala konsentrasi kontaminan\n");
        r.push_str("     f) Komunikasi risiko kepada masyarakat terdampak\n");
    } else {
        r.push_str("   → Risiko dalam batas aman. Rekomendasi:\n");
        r.push_str("     a) Lanjutkan monitoring berkala\n");
        r.push_str("     b) Evaluasi ulang jika ada perubahan sumber/aktivitas\n");
    }
    r.push_str("\n══════════════════════════════════════════════════════════\n");

    r
}
