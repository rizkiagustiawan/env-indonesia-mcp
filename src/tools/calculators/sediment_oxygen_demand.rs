/// Sediment Oxygen Demand (SOD)
/// Ref: DiToro 2001; Chapra 2008; EPA
/// SOD(T) = SOD20 * theta^(T-20)
pub fn assess(sod20_g_m2_day: f64, temp_c: f64, area_m2: f64, river_flow_m3_s: f64) -> String {
    let mut out = String::from("=== Sediment Oxygen Demand (SOD) ===\n");
    out.push_str("Ref: DiToro 2001; Chapra 2008\n\n");

    let theta: f64 = 1.065; // temperature correction (typical 1.06-1.07)
    let sod_t = sod20_g_m2_day * theta.powf(temp_c - 20.0);
    let total_sod_g_day = sod_t * area_m2;
    let total_sod_kg_day = total_sod_g_day / 1000.0;

    // Impact on water column DO
    // dDO/dt = -SOD * A / (H * Q)  =>  DO_depletion = SOD * A / Q (mg/L)
    let do_depletion_mg_l = if river_flow_m3_s > 0.0 {
        (total_sod_g_day / 1000.0) / (river_flow_m3_s * 86400.0 / 1000.0) // g/day -> kg/day; m3/s -> m3/day -> L
    } else { 0.0 };

    // DO_depletion = SOD(g/day) / Q(m³/day) = g/m³ = mg/L (1 g/m³ ≡ 1 mg/L)
    let do_dep = if river_flow_m3_s > 0.0 {
        total_sod_g_day / (river_flow_m3_s * 86400.0)
    } else { 0.0 };

    out.push_str(&format!("SOD20: {:.2} g/m2/day\n", sod20_g_m2_day));
    out.push_str(&format!("Temperature: {:.1} C (theta=1.065)\n", temp_c));
    out.push_str(&format!("Sediment area: {:.0} m2\n\n", area_m2));

    out.push_str("-- Results --\n\n");
    out.push_str(&format!("  SOD at {:.0}C: {:.3} g/m2/day\n", temp_c, sod_t));
    out.push_str(&format!("  Total SOD: {:.1} g/day ({:.2} kg/day)\n", total_sod_g_day, total_sod_kg_day));
    if river_flow_m3_s > 0.0 {
        out.push_str(&format!("  >> DO depletion in water column: {:.3} mg/L (flow={:.1} m3/s)\n\n", do_dep, river_flow_m3_s));
    }

    out.push_str("-- Classification --\n");
    if sod_t < 1.0 { out.push_str("  Low (clean sand/gravel): <1 g/m2/day\n"); }
    else if sod_t < 3.0 { out.push_str("  Moderate (silt): 1-3 g/m2/day\n"); }
    else if sod_t < 5.0 { out.push_str("  High (organic sediment): 3-5 g/m2/day\n"); }
    else { out.push_str("  Very high (polluted/anoxic): >5 g/m2/day\n"); }

    out.push_str("\n  Ref: DiToro 2001; Chapra 2008\n");
    out
}

#[cfg(test)]
mod tests {
    use super::assess;

    #[test]
    fn do_depletion_units() {
        // SOD=2 g/m2/d, area=1000 m2 -> 2000 g/d; Q=10 m3/s = 864000 m3/d
        // do_dep = 2000/864000 = 0.0023 mg/L (not 0.0000023)
        let result = assess(2.0, 20.0, 1000.0, 10.0);
        assert!(result.contains("DO depletion in water column: 0.002"), "DO depletion 1000x off:\n{result}");
    }
}
