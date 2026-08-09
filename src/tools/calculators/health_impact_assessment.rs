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

/// Thousands-grouped integer string (e.g. 1234567 -> "1,234,567").
fn grp(x: f64) -> String {
    let n = x.round() as i64;
    let s = n.to_string();
    let bytes = s.as_bytes();
    let neg = bytes.first() == Some(&b'-');
    let digits = if neg { &bytes[1..] } else { bytes };
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    let len = digits.len();
    for (i, b) in digits.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

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
    if concentration_ug_m3 < 0.0 {
        return "ERROR [E102]: concentration_ug_m3 must be >= 0.".into();
    }
    if population_exposed < 0.0 {
        return "ERROR [E102]: population_exposed must be >= 0.".into();
    }
    if exposure_years <= 0.0 {
        return "ERROR [E102]: exposure_years must be > 0.".into();
    }
    if valuation_usd_per_daly <= 0.0 {
        return "ERROR [E102]: valuation_usd_per_daly must be > 0.".into();
    }

    let rr = match relative_risk(pollutant, concentration_ug_m3, background_conc_ug_m3) {
        Ok(r) => r,
        Err(e) => return format!("ERROR: {}", e),
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
    let cost_idr = cost_usd * IDR_PER_USD;

    // Cases avoidable if reduced to WHO annual guideline
    let guideline = who_annual_guideline(pollutant);
    let rr_guideline = relative_risk(pollutant, guideline, background_conc_ug_m3).unwrap_or(1.0);
    let af_guideline = (rr_guideline - 1.0) / rr_guideline.max(1e-12);
    let deaths_guideline =
        population_exposed * baseline_rate * af_guideline.max(0.0) * exposure_years;
    let deaths_avoidable = (deaths - deaths_guideline).max(0.0);
    let dalys_avoidable = deaths_avoidable * yll;
    let cost_avoidable_usd = dalys_avoidable * valuation_usd_per_daly;

    // Cases avoidable if reduced to Indonesia PP 22/2021 standard
    let id_std = id_annual_standard(pollutant);
    let rr_id = relative_risk(pollutant, id_std, background_conc_ug_m3).unwrap_or(1.0);
    let af_id = (rr_id - 1.0) / rr_id.max(1e-12);
    let deaths_id = population_exposed * baseline_rate * af_id.max(0.0) * exposure_years;
    let deaths_avoidable_vs_id = (deaths - deaths_id).max(0.0);

    // Compliance checks
    let who_ann = who_annual_guideline(pollutant);
    let who_24 = who_24h_guideline(pollutant);
    let id_ann = id_annual_standard(pollutant);

    let mut out = String::new();
    out.push_str("===============================================================\n");
    out.push_str("  HEALTH IMPACT ASSESSMENT (HIA) - Air Pollution Burden\n");
    out.push_str("===============================================================\n");
    out.push_str("Method: Concentration-Response Function (log-linear, WHO 2021)\n");
    out.push_str("  RR = RR_per_10 ^ ((C - C0) / 10)\n");
    out.push_str("  AF = (RR - 1) / RR\n");
    out.push_str("  Deaths = POP * baseline_mort * AF * years\n");
    out.push_str("  DALYs = Deaths * YLL_per_death\n\n");

    out.push_str("INPUT:\n");
    out.push_str(&format!("  Pollutant              : {}\n", pollutant.to_uppercase()));
    out.push_str(&format!(
        "  Concentration          : {:.2} ug/m3\n",
        concentration_ug_m3
    ));
    out.push_str(&format!(
        "  Background (C0)        : {:.2} ug/m3\n",
        background_conc_ug_m3
    ));
    out.push_str(&format!(
        "  Population exposed     : {}\n",
        grp(population_exposed)
    ));
    out.push_str(&format!("  Exposure duration      : {:.1} years\n", exposure_years));
    out.push_str(&format!(
        "  Valuation              : USD {}/DALY\n",
        grp(valuation_usd_per_daly)
    ));
    out.push_str(&format!(
        "  Baseline mortality     : {:.0}/100,000/yr (Indonesia 2023, World Bank)\n\n",
        INDONESIA_BASELINE_MORTALITY_PER_100K
    ));

    out.push_str("CONCENTRATION-RESPONSE (CRF):\n");
    out.push_str(&format!(
        "  RR per 10 ug/m3        : {:.4} (WHO 2021, {})\n",
        rr_per_10(pollutant).unwrap_or(1.0),
        pollutant.to_uppercase()
    ));
    out.push_str(&format!(
        "  Excess concentration   : {:.2} ug/m3 (C - C0)\n",
        (concentration_ug_m3 - background_conc_ug_m3).max(0.0)
    ));
    out.push_str(&format!("  Relative Risk (RR)     : {:.4}\n", rr));
    out.push_str(&format!(
        "  Attributable Fraction  : {:.4} ({:.2}%)\n\n",
        af, af * 100.0
    ));

    out.push_str("HEALTH BURDEN (at current concentration):\n");
    out.push_str(&format!(
        "  Attributable deaths    : {} over {:.1} yr\n",
        grp(deaths),
        exposure_years
    ));
    out.push_str(&format!(
        "  Deaths per year         : {}\n",
        grp(deaths / exposure_years)
    ));
    out.push_str(&format!("  YLL per death           : {:.0} yr\n", yll));
    out.push_str(&format!("  DALYs (YLL only)        : {}\n\n", grp(dalys)));

    out.push_str("ECONOMIC COST:\n");
    out.push_str(&format!("  USD                    : ${}\n", grp(cost_usd)));
    out.push_str(&format!(
        "  IDR (@ Rp{}/USD)       : Rp{}\n\n",
        grp(IDR_PER_USD),
        grp(cost_idr)
    ));

    out.push_str("MITIGATION - CASES AVOIDABLE:\n");
    out.push_str(&format!(
        "  Reduce to WHO annual guideline ({:.0} ug/m3):\n",
        who_ann
    ));
    out.push_str(&format!(
        "    Deaths avoidable     : {}\n",
        grp(deaths_avoidable)
    ));
    out.push_str(&format!("    DALYs avoidable      : {}\n", grp(dalys_avoidable)));
    out.push_str(&format!(
        "    Economic benefit     : ${}\n",
        grp(cost_avoidable_usd)
    ));
    out.push_str(&format!(
        "  Reduce to PP 22/2021 ({:.0} ug/m3):\n",
        id_ann
    ));
    out.push_str(&format!(
        "    Deaths avoidable     : {}\n\n",
        grp(deaths_avoidable_vs_id)
    ));

    out.push_str("COMPLIANCE CHECK:\n");
    out.push_str(&format!(
        "  vs WHO 2021 annual ({:>5.0} ug/m3): {}\n",
        who_ann,
        if concentration_ug_m3 > who_ann {
            "EXCEEDED"
        } else {
            "OK"
        }
    ));
    out.push_str(&format!(
        "  vs WHO 2021 24h    ({:>5.0} ug/m3): {} (if 24h avg ~ annual)\n",
        who_24,
        if concentration_ug_m3 > who_24 {
            "EXCEEDED"
        } else {
            "OK"
        }
    ));
    out.push_str(&format!(
        "  vs PP 22/2021 annual({:>5.0} ug/m3): {}\n\n",
        id_ann,
        if concentration_ug_m3 > id_ann {
            "EXCEEDED"
        } else {
            "OK"
        }
    ));

    out.push_str("INDONESIA CONTEXT:\n");
    out.push_str("  - Jakarta PM2.5 annual ~40 ug/m3 (2023, IQAir) - 8x WHO guideline.\n");
    out.push_str("  - Published estimates: Jakarta PM2.5 attributable deaths ~10,000-15,000/yr.\n");
    out.push_str("  - This tool's order-of-magnitude should match for Jakarta-scale inputs.\n\n");

    out.push_str("LIMITATIONS (honest assessment):\n");
    out.push_str("  1. CRF is a log-linear approximation; real exposure-response is non-linear\n");
    out.push_str("     (supra-linear at low conc, sub-linear at high conc per Burnett et al. 2018).\n");
    out.push_str("  2. RR=1.0615 per 10 ug/m3 is the lower bound; meta-analysis (SSPH 2024)\n");
    out.push_str("     reports RR=1.095 (1.064-1.127) - true burden may be ~1.5x higher.\n");
    out.push_str("  3. Baseline mortality is national avg (753/100k), NOT local - Jakarta/Surabaya\n");
    out.push_str("     age-stratified rates differ; no age stratification applied here.\n");
    out.push_str("  4. YLL=12 yr/death is conservative; IHME 2023 global PM2.5 avg ~25 YLL/death.\n");
    out.push_str("  5. No co-morbidity adjustment (CVD, COPD amplify risk).\n");
    out.push_str("  6. DALYs here = YLL only (mortality); excludes YLD (morbidity/hospitalization).\n");
    out.push_str("  7. Background C0 is assumed constant; real urban background varies seasonally.\n");
    out.push_str("  8. No threshold assumption (WHO 2021: no safe PM2.5 threshold);\n");
    out.push_str("     setting C0=WHO guideline gives 'avoidable' estimate.\n");
    out.push_str("  9. Exchange rate Rp16,500/USD is an assumption (BI 2024-2025 range 15,500-16,800).\n");
    out.push_str(" 10. Valuation USD/DALY is normative, not measured - use sensitivity analysis.\n");
    out.push_str("===============================================================\n");
    out
}

// ========================= SELF-CHECK TESTS =========================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rr_pm25_loglinear() {
        // PM2.5 40 ug/m3, background 5 (WHO annual): RR = 1.0615^((40-5)/10) = 1.0615^3.5
        let rr = relative_risk("PM2.5", 40.0, 5.0).unwrap();
        let expected = 1.0615_f64.powf(3.5);
        assert!((rr - expected).abs() < 1e-9, "RR={} expected={}", rr, expected);
        // ~1.234 per spec
        assert!((rr - 1.234).abs() < 0.01, "RR should be ~1.234, got {}", rr);
    }

    #[test]
    fn test_attributable_fraction() {
        // AF = (RR-1)/RR for RR=1.234 -> AF ~ 0.19
        let rr = 1.0615_f64.powf(3.5);
        let af = (rr - 1.0) / rr;
        assert!((af - 0.19).abs() < 0.01, "AF should be ~0.19, got {}", af);
    }

    #[test]
    fn test_jakarta_scale_order_of_magnitude() {
        // Spec self-check: PM2.5=40, POP=10M, 1 yr -> ~13,300 deaths (order 10k-15k)
        let out = assess("PM2.5", 40.0, 10_000_000.0, 5.0, 1.0, 75_000.0);
        assert!(!out.contains("ERROR"), "{}", out);
        assert!(out.contains("Attributable deaths"));
        // 10M * (753/1e5) * 0.19 * 1 = 14,307 — within Jakarta published range
        let rr = 1.0615_f64.powf(3.5);
        let af = (rr - 1.0) / rr;
        let deaths = 10_000_000.0 * (753.0 / 100_000.0) * af * 1.0;
        assert!(
            deaths > 8_000.0 && deaths < 20_000.0,
            "deaths={} should be 8k-20k",
            deaths
        );
        // The grp() string should appear
        assert!(out.contains(&grp(deaths)), "deaths {} not in output", grp(deaths));
    }

    #[test]
    fn test_no_excess_when_at_background() {
        let rr = relative_risk("PM2.5", 5.0, 5.0).unwrap();
        assert!((rr - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_unknown_pollutant_errors() {
        assert!(relative_risk("CO2", 100.0, 0.0).is_err());
    }

    #[test]
    fn test_negative_concentration_errors() {
        let out = assess("PM2.5", -1.0, 1000.0, 0.0, 1.0, 50_000.0);
        assert!(out.contains("ERROR"));
    }

    #[test]
    fn test_no2_so2_o3_supported() {
        for p in ["NO2", "SO2", "O3"] {
            let out = assess(p, 30.0, 100_000.0, 0.0, 1.0, 50_000.0);
            assert!(!out.contains("ERROR"), "pollutant {} failed: {}", p, out);
            assert!(out.contains("DALYs"));
        }
    }

    #[test]
    fn test_dalys_equal_deaths_times_yll() {
        let out = assess("PM2.5", 40.0, 1_000_000.0, 5.0, 1.0, 75_000.0);
        // DALYs = deaths * 12 (YLL for PM2.5)
        let rr = 1.0615_f64.powf(3.5);
        let af = (rr - 1.0) / rr;
        let deaths = 1_000_000.0 * (753.0 / 100_000.0) * af;
        let dalys = deaths * 12.0;
        assert!(out.contains(&grp(deaths)), "deaths {} not in output", grp(deaths));
        assert!(out.contains(&grp(dalys)), "dalys {} not in output", grp(dalys));
    }

    #[test]
    fn test_cases_avoidable_present() {
        let out = assess("PM2.5", 40.0, 10_000_000.0, 5.0, 1.0, 75_000.0);
        assert!(out.contains("Deaths avoidable"));
        assert!(out.contains("WHO annual guideline"));
        assert!(out.contains("PP 22/2021"));
    }

    #[test]
    fn test_economic_cost_present() {
        let out = assess("PM2.5", 40.0, 1_000_000.0, 5.0, 1.0, 75_000.0);
        assert!(out.contains("USD"));
        assert!(out.contains("IDR"));
    }

    #[test]
    fn test_limitations_section_present() {
        let out = assess("PM2.5", 40.0, 1_000_000.0, 5.0, 1.0, 75_000.0);
        assert!(out.contains("LIMITATIONS"));
        assert!(out.contains("non-linear"));
    }

    #[test]
    fn test_compliance_check_present() {
        let out = assess("PM2.5", 40.0, 1_000_000.0, 5.0, 1.0, 75_000.0);
        assert!(out.contains("COMPLIANCE CHECK"));
        assert!(out.contains("WHO 2021 annual"));
        assert!(out.contains("EXCEEDED"));
    }

    #[test]
    fn test_grp_thousands_separator() {
        assert_eq!(grp(1_234_567.0), "1,234,567");
        assert_eq!(grp(100.0), "100");
        assert_eq!(grp(0.0), "0");
        assert_eq!(grp(-1_234.0), "-1,234");
    }
}
