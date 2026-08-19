use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Environmental Restoration Cost Estimator
/// Problem: No tool estimates restoration costs for mangrove/peatland/river/mine/coral sites.
/// Method: Unit cost x area x difficulty multiplier + monitoring (NPV) + carbon benefit (BCR)
///
/// Formulas:
///   capital = unit_cost_mid (IDR/ha) * difficulty_mult * area_ha
///   monitoring_annual = 0.10 * capital
///   monitoring_npv   = sum_{t=1..M} monitoring_annual / (1+r)^t   (r=0.05)
///   total_npv        = capital + monitoring_npv  (capex now + opex PV)
///   carbon_tons      = carbon_rate_per_ha * area_ha (mangrove: 800 tCO2 over 20yr;
///                      peatland: 55 tCO2/ha/yr avoided over monitoring window)
///   carbon_value     = carbon_tons * carbon_price (Rp 465,000/tCO2e, Perpres 98/2021)
///   BCR              = carbon_value / total_npv
///   payback (yr)     = total_npv / (carbon_value / project_lifetime_yr)
///
/// References:
///   - Mangrove: World Bank 2023 ($1,640-$3,900/ha Indonesia) ~ Rp 25-75M/ha; UNEP 2024
///   - Peatland: BRG (Badan Restorasi Gambut) 2017 Std Biaya; Rp 40-120M/ha
///   - River: Citarum Harum program; Rp 500M-2B/km
///   - Mine: Permen ESDM 26/2018 (reclamation guarantee); Rp 100-500M/ha
///   - Coral: Coremap/CTI; Rp 5-20M/m2 (artificial reef + transplantation)
///   - Carbon price: estimasi Nilai Ekonomi Karbon / SCC ~$30/tCO2e = Rp 465,000/tCO2e
///     (kerangka NEK diatur Perpres 98/2021; pajak karbon UU 7/2021 = Rp 30,000/tCO2e)
///   - Discount rate 5% per standard environmental CBA (World Bank/ADB practice)

const IDR_PER_USD: f64 = 16_500.0;
const CARBON_PRICE_IDR_PER_TCO2: f64 = 465_000.0; // Estimasi NEK/SCC (~$30/tCO2e), bukan pajak karbon
const DISCOUNT_RATE: f64 = 0.05;

/// Returns (unit_cost_low, unit_cost_mid, unit_cost_high, unit_label, is_per_km)
/// Prices in IDR (2026).
fn unit_cost(restoration_type: &str) -> Result<(f64, f64, f64, &'static str, bool), String> {
    match restoration_type.to_lowercase().as_str() {
        "mangrove" => Ok((25_000_000.0, 50_000_000.0, 75_000_000.0, "IDR/ha", false)),
        "peatland" => Ok((40_000_000.0, 80_000_000.0, 120_000_000.0, "IDR/ha", false)),
        "river" => Ok((500_000_000.0, 1_250_000_000.0, 2_000_000_000.0, "IDR/km", true)),
        "mine" => Ok((100_000_000.0, 300_000_000.0, 500_000_000.0, "IDR/ha", false)),
        "coral" => Ok((5_000_000.0, 12_500_000.0, 20_000_000.0, "IDR/m2", false)),
        other => Err(format!(
            "Unknown restoration_type '{}'. Supported: mangrove, peatland, river, mine, coral.",
            other
        )),
    }
}

fn difficulty_multiplier(level: &str) -> Result<f64, String> {
    match level.to_lowercase().as_str() {
        "light" => Ok(1.0),
        "moderate" => Ok(1.5),
        "severe" => Ok(2.5),
        other => Err(format!(
            "Unknown degradation_level '{}'. Use: light, moderate, severe.",
            other
        )),
    }
}

/// Carbon sequestration (tCO2) over the project lifetime.
/// Returns (total_tons, project_lifetime_yr, description).
fn carbon_benefit(restoration_type: &str, area_ha: f64, years: f64) -> (f64, f64, String) {
    match restoration_type.to_lowercase().as_str() {
        "mangrove" => {
            let lifetime = 20.0_f64.max(years);
            let tons = 800.0 * area_ha;
            (tons, lifetime, "800 tCO2/ha over 20 yr (cumulative)".into())
        }
        "peatland" => {
            let lifetime = years;
            let tons = 55.0 * area_ha * lifetime;
            (tons, lifetime, "55 tCO2/ha/yr avoided emission (rewetting)".into())
        }
        "mine" => {
            let lifetime = years;
            let tons = 10.0 * area_ha * lifetime;
            (tons, lifetime, "10 tCO2/ha/yr revegetation sequestration".into())
        }
        "river" => (0.0, years, "riparian carbon not monetized (negligible)".into()),
        "coral" => (0.0, years, "coral: no direct carbon credit currently".into()),
        _ => (0.0, years, "no carbon benefit modeled".into()),
    }
}

/// Present value of an annuity (monitoring cost over M years at rate r).
/// PV = A * (1 - (1+r)^-M) / r
fn pv_annuity(annual: f64, years: f64, r: f64) -> f64 {
    if years <= 0.0 || r <= 0.0 {
        return annual * years.max(0.0);
    }
    annual * (1.0 - (1.0 + r).powf(-years)) / r
}

fn project_lifetime_min(restoration_type: &str) -> f64 {
    match restoration_type.to_lowercase().as_str() {
        "mangrove" => 20.0,
        "peatland" => 10.0,
        "mine" => 10.0,
        _ => 5.0,
    }
}

fn cost_source(restoration_type: &str) -> &'static str {
    match restoration_type.to_lowercase().as_str() {
        "mangrove" => "World Bank 2023 ($1,640-$3,900/ha IDN); UNEP 2024; MoEF",
        "peatland" => "BRG 2017 Standar Biaya; Badan Restorasi Gambut",
        "river" => "Citarum Harum program; dit. Pengelolaan Sungai",
        "mine" => "Permen ESDM 26/2018 (jaminan reklamasi); KLHK",
        "coral" => "Coremap/CTI; LIPI/BRIN coral transplantation studies",
        _ => "Literature defaults",
    }
}

pub fn assess(
    restoration_type: &str,
    area_ha: f64,
    degradation_level: &str,
    years_since_degradation: f64,
    monitoring_years: f64,
) -> String {
    if area_ha <= 0.0 || monitoring_years < 0.0 || years_since_degradation < 0.0 {
        return json!({"error": "E102", "message": "Parameter tidak valid (harus positif)"}).to_string();
    }

    let (_cost_low, cost_mid, _cost_high, _unit_label, _is_per_km) = match unit_cost(restoration_type)
    {
        Ok(c) => c,
        Err(e) => return json!({"error": "E100", "message": e}).to_string(),
    };
    
    let diff_mult = match difficulty_multiplier(degradation_level) {
        Ok(m) => m,
        Err(e) => return json!({"error": "E100", "message": e}).to_string(),
    };

    let capital = cost_mid * diff_mult * area_ha;
    let monitoring_annual = 0.10 * capital;
    let monitoring_npv = pv_annuity(monitoring_annual, monitoring_years, DISCOUNT_RATE);
    let total_npv = capital + monitoring_npv;
    let total_npv_usd = total_npv / IDR_PER_USD;

    let (carbon_tons, project_lifetime_yr, carbon_desc) = carbon_benefit(
        restoration_type,
        area_ha,
        monitoring_years.max(project_lifetime_min(restoration_type)),
    );
    let carbon_value = carbon_tons * CARBON_PRICE_IDR_PER_TCO2;
    let _carbon_value_usd = carbon_value / IDR_PER_USD;

    let bcr = if total_npv > 0.0 { carbon_value / total_npv } else { 0.0 };

    let annual_carbon_value = if project_lifetime_yr > 0.0 {
        carbon_value / project_lifetime_yr
    } else {
        0.0
    };
    let payback_yr = if annual_carbon_value > 0.0 {
        total_npv / annual_carbon_value
    } else {
        f64::INFINITY
    };

    let degradation_note = if years_since_degradation > 10.0 {
        format!("Severe legacy degradation ({} yr) - recontouring/invasive removal likely needed; cost may exceed upper bound.", years_since_degradation)
    } else if years_since_degradation > 5.0 {
        format!("Moderate legacy ({} yr) - add contingency ~20% for invasive species control.", years_since_degradation)
    } else {
        format!("Recent degradation ({} yr) - standard unit costs apply.", years_since_degradation)
    };

    let res_npv = ScientificResult::new("total_npv_cost", total_npv_usd, "USD")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("calculation", "Unit_Cost_Extrapolation", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("capital_cost_idr", &capital.to_string()))
        .with_claim(Claim::new("monitoring_npv_idr", &monitoring_npv.to_string()))
        .with_claim(Claim::new("cost_source", cost_source(restoration_type)))
        .with_claim(Claim::new("degradation_note", &degradation_note));

    let mut res_bcr = ScientificResult::new("benefit_cost_ratio", bcr, "ratio")
        .with_status(ResultStatus::ScreeningOnly)
        .with_provenance(Provenance::new("calculation", "Carbon_Value_Perpres_98_2021", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("carbon_tons", &carbon_tons.to_string()))
        .with_claim(Claim::new("carbon_value_idr", &carbon_value.to_string()))
        .with_claim(Claim::new("carbon_method", &carbon_desc));

    if payback_yr.is_finite() {
        res_bcr = res_bcr.with_claim(Claim::new("payback_years", &payback_yr.to_string()));
    }

    json!([
        serde_json::from_str::<serde_json::Value>(&res_npv.emit_validated()).unwrap(),
        serde_json::from_str::<serde_json::Value>(&res_bcr.emit_validated()).unwrap()
    ]).to_string()
}

// ========================= SELF-CHECK TESTS =========================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_selfcheck_mangrove_moderate() {
        let out = assess("mangrove", 100.0, "moderate", 2.0, 5.0);
        assert!(!out.contains("error"), "{}", out);
        assert!(out.contains("benefit_cost_ratio"), "missing BCR");
    }

    #[test]
    fn test_difficulty_multipliers() {
        assert_eq!(difficulty_multiplier("light").unwrap(), 1.0);
        assert_eq!(difficulty_multiplier("moderate").unwrap(), 1.5);
        assert_eq!(difficulty_multiplier("severe").unwrap(), 2.5);
        assert!(difficulty_multiplier("extreme").is_err());
    }

    #[test]
    fn test_unit_cost_ranges() {
        let (lo, mid, hi, _, _) = unit_cost("mangrove").unwrap();
        assert!(lo < mid && mid < hi);
        assert_eq!(mid, 50_000_000.0);
        let (lo, _, hi, _, _) = unit_cost("peatland").unwrap();
        assert_eq!(lo, 40_000_000.0);
        assert_eq!(hi, 120_000_000.0);
    }

    #[test]
    fn test_unknown_type_errors() {
        let out = assess("forest", 100.0, "light", 1.0, 5.0);
        assert!(out.contains("error"));
    }

    #[test]
    fn test_negative_area_errors() {
        let out = assess("mangrove", -1.0, "light", 1.0, 5.0);
        assert!(out.contains("error"));
    }

    #[test]
    fn test_peatland_carbon() {
        let (tons, life, _) = carbon_benefit("peatland", 100.0, 10.0);
        assert_eq!(tons, 55_000.0);
        assert_eq!(life, 10.0);
    }

    #[test]
    fn test_mangrove_carbon_800_per_ha() {
        let (tons, _, _) = carbon_benefit("mangrove", 1.0, 20.0);
        assert_eq!(tons, 800.0);
    }

    #[test]
    fn test_pv_annuity_decreasing_with_rate() {
        let pv_low = pv_annuity(1000.0, 10.0, 0.05);
        let pv_high = pv_annuity(1000.0, 10.0, 0.10);
        assert!(pv_high < pv_low, "higher rate -> lower PV");
        assert!((pv_low - 7_721.73).abs() < 1.0);
    }

    #[test]
    fn test_all_types_run() {
        for t in ["mangrove", "peatland", "river", "mine", "coral"] {
            let out = assess(t, 50.0, "moderate", 3.0, 5.0);
            assert!(!out.contains("error"), "type {} errored: {}", t, out);
        }
    }
}
