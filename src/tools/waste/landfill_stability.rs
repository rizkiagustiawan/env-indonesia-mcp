use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Stabilitas Lereng TPA (Simplified Bishop / Infinite Slope)
/// Ref: PermenPU 3/2013, Das (2010) Principles of Geotechnical Engineering

pub fn calculate(
    slope_angle_deg: f64,
    height_m: f64,
    unit_weight_kn_m3: f64,
    cohesion_kpa: f64,
    friction_deg: f64,
    pore_pressure_ratio: f64,
) -> String {
    if !(0.0..90.0).contains(&slope_angle_deg) {
        return json!({"error": "E102", "message": "Sudut lereng harus antara 0 dan 90 derajat (eksklusif)."}).to_string();
    }
    if height_m <= 0.0 {
        return json!({"error": "E102", "message": "Parameter tinggi harus > 0."}).to_string();
    }
    if unit_weight_kn_m3 <= 0.0 {
        return json!({"error": "E102", "message": "Parameter berat isi harus > 0."}).to_string();
    }
    if cohesion_kpa < 0.0 {
        return json!({"error": "E102", "message": "Parameter kohesi tidak boleh negatif."}).to_string();
    }
    if !(0.0..=60.0).contains(&friction_deg) {
        return json!({"error": "E102", "message": "Sudut geser internal harus antara 0 dan 60 derajat."}).to_string();
    }
    if !(0.0..=1.0).contains(&pore_pressure_ratio) {
        return json!({"error": "E102", "message": "Rasio tekanan pori (ru) harus antara 0 dan 1."}).to_string();
    }

    let alpha = slope_angle_deg.to_radians();
    let phi = friction_deg.to_radians();

    // Infinite slope analysis:
    // FoS = c'/(γ×H×sin(α)×cos(α)) + tan(φ')/tan(α) - ru×tan(φ')/tan(α)
    // FoS = c'/(γ×H×sin(α)×cos(α)) + (1 - ru)×tan(φ')/tan(α)
    let sin_a = alpha.sin();
    let cos_a = alpha.cos();
    let tan_a = alpha.tan();
    let tan_phi = phi.tan();

    let cohesion_term = if (sin_a * cos_a) > 1e-12 {
        cohesion_kpa / (unit_weight_kn_m3 * height_m * sin_a * cos_a)
    } else {
        f64::INFINITY
    };

    let friction_term = if tan_a.abs() > 1e-12 {
        (1.0 - pore_pressure_ratio) * tan_phi / tan_a
    } else {
        f64::INFINITY
    };

    let fos = cohesion_term + friction_term;

    // Status per PermenPU 3/2013
    let status = if fos >= 1.5 {
        ResultStatus::Valid
    } else if fos >= 1.3 {
        ResultStatus::ValidWithAssumptions
    } else if fos >= 1.1 {
        ResultStatus::ScreeningOnly
    } else {
        ResultStatus::ValidationFailed
    };

    // Recommended maximum slope angle for FoS = 1.3
    // Solve: 1.3 = c'/(γ×H×sin(α)×cos(α)) + (1-ru)×tan(φ)/tan(α)
    // Iterative approach
    let mut recommended_angle = slope_angle_deg;
    if fos < 1.3 {
        for test_deg in (5..=85).rev() {
            let test_rad = (test_deg as f64).to_radians();
            let sa = test_rad.sin();
            let ca = test_rad.cos();
            let ta = test_rad.tan();
            let c_t = if (sa * ca) > 1e-12 {
                cohesion_kpa / (unit_weight_kn_m3 * height_m * sa * ca)
            } else {
                f64::INFINITY
            };
            let f_t = if ta.abs() > 1e-12 {
                (1.0 - pore_pressure_ratio) * tan_phi / ta
            } else {
                f64::INFINITY
            };
            let test_fos = c_t + f_t;
            if test_fos >= 1.3 {
                recommended_angle = test_deg as f64;
                break;
            }
        }
    }

    // Critical height for FoS = 1.0 at given angle
    // 1.0 = c'/(γ×Hc×sin(α)×cos(α)) + (1-ru)×tan(φ)/tan(α)
    let friction_only = friction_term; // independent of H
    let h_critical = if friction_only < 1.0 && (sin_a * cos_a) > 1e-12 {
        cohesion_kpa / ((1.0 - friction_only) * unit_weight_kn_m3 * sin_a * cos_a)
    } else {
        f64::INFINITY // friction alone provides FoS ≥ 1.0
    };

    let res_fos = ScientificResult::new("factor_of_safety", fos, "dimensionless")
        .with_status(status.clone())
        .with_provenance(Provenance::new("calculation", "InfiniteSlope_Das_2010", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("methodology", "Infinite Slope Analysis"));

    let res_rec_angle = ScientificResult::new("recommended_max_slope_angle", recommended_angle, "degrees")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "PermenPU_3_2013_FoS_1_3", "2026-08-19T00:00:00Z"));

    let mut res_h_crit = ScientificResult::new("critical_height", h_critical, "m")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "InfiniteSlope_Das_2010", "2026-08-19T00:00:00Z"));
    
    if !h_critical.is_finite() {
        // Rust's JSON serialization of f64::INFINITY isn't strictly standard, but commonly emitted as "inf". 
        // Our validator rejects non-finite values. If it's infinite, friction alone is sufficient. We cap to a very large number or nan and set status OutOfDomain.
        res_h_crit.value = f64::NAN; 
        res_h_crit.status = ResultStatus::OutOfDomain;
    }

    let output = if h_critical.is_finite() {
        json!([
            serde_json::from_str::<serde_json::Value>(&res_fos.emit_validated()).unwrap(),
            serde_json::from_str::<serde_json::Value>(&res_rec_angle.emit_validated()).unwrap(),
            serde_json::from_str::<serde_json::Value>(&res_h_crit.emit_validated()).unwrap()
        ])
    } else {
         json!([
            serde_json::from_str::<serde_json::Value>(&res_fos.emit_validated()).unwrap(),
            serde_json::from_str::<serde_json::Value>(&res_rec_angle.emit_validated()).unwrap()
        ])
    };

    output.to_string()
}
