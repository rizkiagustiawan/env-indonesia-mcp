use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

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
        return json!({"error": "E102", "message": "Konsentrasi tidak boleh negatif"}).to_string();
    }
    if intake_rate <= 0.0 {
        return json!({"error": "E102", "message": "Intake rate harus > 0"}).to_string();
    }
    if !(0.0..=365.0).contains(&exposure_freq_days) {
         return json!({"error": "E102", "message": "Frekuensi paparan harus 0-365 hari"}).to_string();
    }
    if exposure_dur_years <= 0.0 || body_weight_kg <= 0.0 || avg_time_years <= 0.0 || csf <= 0.0 {
        return json!({"error": "E102", "message": "Parameter tidak valid (harus > 0)"}).to_string();
    }

    let route_lower = exposure_route.to_lowercase();
    let (_route_name, _route_unit) = match route_lower.as_str() {
        "inhalation" | "inhalasi" => ("Inhalasi", "mg/m³ untuk C, m³/hari untuk IR"),
        "ingestion" | "oral" => ("Ingesti (oral)", "mg/kg untuk C, kg/hari untuk IR"),
        "dermal" => ("Dermal", "mg/cm² untuk C, cm² untuk IR"),
        _ => return json!({"error": "E100", "message": format!("Jalur paparan '{}' tidak dikenal", exposure_route)}).to_string()
    };

    let at_days = avg_time_years * 365.0;
    let cdi = (concentration * intake_rate * exposure_freq_days * exposure_dur_years) / (body_weight_kg * at_days);
    let ilcr = cdi * csf;

    let status = if ilcr < 1e-4 { ResultStatus::Valid } else { ResultStatus::ValidationFailed };

    let mut res = ScientificResult::new("ilcr", ilcr, "probability")
        .with_status(status)
        .with_provenance(Provenance::new("calculation", "EPA_RAGS", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("cdi_mg_kg_d", &cdi.to_string()));

    if ilcr < 1e-6 {
        res = res.with_claim(Claim::new("risk_level", "DAPAT DITERIMA (< 10⁻⁶)"));
    } else if ilcr < 1e-4 {
        res = res.with_claim(Claim::new("risk_level", "DAPAT DIKELOLA (10⁻⁶ – 10⁻⁴)"));
    } else {
        res = res.with_claim(Claim::new("risk_level", "TIDAK DAPAT DITERIMA (> 10⁻⁴)"));
    }

    json!([serde_json::from_str::<serde_json::Value>(&res.emit_validated()).unwrap()]).to_string()
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
    if concentration < 0.0 || intake_rate <= 0.0 || exposure_dur_years <= 0.0 || body_weight_kg <= 0.0 {
        return json!({"error": "E102", "message": "Parameter numerik tidak valid"}).to_string();
    }
    if !(0.0..=365.0).contains(&exposure_freq_days) {
         return json!({"error": "E102", "message": "Frekuensi paparan harus 0-365 hari"}).to_string();
    }

    let route_lower = route.to_lowercase();
    let _route_name = match route_lower.as_str() {
        "inhalation" | "inhalasi" => "Inhalasi",
        "ingestion" | "oral" => "Ingesti (oral)",
        "dermal" => "Dermal",
        _ => return json!({"error": "E100", "message": format!("Jalur paparan '{}' tidak dikenal", route)}).to_string()
    };

    let rfd_lookup = get_rfd(contaminant, &route_lower);
    let (rfd, rfd_source) = match rfd_lookup {
        Some((v, s)) => (v, s.to_string()),
        None => return json!({"error": "E104", "message": format!("RfD tidak ditemukan untuk '{}' jalur '{}'", contaminant, route)}).to_string()
    };

    let at_nc_days = exposure_dur_years * 365.0;
    let cdi = (concentration * intake_rate * exposure_freq_days * exposure_dur_years) / (body_weight_kg * at_nc_days);
    let hq = cdi / rfd;

    let status = if hq <= 1.0 { ResultStatus::Valid } else { ResultStatus::ValidationFailed };
    let risk_level = if hq <= 1.0 { "AMAN (≤ 1)" } else if hq <= 4.0 { "PERLU PERHATIAN (1-4)" } else { "TIDAK AMAN (> 4)" };

    let res = ScientificResult::new("hazard_quotient", hq, "dimensionless")
        .with_status(status)
        .with_provenance(Provenance::new("calculation", "EPA_RAGS", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("cdi_mg_kg_d", &cdi.to_string()))
        .with_claim(Claim::new("rfd", &rfd.to_string()))
        .with_claim(Claim::new("rfd_source", &rfd_source))
        .with_claim(Claim::new("risk_level", risk_level));

    json!([serde_json::from_str::<serde_json::Value>(&res.emit_validated()).unwrap()]).to_string()
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
        return json!({"error": "E102", "message": "Konsentrasi tidak boleh negatif"}).to_string();
    }

    let pop = population_type.to_lowercase();
    let scenario = exposure_scenario.to_lowercase();
    let route_lower = route.to_lowercase();

    let (_pop_label, bw, ir_inhal, ir_oral, ir_soil, sa) = match pop.as_str() {
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
        _ => return json!({"error": "E100", "message": format!("Tipe populasi '{}' tidak dikenal", population_type)}).to_string()
    };

    let (_scenario_label, fe, dt) = match scenario.as_str() {
        "residensial" | "residential" => ("Residensial", ARKL_FE_RESIDENTIAL, ARKL_DT_RESIDENTIAL),
        "okupasional" | "occupational" => ("Okupasional", ARKL_FE_OCCUPATIONAL, ARKL_DT_OCCUPATIONAL),
        "sekolah" | "school" => ("Sekolah", ARKL_FE_SCHOOL, ARKL_DT_CHILD),
        _ => return json!({"error": "E100", "message": format!("Skenario paparan '{}' tidak dikenal", exposure_scenario)}).to_string()
    };

    let dt_actual = if pop.as_str() == "anak" || pop.as_str() == "child" { ARKL_DT_CHILD } else { dt };

    let (_route_name, ir) = match route_lower.as_str() {
        "inhalation" | "inhalasi" => ("Inhalasi", ir_inhal),
        "ingestion" | "oral" => ("Ingesti (oral/air minum)", ir_oral),
        "dermal" => ("Dermal", sa),
        "soil" | "tanah" => ("Ingesti tanah", ir_soil / 1_000_000.0), // convert mg to kg
        _ => return json!({"error": "E100", "message": format!("Jalur paparan '{}' tidak dikenal", route)}).to_string()
    };

    let at_nc_days = dt_actual * 365.0;
    let cdi_nc = (concentration * ir * fe * dt_actual) / (bw * at_nc_days);

    let at_cancer_days = ARKL_AT_CANCER * 365.0;
    let cdi_cancer = (concentration * ir * fe * dt_actual) / (bw * at_cancer_days);

    let rfd_result = get_rfd(contaminant, &route_lower);
    let (rfd_val, rfd_source, hq) = match rfd_result {
        Some((v, s)) => (Some(v), s.to_string(), Some(cdi_nc / v)),
        None => (None, "Tidak tersedia".to_string(), None),
    };

    let csf_result = get_csf(contaminant, &route_lower);
    let (csf_val, csf_source, ilcr) = match csf_result {
        Some((v, s)) => (Some(v), s.to_string(), Some(cdi_cancer * v)),
        None => (None, "Tidak tersedia".to_string(), None),
    };

    let mut results = vec![];

    if let Some(hq_val) = hq {
        let status = if hq_val <= 1.0 { ResultStatus::Valid } else { ResultStatus::ValidationFailed };
        let mut res = ScientificResult::new("arkl_hazard_quotient", hq_val, "dimensionless")
            .with_status(status)
            .with_provenance(Provenance::new("calculation", "ARKL_ID_Kemenkes_2012", "2026-08-19T00:00:00Z"))
            .with_claim(Claim::new("rfd", &rfd_val.unwrap().to_string()))
            .with_claim(Claim::new("rfd_source", &rfd_source))
            .with_claim(Claim::new("indonesian_bw_kg", &bw.to_string()))
            .with_claim(Claim::new("cdi_nc_mg_kg_d", &cdi_nc.to_string()));
            
        if hq_val > 1.0 { res = res.with_claim(Claim::new("warning", "HQ > 1, unsafe non-carcinogenic risk")); }
        results.push(res);
    }

    if let Some(ilcr_val) = ilcr {
        let status = if ilcr_val < 1e-4 { ResultStatus::Valid } else { ResultStatus::ValidationFailed };
        let mut res = ScientificResult::new("arkl_ilcr", ilcr_val, "probability")
            .with_status(status)
            .with_provenance(Provenance::new("calculation", "ARKL_ID_Kemenkes_2012", "2026-08-19T00:00:00Z"))
            .with_claim(Claim::new("csf", &csf_val.unwrap().to_string()))
            .with_claim(Claim::new("csf_source", &csf_source))
            .with_claim(Claim::new("indonesian_bw_kg", &bw.to_string()))
            .with_claim(Claim::new("cdi_cancer_mg_kg_d", &cdi_cancer.to_string()));
            
        if ilcr_val >= 1e-4 { res = res.with_claim(Claim::new("warning", "ILCR >= 10^-4, unacceptable cancer risk")); }
        results.push(res);
    }

    if results.is_empty() {
        let res = ScientificResult::new("arkl_assessment", f64::NAN, "index")
            .with_status(ResultStatus::OutOfDomain)
            .with_provenance(Provenance::new("calculation", "ARKL_ID_Kemenkes_2012", "2026-08-19T00:00:00Z"))
            .with_claim(Claim::new("error", "No toxicity data (RfD/CSF) found for this contaminant and route."));
        results.push(res);
    }

    let json_array: Vec<serde_json::Value> = results.iter()
        .map(|r| serde_json::from_str(&r.clone().emit_validated()).unwrap())
        .collect();

    json!(json_array).to_string()
}
