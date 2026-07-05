/// Metode STORET
/// Ref: KepmenLH No. 115 Tahun 2003

pub fn calculate(data_json: &str) -> String {
    // data_json format: [{"name":"BOD", "type":"kimia", "samples": [{"value":4.0, "limit":2.0}, ...]}, ...]
    let data: Result<Vec<serde_json::Value>, _> = serde_json::from_str(data_json);
    if data.is_err() { return "ERROR [E103]: Format data JSON tidak valid.".into(); }

    let params = data.unwrap();
    if params.is_empty() { return "ERROR: Masukkan minimal 1 parameter.".into(); }

    let mut total_score = 0;
    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Metode STORET\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nRef: KepmenLH 115/2003\n\n");

    for p in params {
        let name = p["name"].as_str().unwrap_or("Unknown");
        let p_type = p["type"].as_str().unwrap_or("kimia").to_lowercase(); // fisika, kimia, biologi
        let samples = p["samples"].as_array();

        if samples.is_none() || samples.unwrap().is_empty() { continue; }
        let samples = samples.unwrap();
        let num_samples = samples.len();

        let mut values = Vec::new();
        let mut limit = 0.0;
        for s in samples {
            values.push(s["value"].as_f64().unwrap_or(0.0));
            limit = s["limit"].as_f64().unwrap_or(limit);
        }

        let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mean_val = values.iter().sum::<f64>() / num_samples as f64;

        let mut param_score = 0;

        // Penentuan bobot berdasarkan tipe dan jumlah sampel
        let (w_max_min, w_mean) = if num_samples < 10 {
            match p_type.as_str() {
                "fisika" => (-1, -3),
                "biologi" => (-3, -9),
                _ => (-2, -6), // default kimia
            }
        } else {
            match p_type.as_str() {
                "fisika" => (-2, -6),
                "biologi" => (-6, -18),
                _ => (-4, -12), // default kimia
            }
        };

        if max_val > limit { param_score += w_max_min; }
        if min_val > limit { param_score += w_max_min; } // Asumsi baku mutu maks. Untuk parameter minimum (spt DO), logika di-handle pre-processing.
        if mean_val > limit { param_score += w_mean; }

        total_score += param_score;

        out.push_str(&format!("{:<10} ({} smpl) | Max:{:.1} Min:{:.1} Avg:{:.1} BM:{:.1} -> Skor: {}\n", 
            &name[..name.len().min(10)], num_samples, max_val, min_val, mean_val, limit, param_score));
    }

    let status = if total_score == 0 { "Kelas A: Memenuhi Baku Mutu (Baik Sekali)" }
    else if total_score >= -10 { "Kelas B: Cemar Ringan" }
    else if total_score >= -30 { "Kelas C: Cemar Sedang" }
    else { "Kelas D: Cemar Berat" };

    out.push_str(&format!("\nTotal Skor STORET = {}\n", total_score));
    out.push_str(&format!("Status Mutu Air: {}\n", status));

    out
}
