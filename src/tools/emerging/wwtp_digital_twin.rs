/// WWTP Digital Twin — Simplified ASM1 + Predictive Control
/// Ref: Wang et al. 2024 (Engineering); ASM1 (Henze et al. 2000)
/// Monod kinetics + mass balance + aeration optimization
pub fn assess(influent_bod_mg_l: f64, influent_cod_mg_l: f64, flow_m3_day: f64, mlss_mg_l: f64, do_mg_l: f64, temp_c: f64, volume_m3: f64, target_bod_mg_l: f64) -> String {
    let mut out = String::from("=== WWTP Digital Twin (Simplified ASM1) ===\n");
    out.push_str("Ref: Wang et al. 2024 (Engineering); ASM1 (Henze 2000)\n\n");
    let k_h = 3.0 * 1.072_f64.powf(temp_c - 20.0); // hydrolysis rate (1/day)
    let mu_h = 4.0 * 1.072_f64.powf(temp_c - 20.0); // heterotroph max growth (1/day)
    let k_s = 20.0; // half-saturation for substrate (mg COD/L)
    let k_oh = 0.2; // half-saturation for DO (mg O2/L)
    let b_h = 0.3 * 1.072_f64.powf(temp_c - 20.0); // decay rate (1/day)
    let y_h = 0.67; // yield coefficient
    let q = flow_m3_day / 24.0 / 60.0; // m3/min
    let bod_load = influent_bod_mg_l * flow_m3_day / 1000.0; // kg/day
    let fm = bod_load / (volume_m3 * mlss_mg_l / 1000.0); // F/M
    let monod_substrate = influent_bod_mg_l / (k_s + influent_bod_mg_l).max(1e-6);
    let monod_do = do_mg_l / (k_oh + do_mg_l).max(1e-6);
    let growth_rate = mu_h * monod_substrate * monod_do;
    let substrate_removal_rate = growth_rate / y_h * mlss_mg_l;
    let predicted_bod_eff = influent_bod_mg_l * (-growth_rate * volume_m3 * 24.0 / flow_m3_day.max(1e-6)).exp();
    let o2_demand = bod_load * (1.0 - y_h) + mlss_mg_l * volume_m3 / 1000.0 * b_h * 1.42 * y_h;
    let air_required_m3_hr = o2_demand * 1000.0 / (0.232 * 1.225 * 0.3 * 24.0);
    let optimal_do = if predicted_bod_eff > target_bod_mg_l { do_mg_l * 1.2 } else { do_mg_l };
    out.push_str(&format!("Influent: BOD={:.0} mg/L, COD={:.0}, Flow={:.0} m3/day\n", influent_bod_mg_l, influent_cod_mg_l, flow_m3_day));
    out.push_str(&format!("MLSS: {:.0} mg/L, DO: {:.1} mg/L, Temp: {:.0}°C\n", mlss_mg_l, do_mg_l, temp_c));
    out.push_str(&format!("Volume: {:.0} m3, Target effluent BOD: {:.0}\n\n", volume_m3, target_bod_mg_l));
    out.push_str("-- ASM1 Kinetics (Simplified) --\n\n");
    out.push_str(&format!("  μ_H ({:.0}°C): {:.3}/day\n", temp_c, mu_h));
    out.push_str(&format!("  K_S: {:.0} mg/L, K_OH: {:.1} mg/L\n", k_s, k_oh));
    out.push_str(&format!("  b_H: {:.3}/day, Y_H: {:.2}\n", b_h, y_h));
    out.push_str(&format!("  Monod (substrate): {:.3}\n", monod_substrate));
    out.push_str(&format!("  Monod (DO): {:.3}\n", monod_do));
    out.push_str(&format!("  >> Growth rate: {:.3}/day\n\n", growth_rate));
    out.push_str("-- Digital Twin Predictions --\n\n");
    out.push_str(&format!("  F/M: {:.3} kg BOD/kg MLSS/day\n", fm));
    out.push_str(&format!("  Predicted effluent BOD: {:.1} mg/L\n", predicted_bod_eff));
    out.push_str(&format!("  O2 demand: {:.1} kg/day\n", o2_demand));
    out.push_str(&format!("  Air required: {:.0} m3/hr\n", air_required_m3_hr));
    out.push_str(&format!("  >> Recommended DO setpoint: {:.1} mg/L\n\n", optimal_do));
    out.push_str("-- STATUS KEPATUHAN --\n");
    out.push_str(&format!("  Permen LH 11/2025: BOD ≤30 mg/L → {}\n", if predicted_bod_eff <= 30.0 {"✅"} else {"❌"}));
    out.push_str("\n-- Optimization --\n");
    if predicted_bod_eff > target_bod_mg_l {
        out.push_str("  1. Increase DO setpoint\n");
        out.push_str("  2. Increase MLSS (raise SRT)\n");
        out.push_str("  3. Reduce influent load (equalization)\n");
    } else {
        out.push_str("  Optimize: reduce aeration (energy savings)\n");
    }
    out.push_str("\n  Ref: Wang et al. 2024; ASM1 (Henze 2000)\n");
    out
}
