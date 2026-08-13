/// Vapor Intrusion — Johnson & Ettinger Model
/// Ref: Johnson & Ettinger 1991; EPA 2017; Tillman 2007
pub fn assess(
    source_conc_ug_m3: f64,
    soil_porosity_total: f64,
    soil_porosity_water: f64,
    soil_porosity_air: f64,
    stratum_thickness_m: f64,
    bldg_footprint_m2: f64,
    bldg_height_m: f64,
    air_exchange_rate_hr: f64,
    crack_area_m2: f64,
    crack_depth_m: f64,
) -> String {
    let mut out = String::from("=== Vapor Intrusion (Johnson & Ettinger) ===\n");
    out.push_str("Ref: Johnson & Ettinger 1991; EPA 2017\n\n");

    if source_conc_ug_m3 <= 0.0 || stratum_thickness_m <= 0.0 {
        return "ERROR [E102]: source conc and thickness must be > 0.".into();
    }

    let n = soil_porosity_total;
    let theta_a = soil_porosity_air;
    let theta_w = soil_porosity_water;
    let L_T = stratum_thickness_m; // source to building
    let A_b = bldg_footprint_m2;
    let H_b = bldg_height_m;
    let ACH = air_exchange_rate_hr;
    let A_crack = crack_area_m2;
    let L_crack = crack_depth_m;

    // Effective diffusion coefficient (typical soil, vapor phase)
    // D_eff = D_air * theta_a^(10/3) / n^2 (Millington-Quirk)
    let d_air = 0.1; // m2/day typical for VOCs in air
    let d_eff = d_air * theta_a.powf(10.0/3.0) / n.powi(2).max(1e-15);

    // Q_soil = building ventilation rate (m3/day) = ACH[1/hr] × V[m³] × 24[hr/day]
    let q_soil = A_b * H_b * ACH * 24.0;

    // Attenuation coefficient (J&E 1991):
    // alpha = (D_eff * A_b * L_crack) / (Q_soil * L_T + D_eff * A_b * L_crack * (L_T/L_crack))
    // Simplified: alpha = 1 / (1 + (Q_soil * L_T) / (D_eff * A_b * L_crack))
    let numerator = d_eff * A_b * L_crack;
    let denominator = q_soil * L_T + numerator * (L_T / L_crack.max(0.01));
    let alpha = numerator / denominator.max(1e-15);

    // Indoor air concentration
    let c_indoor = source_conc_ug_m3 * alpha;

    out.push_str(&format!("Source concentration: {:.0} ug/m3\n", source_conc_ug_m3));
    out.push_str(&format!("Soil: n={:.2}, theta_a={:.2}, theta_w={:.2}\n", n, theta_a, theta_w));
    out.push_str(&format!("Stratum thickness: {:.1} m\n", L_T));
    out.push_str(&format!("Building: {:.0} m2 x {:.1}m, ACH={:.1}\n\n", A_b, H_b, ACH));

    out.push_str("-- J&E Parameters --\n\n");
    out.push_str(&format!("  D_eff (Millington-Quirk): {:.2e} m2/day\n", d_eff));
    out.push_str(&format!("  Q_soil (ventilation): {:.2} m3/day\n", q_soil));
    out.push_str(&format!("  Crack area: {:.4} m2, depth: {:.3} m\n", A_crack, L_crack));
    out.push_str(&format!("  >> Attenuation factor alpha: {:.6}\n\n", alpha));

    out.push_str("-- Indoor Air Concentration --\n\n");
    out.push_str(&format!("  >> C_indoor = C_source * alpha = {:.0} * {:.6} = {:.4} ug/m3\n\n", source_conc_ug_m3, alpha, c_indoor));

    // Risk assessment
    // Typical screening levels: benzene = 0.31 ug/m3, TCE = 2.0 ug/m3
    let screening_level = 1.0; // generic
    if c_indoor > screening_level {
        out.push_str(&format!("  [WARN] C_indoor ({:.4} ug/m3) exceeds screening level ({:.1} ug/m3)\n", c_indoor, screening_level));
        out.push_str("  Mitigation: sub-slab depressurization, vapor barrier, increased ventilation\n");
    } else {
        out.push_str(&format!("  [OK] Below screening level\n"));
    }

    out.push_str("\n  Ref: Johnson & Ettinger 1991; EPA 2017 (J&E model)\n");
    out
}

#[cfg(test)]
mod tests {
    use super::assess;

    #[test]
    fn ventilation_rate_units() {
        // V=100×3=300 m3, ACH=0.5/hr -> Q = 300*0.5*24 = 3600 m3/day (not /24)
        let result = assess(1000.0, 0.4, 0.1, 0.3, 2.0, 100.0, 3.0, 0.5, 0.01, 0.05);
        assert!(result.contains("Q_soil (ventilation): 3600.00"), "ACH conversion wrong:\n{result}");
    }
}
