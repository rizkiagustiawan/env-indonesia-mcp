/// Carlson Trophic State Index (TSI)
/// Ref: Carlson (1977), valid untuk DANAU saja (bukan sungai)

pub fn calculate(secchi_depth_m: Option<f64>, chlorophyll_ugl: Option<f64>, total_phosphorus_ugl: Option<f64>) -> String {
    let mut out = String::from("=== Carlson Trophic State Index (TSI) ===\n");
    out.push_str("Ref: Carlson (1977)\n");
    out.push_str("⚠️ Hanya valid untuk DANAU. Tidak valid untuk sungai/estuari.\n\n");

    let mut tsi_values: Vec<(String, f64)> = Vec::new();

    if let Some(sd) = secchi_depth_m {
        if sd <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
        let tsi = 60.0 - 14.41 * sd.ln();
        tsi_values.push(("TSI(SD)".into(), tsi));
        out.push_str(&format!("Secchi Depth = {:.2} m → TSI(SD) = {:.1}\n", sd, tsi));
    }
    if let Some(chl) = chlorophyll_ugl {
        if chl <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
        let tsi = 9.81 * chl.ln() + 30.6;
        tsi_values.push(("TSI(Chl)".into(), tsi));
        out.push_str(&format!("Chlorophyll-a = {:.2} µg/L → TSI(Chl) = {:.1}\n", chl, tsi));
    }
    if let Some(tp) = total_phosphorus_ugl {
        if tp <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
        let tsi = 14.42 * tp.ln() + 4.15;
        tsi_values.push(("TSI(TP)".into(), tsi));
        out.push_str(&format!("Total Phosphorus = {:.2} µg/L → TSI(TP) = {:.1}\n", tp, tsi));
    }

    if tsi_values.is_empty() { return "ERROR: Masukkan minimal 1 parameter (secchi/chlorophyll/phosphorus).".into(); }

    let avg_tsi: f64 = tsi_values.iter().map(|(_, v)| v).sum::<f64>() / tsi_values.len() as f64;
    let cat = if avg_tsi < 40.0 { "Oligotrofik (bersih)" } else if avg_tsi < 50.0 { "Mesotrofik (sedang)" } else if avg_tsi < 70.0 { "Eutrofik (subur berlebih)" } else { "Hipereutrofik (sangat tercemar nutrisi)" };

    out.push_str(&format!("\nTSI rata-rata = {:.1}\nKategori: {}\n", avg_tsi, cat));
    if avg_tsi >= 50.0 { out.push_str("\n⚠️ Eutrofikasi terdeteksi. Risiko: algal bloom, penurunan DO, kematian ikan.\n"); }
    out
}
