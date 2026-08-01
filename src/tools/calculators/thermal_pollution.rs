/// Thermal Pollution Calculator (Mixing Zone)
/// Ref: PLTU cooling water discharge regulation

pub fn calculate(
    q_river_m3s: f64,
    t_river_c: f64,
    q_discharge_m3s: f64,
    t_discharge_c: f64,
) -> String {
    if q_river_m3s <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if q_discharge_m3s <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    let t_mix = (q_river_m3s * t_river_c + q_discharge_m3s * t_discharge_c)
        / (q_river_m3s + q_discharge_m3s);
    let delta_t = t_mix - t_river_c;

    let mut out = format!("=== Thermal Pollution (Mixing Zone) ===\n\nDebit sungai: {:.2} m³/s @ {:.1}°C\nDebit buangan: {:.2} m³/s @ {:.1}°C\n\nSuhu campuran: {:.1}°C\nΔT = {:.1}°C\n\n", q_river_m3s, t_river_c, q_discharge_m3s, t_discharge_c, t_mix, delta_t);
    out.push_str(&format!(
        "Baku Mutu PP 22/2021: deviasi maks 3°C dari alami.\n{}\n",
        if delta_t.abs() > 3.0 {
            "❌ MELEBIHI baku mutu!"
        } else {
            "✅ Memenuhi baku mutu."
        }
    ));
    out
}
