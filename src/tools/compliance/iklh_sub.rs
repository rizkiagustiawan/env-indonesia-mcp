/// IKLH Sub-Indices Calculator
/// Ref: PermenLHK P.14/2020

/// IKA dari nilai-nilai Indeks Pencemaran
pub fn calculate_ika(ip_values: &[f64]) -> String {
    if ip_values.is_empty() {
        return "ERROR: Masukkan minimal 1 nilai Indeks Pencemaran.".into();
    }

    for (i, v) in ip_values.iter().enumerate() {
        if *v < 0.0 {
            return format!("ERROR [E102]: Parameter tidak boleh negatif. indeks={}, value={}", i + 1, v);
        }
    }

    let mean_ip: f64 = ip_values.iter().sum::<f64>() / ip_values.len() as f64;
    // IKA = 100 - (mean_IP * 10), capped to 0-100
    // Scaling: IP 0 = IKA 100 (sempurna), IP 10 = IKA 0 (cemar berat)
    let ika = (100.0 - (mean_ip * 10.0)).clamp(0.0, 100.0);

    let kategori = if ika >= 80.0 { "Sangat Baik" }
        else if ika >= 60.0 { "Baik" }
        else if ika >= 40.0 { "Cukup" }
        else if ika >= 20.0 { "Kurang" }
        else { "Sangat Kurang" };

    let mut out = String::from("=== IKA (Indeks Kualitas Air) ===\n");
    out.push_str("Ref: PermenLHK P.14/2020\n\n");
    out.push_str(&format!("Jumlah Titik Pantau: {}\n", ip_values.len()));
    out.push_str(&format!("Nilai IP: {:?}\n", ip_values));
    out.push_str(&format!("Rata-rata IP: {:.2}\n\n", mean_ip));
    out.push_str(&format!("IKA = 100 - ({:.2} × 10) = {:.2}\n", mean_ip, ika));
    out.push_str(&format!("Kategori: {} ({:.2})\n", kategori, ika));
    out
}

/// IKU dari nilai-nilai ISPU
pub fn calculate_iku(ispu_values: &[f64]) -> String {
    if ispu_values.is_empty() {
        return "ERROR: Masukkan minimal 1 nilai ISPU.".into();
    }

    for (i, v) in ispu_values.iter().enumerate() {
        if *v < 0.0 {
            return format!("ERROR [E102]: Parameter tidak boleh negatif. indeks={}, value={}", i + 1, v);
        }
    }

    let mean_ispu: f64 = ispu_values.iter().sum::<f64>() / ispu_values.len() as f64;
    // IKU = 100 - (mean_ISPU * 0.25), scaled so ISPU 0 = IKU 100, ISPU 400 = IKU 0
    let iku = (100.0 - (mean_ispu * 0.25)).clamp(0.0, 100.0);

    let kategori = if iku >= 80.0 { "Sangat Baik" }
        else if iku >= 60.0 { "Baik" }
        else if iku >= 40.0 { "Cukup" }
        else if iku >= 20.0 { "Kurang" }
        else { "Sangat Kurang" };

    let mut out = String::from("=== IKU (Indeks Kualitas Udara) ===\n");
    out.push_str("Ref: PermenLHK P.14/2020\n\n");
    out.push_str(&format!("Jumlah Stasiun: {}\n", ispu_values.len()));
    out.push_str(&format!("Nilai ISPU: {:?}\n", ispu_values));
    out.push_str(&format!("Rata-rata ISPU: {:.2}\n\n", mean_ispu));
    out.push_str(&format!("IKU = 100 - ({:.2} × 0.25) = {:.2}\n", mean_ispu, iku));
    out.push_str(&format!("Kategori: {} ({:.2})\n", kategori, iku));
    out
}

/// IKTL dari persentase tutupan lahan
pub fn calculate_iktl(forest_cover_pct: f64, target_pct: f64) -> String {
    if forest_cover_pct < 0.0 || forest_cover_pct > 100.0 {
        return format!("ERROR: Persentase tutupan lahan ({:.1}%) harus 0-100.", forest_cover_pct);
    }
    if target_pct <= 0.0 || target_pct > 100.0 {
        return format!("ERROR [E102]: Parameter harus > 0 dan <= 100. {}", target_pct);
    }

    // IKTL = (forest_cover_pct / target_pct) * 100, capped at 100
    let iktl = ((forest_cover_pct / target_pct) * 100.0).min(100.0);

    let kategori = if iktl >= 80.0 { "Sangat Baik" }
        else if iktl >= 60.0 { "Baik" }
        else if iktl >= 40.0 { "Cukup" }
        else if iktl >= 20.0 { "Kurang" }
        else { "Sangat Kurang" };

    let mut out = String::from("=== IKTL (Indeks Kualitas Tutupan Lahan) ===\n");
    out.push_str("Ref: PermenLHK P.14/2020\n\n");
    out.push_str(&format!("Tutupan Lahan Aktual : {:.1}%\n", forest_cover_pct));
    out.push_str(&format!("Target Tutupan Lahan : {:.1}%\n\n", target_pct));
    out.push_str(&format!("IKTL = ({:.1} / {:.1}) × 100 = {:.2}\n", forest_cover_pct, target_pct, iktl));
    out.push_str(&format!("Kategori: {} ({:.2})\n", kategori, iktl));
    out
}

/// IKAL dari parameter kualitas air laut
pub fn calculate_ikal(sea_quality_params: &str) -> String {
    // Parse JSON: [{"name":"TSS","ci":25.0,"lij":80.0}, ...]
    let data: Result<Vec<serde_json::Value>, _> = serde_json::from_str(sea_quality_params);
    if data.is_err() {
        return "ERROR: Format JSON tidak valid. Contoh: [{\"name\":\"TSS\",\"ci\":25.0,\"lij\":80.0}]".into();
    }

    let params = data.unwrap();
    if params.is_empty() {
        return "ERROR: Masukkan minimal 1 parameter kualitas air laut.".into();
    }

    let mut ratios = Vec::new();
    let mut out = String::from("=== IKAL (Indeks Kualitas Air Laut) ===\n");
    out.push_str("Ref: PermenLHK P.14/2020\n\n");
    out.push_str(format!("{:<10} | {:<8} | {:<8} | {}\n", "Parameter", "Ci", "Lij", "Rasio").as_str());
    out.push_str("─────────────────────────────────────\n");

    for p in &params {
        let name = p["name"].as_str().unwrap_or("?");
        let ci = p["ci"].as_f64().unwrap_or(0.0);
        let lij = p["lij"].as_f64().unwrap_or(0.0);

        if lij <= 0.0 {
            return format!("ERROR [E102]: Parameter harus > 0. {}", name);
        }

        let ratio = ci / lij;
        ratios.push(ratio);
        out.push_str(&format!("{:<10} | {:<8.2} | {:<8.2} | {:.3}\n", &name[..name.len().min(10)], ci, lij, ratio));
    }

    let mean_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;
    // IKAL = 100 - (mean_ratio * 10), similar to IKA approach
    let ikal = (100.0 - (mean_ratio * 10.0)).clamp(0.0, 100.0);

    let kategori = if ikal >= 80.0 { "Sangat Baik" }
        else if ikal >= 60.0 { "Baik" }
        else if ikal >= 40.0 { "Cukup" }
        else if ikal >= 20.0 { "Kurang" }
        else { "Sangat Kurang" };

    out.push_str(&format!("\nRata-rata Rasio Ci/Lij: {:.3}\n", mean_ratio));
    out.push_str(&format!("IKAL = 100 - ({:.3} × 10) = {:.2}\n", mean_ratio, ikal));
    out.push_str(&format!("Kategori: {} ({:.2})\n", kategori, ikal));
    out
}
