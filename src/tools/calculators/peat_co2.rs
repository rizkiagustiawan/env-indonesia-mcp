use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Peat CO2 Emission Estimator
/// Ref: IPCC 2013 Wetlands Supplement; Hooijer et al. 2012; Page et al. 2002
/// Peat fire emits 10x more CO2 per ha than mineral soil fire.

pub fn calculate(
    burned_area_ha: f64,
    peat_depth_m: f64,
    severity_class: &str,
) -> String {
    if burned_area_ha < 0.0 || peat_depth_m < 0.0 {
        return json!({"error": "E102", "message": "Area dan kedalaman tidak boleh negatif"}).to_string();
    }

    let severity_factor = match severity_class.to_lowercase().as_str() {
        "low" => 0.5,
        "moderate" => 0.75,
        "high" | "severe" => 1.0,
        _ => 0.75,
    };

    let ef_co2_t_per_ha_per_m = 343.0;
    let ef_ch4_t_per_ha_per_m = 2.0;
    let ef_co_t_per_ha_per_m = 10.0;

    let co2 = burned_area_ha * peat_depth_m * ef_co2_t_per_ha_per_m * severity_factor;
    let ch4 = burned_area_ha * peat_depth_m * ef_ch4_t_per_ha_per_m * severity_factor;
    let co = burned_area_ha * peat_depth_m * ef_co_t_per_ha_per_m * severity_factor;
    let co2e = co2 + ch4 * 28.0 + co * 1.0;

    let mineral_soil_co2 = burned_area_ha * 3.0;
    let ratio = if mineral_soil_co2 > 0.0 { co2 / mineral_soil_co2 } else { 0.0 };

    let car_equivalent = co2e / 4.6;

    let res_co2e = ScientificResult::new("total_co2e", co2e, "tons")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "IPCC_2013_Wetlands", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("co2_tons", &co2.to_string()))
        .with_claim(Claim::new("ch4_tons", &ch4.to_string()))
        .with_claim(Claim::new("co_tons", &co.to_string()))
        .with_claim(Claim::new("severity_factor", &severity_factor.to_string()))
        .with_claim(Claim::new("context", &format!("Equivalent to {:.0} cars/year", car_equivalent)))
        .with_claim(Claim::new("context", &format!("{:.1}x more CO2 than mineral soil fire", ratio)))
        .with_claim(Claim::new("limitation", "Severity factor is approximate; actual burn depth varies. Page et al. 2002 measured higher EFs for Kalimantan."));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_co2e.emit_validated()).unwrap()
    ]).to_string()
}
