/// Perhitungan Timbulan Lindi (Leachate) — Metode Neraca Air
/// Ref: PermenPU 3/2013, Tchobanoglous et al. (1993) Integrated Solid Waste Management

pub fn calculate(area_m2: f64, monthly_rainfall_mm: &[f64], monthly_et_mm: &[f64], soil_storage_mm: f64, runoff_coeff: f64) -> String {
    if area_m2 <= 0.0 { return "ERROR: Luas area harus > 0.".into(); }
    if monthly_rainfall_mm.len() != 12 {
        return format!("ERROR: Data curah hujan harus 12 bulan, diberikan {} bulan.", monthly_rainfall_mm.len());
    }
    if monthly_et_mm.len() != 12 {
        return format!("ERROR: Data evapotranspirasi harus 12 bulan, diberikan {} bulan.", monthly_et_mm.len());
    }
    if runoff_coeff < 0.0 || runoff_coeff > 1.0 {
        return "ERROR: Koefisien runoff harus antara 0 dan 1.".into();
    }
    if soil_storage_mm < 0.0 { return "ERROR: Kapasitas simpan tanah tidak boleh negatif.".into(); }

    let month_names = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];

    let mut monthly_leachate_mm = Vec::with_capacity(12);
    let mut storage = soil_storage_mm;
    let mut total_leachate_mm = 0.0;
    let mut total_rainfall = 0.0;
    let mut total_et = 0.0;
    let mut total_runoff = 0.0;
    let mut peak_month_idx = 0_usize;
    let mut peak_leachate = 0.0_f64;

    let mut rows = Vec::new();

    for i in 0..12 {
        let p = monthly_rainfall_mm[i];
        let et = monthly_et_mm[i];
        let runoff = p * runoff_coeff;
        let infiltration = p - runoff - et;

        total_rainfall += p;
        total_et += et;
        total_runoff += runoff;

        let leachate = if infiltration > 0.0 {
            let excess = infiltration - (soil_storage_mm - storage).max(0.0);
            storage = (storage + infiltration).min(soil_storage_mm);
            excess.max(0.0)
        } else {
            storage = (storage + infiltration).max(0.0);
            0.0
        };

        monthly_leachate_mm.push(leachate);
        total_leachate_mm += leachate;

        if leachate > peak_leachate {
            peak_leachate = leachate;
            peak_month_idx = i;
        }

        let leachate_m3 = leachate / 1000.0 * area_m2;
        rows.push(format!(
            "│ {:3} │ {:>8.1} │ {:>8.1} │ {:>8.1} │ {:>8.1} │ {:>10.1} │",
            month_names[i], p, et, runoff, leachate, leachate_m3
        ));
    }

    let total_leachate_m3 = total_leachate_mm / 1000.0 * area_m2;
    let peak_leachate_m3 = peak_leachate / 1000.0 * area_m2;
    let avg_daily_m3 = total_leachate_m3 / 365.0;
    let design_capacity_m3 = peak_leachate_m3 / 30.0 * 1.5; // peak month daily × 1.5 safety factor

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════════════════════════\n");
    result.push_str("PERHITUNGAN TIMBULAN LINDI — METODE NERACA AIR\n");
    result.push_str("Ref: PermenPU 3/2013, Tchobanoglous et al. (1993)\n");
    result.push_str("══════════════════════════════════════════════════════════════════\n\n");

    result.push_str(&format!("Luas TPA             : {:.2} m² ({:.2} Ha)\n", area_m2, area_m2 / 10000.0));
    result.push_str(&format!("Koefisien runoff     : {:.2}\n", runoff_coeff));
    result.push_str(&format!("Kapasitas simpan     : {:.1} mm\n\n", soil_storage_mm));

    result.push_str("L = P - ET - R - ΔS (per bulan)\n\n");

    result.push_str("┌─────┬──────────┬──────────┬──────────┬──────────┬────────────┐\n");
    result.push_str("│ Bln │  CH (mm) │  ET (mm) │  RO (mm) │  L  (mm) │   L  (m³)  │\n");
    result.push_str("├─────┼──────────┼──────────┼──────────┼──────────┼────────────┤\n");
    for row in &rows {
        result.push_str(row);
        result.push('\n');
    }
    result.push_str("└─────┴──────────┴──────────┴──────────┴──────────┴────────────┘\n\n");

    result.push_str("REKAPITULASI TAHUNAN:\n");
    result.push_str(&format!("• Total curah hujan      : {:.1} mm\n", total_rainfall));
    result.push_str(&format!("• Total evapotranspirasi : {:.1} mm\n", total_et));
    result.push_str(&format!("• Total runoff           : {:.1} mm\n", total_runoff));
    result.push_str(&format!("• Total lindi            : {:.1} mm = {:.1} m³/tahun\n", total_leachate_mm, total_leachate_m3));
    result.push_str(&format!("• Rata-rata harian       : {:.2} m³/hari\n", avg_daily_m3));
    result.push_str(&format!("• Bulan puncak           : {} ({:.1} mm = {:.1} m³)\n\n", month_names[peak_month_idx], peak_leachate, peak_leachate_m3));

    result.push_str("KAPASITAS PENGOLAHAN REKOMENDASI:\n");
    result.push_str(&format!("• Kapasitas desain IPAL  : {:.2} m³/hari (× 1.5 faktor keamanan)\n", design_capacity_m3));

    result.push_str("\nKARAKTERISTIK TIPIKAL LINDI:\n");
    result.push_str("• BOD     : 2.000 – 30.000 mg/L\n");
    result.push_str("• COD     : 3.000 – 60.000 mg/L\n");
    result.push_str("• NH₃-N   : 200 – 1.000 mg/L\n");
    result.push_str("• pH      : 4 – 9\n");
    result.push_str("• TDS     : 2.000 – 60.000 mg/L\n");
    result.push_str("══════════════════════════════════════════════════════════════════\n");

    result
}
