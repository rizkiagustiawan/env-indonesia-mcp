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
///   - Carbon price: Perpres 98/2021 (Nilai Ekonomi Karbon, NEK) Rp 465,000/tCO2e
///   - Discount rate 5% per standard environmental CBA (World Bank/ADB practice)

const IDR_PER_USD: f64 = 16_500.0;
const CARBON_PRICE_IDR_PER_TCO2: f64 = 465_000.0; // Perpres 98/2021 NEK
const DISCOUNT_RATE: f64 = 0.05;

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
            // 800 tCO2/ha over 20 yr (cumulative sequestration)
            let lifetime = 20.0_f64.max(years);
            let tons = 800.0 * area_ha;
            (tons, lifetime, "800 tCO2/ha over 20 yr (cumulative)".into())
        }
        "peatland" => {
            // 55 tCO2/ha/yr avoided emission (rewetting stops oxidation)
            let lifetime = years;
            let tons = 55.0 * area_ha * lifetime;
            (tons, lifetime, "55 tCO2/ha/yr avoided emission (rewetting)".into())
        }
        "mine" => {
            // Revegetation: ~10 tCO2/ha/yr sequestration once established
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
        _ => "",
    }
}

pub fn assess(
    restoration_type: &str,
    area_ha: f64,
    degradation_level: &str,
    years_since_degradation: f64,
    monitoring_years: f64,
) -> String {
    // ---- Validate ----
    if area_ha <= 0.0 {
        return "ERROR [E102]: area_ha must be > 0.".into();
    }
    if monitoring_years < 0.0 {
        return "ERROR [E102]: monitoring_years must be >= 0.".into();
    }
    if years_since_degradation < 0.0 {
        return "ERROR [E102]: years_since_degradation must be >= 0.".into();
    }

    let (cost_low, cost_mid, cost_high, unit_label, is_per_km) = match unit_cost(restoration_type)
    {
        Ok(c) => c,
        Err(e) => return format!("ERROR: {}", e),
    };
    let diff_mult = match difficulty_multiplier(degradation_level) {
        Ok(m) => m,
        Err(e) => return format!("ERROR: {}", e),
    };

    // Capital cost (mid estimate). For river, area_ha is interpreted as km (caller passes km).
    let area_label = if is_per_km { "km" } else { "ha" };
    let capital_low = cost_low * diff_mult * area_ha;
    let capital = cost_mid * diff_mult * area_ha;
    let capital_high = cost_high * diff_mult * area_ha;

    // Monitoring: 10% of capital per year, discounted PV over monitoring_years
    let monitoring_annual = 0.10 * capital;
    let monitoring_npv = pv_annuity(monitoring_annual, monitoring_years, DISCOUNT_RATE);

    // Total NPV (capital now + PV of monitoring)
    let total_npv = capital + monitoring_npv;

    // Carbon benefit
    let (carbon_tons, project_lifetime_yr, carbon_desc) = carbon_benefit(
        restoration_type,
        area_ha,
        monitoring_years.max(project_lifetime_min(restoration_type)),
    );
    let carbon_value = carbon_tons * CARBON_PRICE_IDR_PER_TCO2;
    let carbon_value_usd = carbon_value / IDR_PER_USD;

    // BCR (carbon benefit only — excludes other ecosystem services, flood protection, fisheries)
    let bcr = if total_npv > 0.0 { carbon_value / total_npv } else { 0.0 };

    // Payback period (years) — when cumulative carbon value equals total cost
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

    // Difficulty escalation note if years_since_degradation > 5
    let degradation_note = if years_since_degradation > 10.0 {
        format!("Severe legacy degradation ({} yr) - recontouring/invasive removal likely needed; cost may exceed upper bound.", years_since_degradation)
    } else if years_since_degradation > 5.0 {
        format!("Moderate legacy ({} yr) - add contingency ~20% for invasive species control.", years_since_degradation)
    } else {
        format!("Recent degradation ({} yr) - standard unit costs apply.", years_since_degradation)
    };

    let mut out = String::new();
    out.push_str("===============================================================\n");
    out.push_str("  ENVIRONMENTAL RESTORATION COST ESTIMATE\n");
    out.push_str("===============================================================\n");
    out.push_str("Method: Unit cost x area x difficulty + monitoring PV + carbon benefit (BCR)\n\n");

    out.push_str("INPUT:\n");
    out.push_str(&format!("  Restoration type    : {}\n", restoration_type.to_uppercase()));
    out.push_str(&format!("  Area                : {:.1} {}\n", area_ha, area_label));
    out.push_str(&format!(
        "  Degradation level   : {} (multiplier {:.1}x)\n",
        degradation_level, diff_mult
    ));
    out.push_str(&format!(
        "  Years since degrad. : {:.1}\n",
        years_since_degradation
    ));
    out.push_str(&format!("  Monitoring years    : {:.1}\n", monitoring_years));
    out.push_str(&format!(
        "  Discount rate       : {:.0}%\n\n",
        DISCOUNT_RATE * 100.0
    ));

    out.push_str("UNIT COST (Indonesia, 2026 IDR):\n");
    out.push_str(&format!(
        "  Range   : Rp{} - Rp{} {}\n",
        grp(cost_low),
        grp(cost_high),
        unit_label
    ));
    out.push_str(&format!("  Midpoint: Rp{} {}\n", grp(cost_mid), unit_label));
    out.push_str(&format!("  Source  : {}\n\n", cost_source(restoration_type)));

    out.push_str("COST BREAKDOWN:\n");
    out.push_str("  Capital (low/mid/high):\n");
    out.push_str(&format!("    Low    : Rp{}\n", grp(capital_low)));
    out.push_str(&format!("    MID    : Rp{}  <-- primary estimate\n", grp(capital)));
    out.push_str(&format!("    High   : Rp{}\n", grp(capital_high)));
    out.push_str(&format!("    USD mid: ${}\n", grp(capital / IDR_PER_USD)));
    out.push_str(&format!(
        "  Monitoring (10%/yr, {:.0} yr PV): Rp{}\n",
        monitoring_years,
        grp(monitoring_npv)
    ));
    out.push_str(&format!(
        "  TOTAL NPV (mid)   : Rp{}  (${})\n\n",
        grp(total_npv),
        grp(total_npv / IDR_PER_USD)
    ));

    out.push_str("CARBON BENEFIT:\n");
    out.push_str(&format!("  Method   : {}\n", carbon_desc));
    out.push_str(&format!("  Project life: {:.0} yr\n", project_lifetime_yr));
    out.push_str(&format!("  CO2 eq   : {} tCO2e\n", grp(carbon_tons)));
    out.push_str(&format!(
        "  Carbon price: Rp{}/tCO2e (Perpres 98/2021 NEK)\n",
        grp(CARBON_PRICE_IDR_PER_TCO2)
    ));
    out.push_str(&format!(
        "  Value    : Rp{}  (${})\n\n",
        grp(carbon_value),
        grp(carbon_value_usd)
    ));

    out.push_str("ECONOMIC INDICATORS:\n");
    out.push_str(&format!("  BCR (carbon only)  : {:.2}\n", bcr));
    let bcr_verdict = if bcr > 1.0 {
        "CARBON BENEFIT ALONE JUSTIFIES COST (BCR > 1)"
    } else if bcr > 0.5 {
        "Carbon benefit covers >50% of cost; co-benefits (fisheries, flood) needed for BCR>1"
    } else {
        "Carbon benefit insufficient alone; ecosystem service co-benefits required"
    };
    out.push_str(&format!("  Verdict            : {}\n", bcr_verdict));
    if payback_yr.is_finite() {
        out.push_str(&format!(
            "  Carbon payback     : {:.1} yr (vs project life {:.0} yr)\n",
            payback_yr, project_lifetime_yr
        ));
    } else {
        out.push_str("  Carbon payback     : N/A (no carbon benefit modeled)\n");
    }
    out.push_str(&format!("  Note               : {}\n\n", degradation_note));

    out.push_str("LIMITATIONS (honest assessment):\n");
    out.push_str("  1. Unit costs are published RANGES - site-specific (soil, access, labor)\n");
    out.push_str("     can shift cost by 2-3x. Mid estimate is a planning figure, not a quote.\n");
    out.push_str("  2. No contingency included (add 15-25% for implementation risk).\n");
    out.push_str("  3. No land acquisition / compensation cost (often dominant in Indonesia).\n");
    out.push_str("  4. Simplified carbon: uses fixed sequestration rate, no MRV/persistence,\n");
    out.push_str("     no reversal risk deduction. Real carbon credit value is lower.\n");
    out.push_str("  5. Carbon price Rp465k/tCO2e is the Perpres 98/2021 NEK reference value;\n");
    out.push_str("     actual market/ETS price may differ (Perpres 110/2025 supersedes 98/2021).\n");
    out.push_str("  6. BCR includes CARBON benefit only - excludes fisheries, flood protection,\n");
    out.push_str("     biodiversity, water purification (true BCR is higher, esp. mangrove).\n");
    out.push_str("  7. River cost is per km (pass area_ha=km value); difficulty multiplier\n");
    out.push_str("     assumes uniform degradation along reach - rarely true.\n");
    out.push_str("  8. Coral cost is per m2; area_ha here is interpreted as m2 for coral.\n");
    out.push_str("  9. Monitoring PV assumes constant 10%/yr - real O&M rises with inflation.\n");
    out.push_str(" 10. No lag for carbon accrual (mangrove sequestration ramps over years).\n");
    out.push_str("===============================================================\n");
    out
}

// ========================= SELF-CHECK TESTS =========================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_selfcheck_mangrove_moderate() {
        // Spec: mangrove 100ha, moderate -> 50M/ha * 1.5 * 100 = Rp 7.5B capital
        // + monitoring 5yr * 10% * 7.5B = 3.75B -> total 11.25B (NPV slightly less due to discounting)
        let out = assess("mangrove", 100.0, "moderate", 2.0, 5.0);
        assert!(!out.contains("ERROR"), "{}", out);
        // Capital mid = 50,000,000 * 1.5 * 100 = 7,500,000,000
        assert!(out.contains("7,500,000,000"), "capital should be Rp 7.5B, got: {}", out);
        // Carbon: 800 * 100 = 80,000 tCO2 * 465,000 = 37,200,000,000 (Rp 37.2T)
        assert!(out.contains("80,000 tCO2"), "should report 80,000 tCO2: {}", out);
        assert!(
            out.contains("37,200,000,000"),
            "carbon value should be Rp 37.2T: {}",
            out
        );
        assert!(out.contains("BCR"), "missing BCR");
        let capital = 50_000_000.0 * 1.5 * 100.0;
        let monitoring_npv = pv_annuity(0.10 * capital, 5.0, 0.05);
        let total = capital + monitoring_npv;
        let carbon_value = 80_000.0 * 465_000.0;
        let bcr = carbon_value / total;
        assert!(bcr > 1.0, "BCR should be > 1, got {}", bcr);
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
        assert!(out.contains("ERROR"));
        assert!(out.contains("Unknown restoration_type"));
    }

    #[test]
    fn test_negative_area_errors() {
        let out = assess("mangrove", -1.0, "light", 1.0, 5.0);
        assert!(out.contains("ERROR"));
    }

    #[test]
    fn test_peatland_carbon() {
        // 55 tCO2/ha/yr * 100 ha * 10 yr = 55,000 tCO2
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
        // 5% annuity of 1000 over 10 yr = 1000*(1-1.05^-10)/0.05 = 7,721.73
        assert!((pv_low - 7_721.73).abs() < 1.0);
    }

    #[test]
    fn test_all_types_run() {
        for t in ["mangrove", "peatland", "river", "mine", "coral"] {
            let out = assess(t, 50.0, "moderate", 3.0, 5.0);
            assert!(!out.contains("ERROR"), "type {} errored: {}", t, out);
            assert!(out.contains("TOTAL NPV"));
            assert!(out.contains("BCR"));
        }
    }

    #[test]
    fn test_limitations_present() {
        let out = assess("mangrove", 100.0, "light", 1.0, 5.0);
        assert!(out.contains("LIMITATIONS"));
        assert!(out.contains("contingency"));
        assert!(out.contains("land acquisition"));
    }

    #[test]
    fn test_degradation_note_scaling() {
        let recent = assess("mangrove", 100.0, "moderate", 2.0, 5.0);
        assert!(recent.contains("Recent degradation"));
        let moderate = assess("mangrove", 100.0, "moderate", 7.0, 5.0);
        assert!(moderate.contains("Moderate legacy"));
        let severe = assess("mangrove", 100.0, "moderate", 15.0, 5.0);
        assert!(severe.contains("Severe legacy"));
    }

    #[test]
    fn test_river_is_per_km() {
        // River: 1.25B/km mid * 1.0 (light) * 10 km = 12.5B
        let out = assess("river", 10.0, "light", 1.0, 5.0);
        assert!(out.contains("km"));
        assert!(out.contains("12,500,000,000"), "river capital should be 12.5B: {}", out);
    }

    #[test]
    fn test_grp_thousands_separator() {
        assert_eq!(grp(7_500_000_000.0), "7,500,000,000");
        assert_eq!(grp(1_234_567.0), "1,234,567");
        assert_eq!(grp(0.0), "0");
    }
}
