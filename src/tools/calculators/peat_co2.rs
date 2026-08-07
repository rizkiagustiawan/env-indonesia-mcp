/// Peat CO2 Emission Estimator
/// Ref: IPCC 2013 Wetlands Supplement; Hooijer et al. 2012; Page et al. 2002
/// Peat fire emits 10x more CO2 per ha than mineral soil fire.

pub fn calculate(
    burned_area_ha: f64,
    peat_depth_m: f64,
    severity_class: &str,
) -> String {
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Peat Fire CO2 Emission Estimate\n");
    out.push_str("Ref: IPCC 2013 Wetlands Supplement; Hooijer et al. 2012; Page et al. 2002\n\n");

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
    let indonesia_annual_co2 = 692_000_000.0;
    let pct_national = (co2e / indonesia_annual_co2) * 100.0;

    out.push_str(&format!("Burned area: {:.1} ha\n", burned_area_ha));
    out.push_str(&format!("Peat depth: {:.1} m\n", peat_depth_m));
    out.push_str(&format!("Severity: {} (factor: {:.2})\n\n", severity_class, severity_factor));

    out.push_str("EMISSIONS:\n");
    out.push_str(&format!("  CO2:  {:.1} tons\n", co2));
    out.push_str(&format!("  CH4:  {:.1} tons (GWP-100: 28x)\n", ch4));
    out.push_str(&format!("  CO:   {:.1} tons (GWP-100: 1x)\n", co));
    out.push_str(&format!("  CO2e: {:.1} tons (total)\n\n", co2e));

    out.push_str("CONTEXT:\n");
    out.push_str(&format!("  vs mineral soil fire: {:.1}x more CO2\n", ratio));
    out.push_str(&format!("  Equivalent to {:.0} cars/year\n", car_equivalent));
    out.push_str(&format!("  Indonesia annual CO2: ~692 Mt → this fire = {:.4}%\n\n", pct_national));

    out.push_str("METHODOLOGY:\n");
    out.push_str("  IPCC 2013 Wetlands Supplement Table 2.7 (EF for peat fires)\n");
    out.push_str("  CO2 EF = 343 t/ha/m peat depth (IPCC default)\n");
    out.push_str("  CH4 EF = 2 t/ha/m; CO EF = 10 t/ha/m\n\n");

    out.push_str("LIMITATION:\n");
    out.push_str("  - Peat depth varies 0.5-12m in Indonesia — value is site-specific\n");
    out.push_str("  - IPCC EF is global default — Indonesia tropical peat may differ\n");
    out.push_str("  - Severity factor is approximate — actual burn depth varies\n");
    out.push_str("  - Does not account for subsurface peat combustion beyond measured depth\n");
    out.push_str("  - Page et al. 2002 measured higher EFs for Kalimantan peat fires\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}
