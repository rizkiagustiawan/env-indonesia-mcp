/// Heat Index Calculator
/// Ref: Rothfusz (NWS), WHO/NOAA danger categories

pub fn calculate(temp_c: f64, rh_pct: f64) -> String {
    if rh_pct < 0.0 || rh_pct > 100.0 {
        return format!("ERROR: RH {}% harus 0-100.", rh_pct);
    }

    // Rothfusz regression (°C version)
    let hi = if temp_c < 27.0 {
        temp_c
    } else {
        let t = temp_c;
        let r = rh_pct;
        -8.78469475556
            + 1.61139411 * t
            + 2.33854883889 * r
            + (-0.14611605) * t * r
            + (-0.012308094) * t * t
            + (-0.0164248277778) * r * r
            + 2.211732e-3 * t * t * r
            + 7.2546e-4 * t * r * r
            + (-3.582e-6) * t * t * r * r
    };

    let cat = if hi < 27.0 {
        "Aman"
    } else if hi < 32.0 {
        "Hati-hati (Caution)"
    } else if hi < 41.0 {
        "Sangat Hati-hati (Extreme Caution)"
    } else if hi < 54.0 {
        "BAHAYA (Danger)"
    } else {
        "BAHAYA EKSTREM (Extreme Danger)"
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("  Heat Index Calculator\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: Rothfusz/NWS regression, NOAA categories\n");
    if temp_c < 27.0 {
        out.push_str("⚠️ Formula valid untuk T ≥ 27°C & RH ≥ 40%.\n");
    }
    out.push_str("⚠️ Akurasi: ±1.3°F (NWS). Overestimate ≤1.5°C di extreme humidity (Romps & Lu, PNAS 2022).\n");
    out.push_str(&format!("\nINPUT:\n  Suhu = {:.1}°C\n  RH = {:.0}%\n\nHASIL:\n  Heat Index = {:.1}°C\n  Kategori: {}\n\n", temp_c, rh_pct, hi, cat));
    if hi >= 41.0 {
        out.push_str("⛔ Heat stroke risk sangat tinggi. Aktivitas outdoor harus dihentikan.\n");
    }
    if hi >= 35.0 {
        out.push_str("⚠️ Wet-bulb 35°C = ambang batas fatal manusia (Sherwood & Huber, 2010).\n");
    }
    out
}
