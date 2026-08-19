use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Peluruhan Coliform — Model Mancini
/// Ref: Mancini (1978), PP 22/2021 tentang Baku Mutu Air

pub fn calculate(
    initial_count_per_100ml: f64,
    temperature_c: f64,
    time_hours: f64,
    water_type: &str,
) -> String {
    if initial_count_per_100ml <= 0.0 {
        return json!({"error": "E102", "message": "Parameter initial_count harus > 0"}).to_string();
    }
    if !(0.0..=45.0).contains(&temperature_c) {
        return json!({"error": "E102", "message": "Suhu harus antara 0 dan 45 °C"}).to_string();
    }
    if time_hours < 0.0 {
        return json!({"error": "E102", "message": "Parameter waktu tidak boleh negatif"}).to_string();
    }

    let wt_lower = water_type.to_lowercase();

    // Mancini model: k = k_base × θ^(T-20)
    // k in day⁻¹, θ = 1.07 (temperature coefficient)
    let (k_base, _t90_base_hr, _water_name) = match wt_lower.as_str() {
        "freshwater" | "air_tawar" | "sungai" => (0.8, 60.0, "Air Tawar (Freshwater)"),
        "seawater" | "air_laut" | "laut" => (2.0, 36.0, "Air Laut (Seawater)"),
        "tropical" | "tropis" | "pantai_tropis" => (3.0, 18.0, "Perairan Tropis (Tropical)"),
        "estuari" | "estuary" | "muara" => (1.5, 48.0, "Estuari (Estuary)"),
        _ => {
            return json!({"error": "E100", "message": format!("Tipe perairan '{}' tidak dikenal", water_type)}).to_string();
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
    let limit_class1 = 1000.0; // per 100 mL
    let limit_class2 = 5000.0;
    let limit_class3 = 10000.0;

    let comply_class1 = remaining <= limit_class1;

    let res_remaining = ScientificResult::new("remaining_coliform", remaining, "count/100mL")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "Mancini_1978", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("methodology", "Exponential decay based on temperature and water type"));

    let res_t90 = ScientificResult::new("t90_die_off_time", t90_hr, "hours")
        .with_status(ResultStatus::Valid)
        .with_provenance(Provenance::new("calculation", "Mancini_1978", "2026-08-19T00:00:00Z"));
        
    let mut res_log_removal = ScientificResult::new("log_removal", log_removal, "log10")
        .with_status(ResultStatus::Valid)
        .with_provenance(Provenance::new("calculation", "Mancini_1978", "2026-08-19T00:00:00Z"));
        
    if !log_removal.is_finite() {
        res_log_removal.value = f64::NAN;
        res_log_removal.status = ResultStatus::OutOfDomain;
    }

    let compliance_score = if comply_class1 { 1.0 } else if remaining <= limit_class2 { 2.0 } else if remaining <= limit_class3 { 3.0 } else { 4.0 };
    
    let res_compliance = ScientificResult::new("pp22_2021_class_compliance", compliance_score, "class_tier")
        .with_status(if comply_class1 { ResultStatus::Valid } else { ResultStatus::ScreeningOnly })
        .with_provenance(Provenance::new("regulatory_limit", "PP_22_2021", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("class_1_limit", "1000 count/100mL"))
        .with_claim(Claim::new("class_2_limit", "5000 count/100mL"))
        .with_claim(Claim::new("class_3_limit", "10000 count/100mL"));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_remaining.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_t90.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_log_removal.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_compliance.emit_validated()).unwrap()
    ]).to_string()
}
