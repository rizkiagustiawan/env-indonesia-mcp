use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Perhitungan Timbulan Lindi (Leachate) — Metode Neraca Air
/// Ref: PermenPU 3/2013, Tchobanoglous et al. (1993) Integrated Solid Waste Management

pub fn calculate(
    area_m2: f64,
    monthly_rainfall_mm: &[f64],
    monthly_et_mm: &[f64],
    soil_storage_mm: f64,
    runoff_coeff: f64,
) -> String {
    if area_m2 <= 0.0 {
        return json!({"error": "E102", "message": "Parameter area harus > 0"}).to_string();
    }
    if monthly_rainfall_mm.len() != 12 || monthly_et_mm.len() != 12 {
        return json!({"error": "E100", "message": "Data hujan dan ET harus persis 12 bulan"}).to_string();
    }
    if runoff_coeff < 0.0 || runoff_coeff > 1.0 {
        return json!({"error": "E102", "message": "Koefisien runoff harus antara 0 dan 1"}).to_string();
    }

    let mut storage = soil_storage_mm;
    let mut total_leachate_mm = 0.0;
    let mut total_rainfall = 0.0;
    let mut total_et = 0.0;
    let mut total_runoff = 0.0;

    for i in 0..12 {
        let p = monthly_rainfall_mm[i];
        let et = monthly_et_mm[i];
        let runoff = p * runoff_coeff;
        let infiltration = p - runoff - et;

        total_rainfall += p;
        total_et += et;
        total_runoff += runoff;

        let leachate = if infiltration > 0.0 {
            let excess = infiltration - (soil_storage_mm - storage).max(0.0);
            storage = (storage + infiltration).min(soil_storage_mm);
            excess.max(0.0)
        } else {
            storage = (storage + infiltration).max(0.0);
            0.0
        };

        total_leachate_mm += leachate;
    }

    let total_leachate_m3 = total_leachate_mm / 1000.0 * area_m2;
    let avg_daily_m3 = total_leachate_m3 / 365.0;

    let res_m3_yr = ScientificResult::new("annual_leachate_volume", total_leachate_m3, "m3/year")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("calculation", "WaterBalance_Tchobanoglous", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("methodology", "Monthly water balance assumes uniform routing. Excludes transit lag and layered Richards effects."));

    let res_daily = ScientificResult::new("avg_daily_leachate", avg_daily_m3, "m3/day")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("calculation", "WaterBalance_Tchobanoglous", "2026-08-19T00:00:00Z"));

    let res_total_rain = ScientificResult::new("total_rainfall", total_rainfall, "mm/year")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("input_aggregation", "Rainfall_Sum", "2026-08-19T00:00:00Z"));

    let res_total_et = ScientificResult::new("total_evapotranspiration", total_et, "mm/year")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("input_aggregation", "ET_Sum", "2026-08-19T00:00:00Z"));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_m3_yr.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_daily.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_total_rain.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_total_et.emit_validated()).unwrap()
    ]).to_string()
}
