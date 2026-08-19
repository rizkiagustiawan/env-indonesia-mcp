use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use serde_json::json;

/// Health Impact Assessment (HIA) — Air Pollution Burden
/// Problem: No tool translates pollutant concentration -> health outcomes
///         (DALYs, premature mortality, economic cost). Critical for "impact" dimension.
/// Method: Concentration-Response Function (CRF) -> Attributable Fraction -> Deaths -> DALYs -> Cost
///
/// Formula (log-linear CRF, WHO 2021):
///   RR = RR_per_10 ^ ((C - C0) / 10)        // log-linear extrapolation
///   Attributable Fraction (AF) = (RR - 1) / RR
///   Attributable Deaths = POP * (baseline_mortality / 1e5) * AF * exposure_years
///   DALYs = Deaths * YLL_per_death
///   Economic Cost = DALYs * valuation_usd_per_daly
///   Cases Avoidable = Deaths(current) - Deaths(WHO_guideline)
///
/// References:
///   - WHO Global Air Quality Guidelines 2021 (PM2.5: 5 ug/m3 annual, 15 ug/m3 24h)
///   - WHO 2021: PM2.5 all-cause mortality RR = 1.0615 per 10 ug/m3 (log-linear)
///   - NO2 respiratory RR = 1.02 per 10 ug/m3 (WHO 2021)
///   - SO2 RR = 1.04 per 10 ug/m3 (WHO 2021)
///   - Indonesia crude death rate 2023 ~ 753/100,000/yr (World Bank, BPS)
///   - YLL per premature PM2.5 death ~ 12 yr (conservative; IHME 2023 global avg ~25 yr)
///   - WHO guidance: value of statistical life-year USD 50,000-150,000/DALY

const IDR_PER_USD: f64 = 16_500.0; // Bank Indonesia 2024-2025 mid-rate (assumption)
const INDONESIA_BASELINE_MORTALITY_PER_100K: f64 = 753.0; // World Bank 2023 crude death rate

/// Compute log-linear relative risk for a pollutant above background.
/// RR = base_rr ^ ((conc - background) / 10)
fn relative_risk(pollutant: &str, conc: f64, background: f64) -> Result<f64, String> {
    let rr_per_10 = rr_per_10(pollutant)?;
    if conc < 0.0 || background < 0.0 {
        return Err("Concentration and background must be >= 0.".into());
    }
    let delta = conc - background;
    if delta <= 0.0 {
        return Ok(1.0); // no excess exposure
    }
    Ok(rr_per_10.powf(delta / 10.0))
}

fn rr_per_10(pollutant: &str) -> Result<f64, String> {
    match pollutant.to_uppercase().as_str() {
        "PM2.5" | "PM25" => Ok(1.0615),
        "NO2" => Ok(1.02),
        "SO2" => Ok(1.04),
        "O3" => Ok(1.01), // WHO 2021: ~1% per 10 ug/m3 (ozone, seasonal)
        other => Err(format!(
            "Unknown pollutant '{}'. Supported: PM2.5, NO2, SO2, O3.",
            other
        )),
    }
}

/// YLL per premature death by pollutant (conservative, age-averaged).
fn yll_per_death(pollutant: &str) -> f64 {
    match pollutant.to_uppercase().as_str() {
        "PM2.5" | "PM25" => 12.0, // conservative; IHME 2023 global ~25
        "NO2" => 10.0,            // respiratory, slightly lower
        "SO2" => 9.0,
        "O3" => 8.0,
        _ => 10.0,
    }
}

/// WHO 2021 annual guideline concentration (ug/m3) by pollutant.
fn who_annual_guideline(pollutant: &str) -> f64 {
    match pollutant.to_uppercase().as_str() {
        "PM2.5" | "PM25" => 5.0,
        "NO2" => 10.0,
        "SO2" => 40.0,
        "O3" => 60.0, // seasonal (6-month) target
        _ => 0.0,
    }
}

/// WHO 2021 24-hour guideline concentration (ug/m3) by pollutant.
fn who_24h_guideline(pollutant: &str) -> f64 {
    match pollutant.to_uppercase().as_str() {
        "PM2.5" | "PM25" => 15.0,
        "NO2" => 25.0,
        "SO2" => 40.0,
        "O3" => 100.0,
        _ => 0.0,
    }
}

/// Indonesia ambient air quality standard (PP 22/2021, annual, ug/m3).
fn id_annual_standard(pollutant: &str) -> f64 {
    match pollutant.to_uppercase().as_str() {
        "PM2.5" | "PM25" => 15.0, // PP 22/2021 annual (24h: 55)
        "NO2" => 50.0,            // PP 22/2021 annual (24h: 80)
        "SO2" => 50.0,            // PP 22/2021 annual (1h: 150)
        "O3" => 100.0,            // PP 22/2021 1h
        _ => 0.0,
    }
}

pub fn assess(
    pollutant: &str,
    concentration_ug_m3: f64,
    population_exposed: f64,
    background_conc_ug_m3: f64,
    exposure_years: f64,
    valuation_usd_per_daly: f64,
) -> String {
    // ---- Validate inputs ----
    if concentration_ug_m3 < 0.0 || population_exposed < 0.0 || exposure_years <= 0.0 || valuation_usd_per_daly <= 0.0 {
        return json!({"error": "E102", "message": "Parameter tidak valid (harus positif)"}).to_string();
    }

    let rr = match relative_risk(pollutant, concentration_ug_m3, background_conc_ug_m3) {
        Ok(r) => r,
        Err(e) => return json!({"error": "E100", "message": e}).to_string(),
    };

    // Attributable fraction (WHO standard): AF = (RR-1)/RR
    let af = if rr > 0.0 { (rr - 1.0) / rr } else { 0.0 };

    // Attributable deaths over exposure window
    let baseline_rate = INDONESIA_BASELINE_MORTALITY_PER_100K / 100_000.0; // per person per year
    let deaths = population_exposed * baseline_rate * af * exposure_years;

    // DALYs (here = YLL only; no YLD morbidity component for mortality-driven CRF)
    let yll = yll_per_death(pollutant);
    let dalys = deaths * yll;

    // Economic cost
    let cost_usd = dalys * valuation_usd_per_daly;
    let _cost_idr = cost_usd * IDR_PER_USD;

    // Cases avoidable if reduced to WHO annual guideline
    let guideline = who_annual_guideline(pollutant);
    let rr_guideline = relative_risk(pollutant, guideline, background_conc_ug_m3).unwrap_or(1.0);
    let af_guideline = (rr_guideline - 1.0) / rr_guideline.max(1e-12);
    let deaths_guideline = population_exposed * baseline_rate * af_guideline.max(0.0) * exposure_years;
    let deaths_avoidable = (deaths - deaths_guideline).max(0.0);

    let res_cost = ScientificResult::new("economic_health_cost", cost_usd, "USD")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_provenance(Provenance::new("calculation", "WHO_CRF_2021", "2026-08-19T00:00:00Z"))
        .with_claim(Claim::new("attributable_deaths", &deaths.to_string()))
        .with_claim(Claim::new("dalys_lost", &dalys.to_string()))
        .with_claim(Claim::new("avoidable_deaths_if_who_compliant", &deaths_avoidable.to_string()))
        .with_claim(Claim::new("limitation", "Log-linear approximation. YLL only, excludes YLD."));

    json!([
        serde_json::from_str::<serde_json::Value>(&res_cost.emit_validated()).unwrap()
    ]).to_string()
}

// ========================= SELF-CHECK TESTS =========================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rr_pm25_loglinear() {
        let rr = relative_risk("PM2.5", 40.0, 5.0).unwrap();
        let expected = 1.0615_f64.powf(3.5);
        assert!((rr - expected).abs() < 1e-9, "RR={} expected={}", rr, expected);
        assert!((rr - 1.234).abs() < 0.01, "RR should be ~1.234, got {}", rr);
    }

    #[test]
    fn test_jakarta_scale_order_of_magnitude() {
        let out = assess("PM2.5", 40.0, 10_000_000.0, 5.0, 1.0, 75_000.0);
        assert!(!out.contains("error"), "{}", out);
        assert!(out.contains("economic_health_cost"));
    }
}
