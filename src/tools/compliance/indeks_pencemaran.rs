/// Indeks Pencemaran (IP) Calculator
/// Ref: KepmenLH No. 115 Tahun 2003

pub fn calculate(data_json: &str, temp_c: f64) -> String {
    // data_json format: [{"name": "BOD", "ci": 4.0, "lij": 2.0, "is_do": false}, ...]
    let data: Result<Vec<serde_json::Value>, _> = serde_json::from_str(data_json);
    
    if data.is_err() {
        return "ERROR [E103]: Format data JSON tidak valid.".into();
    }
    
    let params = data.unwrap();
    if params.is_empty() { return "ERROR: Masukkan minimal 1 parameter.".into(); }

    // Hitung DO Saturation
    let tk = temp_c + 273.15;
    let ln_do = -139.3441 + (1.575701e5/tk) - (6.642308e7/(tk*tk)) + (1.243800e10/(tk*tk*tk)) - (8.621949e11/(tk*tk*tk*tk));
    let do_sat = ln_do.exp();

    let mut ratios = Vec::new();
    let mut out = format!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Indeks Pencemaran (IP)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nRef: KepmenLH 115/2003\nSuhu Air: {:.1}°C (DO Saturation: {:.2} mg/L)\n\n", temp_c, do_sat);
    
    out.push_str(format!("{:<10} | {:<7} | {:<7} | {:<7}\n", "Parameter", "Ci", "Lij", "Rasio").as_str());
    out.push_str("----------------------------------------\n");

    for p in params {
        let name = p["name"].as_str().unwrap_or("Unknown");
        let ci = p["ci"].as_f64().unwrap_or(0.0);
        let lij = p["lij"].as_f64().unwrap_or(0.0);
        let is_do = p["is_do"].as_bool().unwrap_or(false);

        if lij <= 0.0 { return format!("ERROR [E102]: Parameter harus > 0. {}", name); }

        let mut ratio = if is_do {
            if do_sat <= lij { 1.0 } else { (do_sat - ci) / (do_sat - lij) }
        } else {
            ci / lij
        };

        // Normalisasi untuk ratio > 1.0
        let original_ratio = ratio;
        if ratio > 1.0 {
            ratio = 1.0 + 5.0 * ratio.log10();
        }

        ratios.push(ratio);
        out.push_str(&format!("{:<10} | {:<7.2} | {:<7.2} | {:.3}{}\n", 
            &name[..name.len().min(10)], ci, lij, ratio, 
            if original_ratio > 1.0 { "*" } else { "" }));
    }

    let sum_ratios: f64 = ratios.iter().sum();
    let mean_ratio = sum_ratios / ratios.len() as f64;
    let max_ratio = ratios.iter().copied().fold(f64::NAN, f64::max);

    let ip = ((max_ratio.powi(2) + mean_ratio.powi(2)) / 2.0).sqrt();

    let status = if ip <= 1.0 { "Memenuhi Baku Mutu (Baik)" }
    else if ip <= 5.0 { "Cemar Ringan" }
    else if ip <= 10.0 { "Cemar Sedang" }
    else { "Cemar Berat" };

    out.push_str("\n* Dinormalisasi: 1 + 5×log(Ci/Lij)\n\n");
    out.push_str(&format!("Maksimum Rasio (Ci/Lij)_M = {:.3}\n", max_ratio));
    out.push_str(&format!("Rata-rata Rasio (Ci/Lij)_R = {:.3}\n\n", mean_ratio));
    out.push_str(&format!("Skor IP = {:.2}\n", ip));
    out.push_str(&format!("Status Mutu Air: {}\n", status));

    out
}
