/// DO Saturation Calculator (APHA Standard Methods)
/// Kelarutan oksigen terlarut sebagai fungsi suhu air

pub fn calculate(temp_c: f64) -> String {
    if temp_c < 0.0 || temp_c > 50.0 {
        return format!("ERROR: Suhu air {} °C di luar rentang (0-50).", temp_c);
    }

    let t_k = temp_c + 273.15;
    let ln_do = -139.3441 + (1.575701e5 / t_k) - (6.642308e7 / (t_k * t_k))
        + (1.243800e10 / (t_k * t_k * t_k))
        - (8.621949e11 / (t_k * t_k * t_k * t_k));
    let do_sat = ln_do.exp();

    let mut out = String::from("=== DO Saturation Calculator ===\n");
    out.push_str("Ref: APHA Standard Methods, Benson & Krause (1984)\n\n");
    out.push_str(&format!("Suhu air: {:.1} °C\n", temp_c));
    out.push_str(&format!("DO saturasi: {:.2} mg/L\n\n", do_sat));
    out.push_str("Tabel referensi:\n");
    for t in [0.0_f64, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0] {
        let tk = t + 273.15;
        let ln_d = -139.3441 + (1.575701e5 / tk) - (6.642308e7 / (tk * tk))
            + (1.243800e10 / (tk * tk * tk))
            - (8.621949e11 / (tk * tk * tk * tk));
        out.push_str(&format!("  {:.0}°C → {:.2} mg/L\n", t, ln_d.exp()));
    }
    out.push_str("\nUntuk Indonesia tropis (suhu air 26-32°C): DO sat ≈ 7.0-8.1 mg/L\n");
    out
}
