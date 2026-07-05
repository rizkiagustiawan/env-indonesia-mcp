/// Hukum Kuadrat Terbalik (Inverse Square Law) untuk Radiasi
/// Ref: ICRP 103 (2007), BAPETEN (Badan Pengawas Tenaga Nuklir)

pub fn calculate(dose_rate_at_d1: f64, d1_m: f64, d2_m: f64) -> String {
    if dose_rate_at_d1 <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if d1_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if d2_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    // I₂ = I₁ × (d₁/d₂)²
    let dose_rate_at_d2 = dose_rate_at_d1 * (d1_m / d2_m).powi(2);

    // Safe distance calculations
    // Public: 1 mSv/year = 1000 µSv/year, assume 2000 hr/yr occupancy → 0.5 µSv/hr
    // Worker: 20 mSv/year, assume 2000 hr/yr → 10 µSv/hr
    let target_public_usv_hr = 0.5;  // µSv/hr
    let target_worker_usv_hr = 10.0; // µSv/hr

    // Convert dose_rate to µSv/hr if given in mSv/hr
    // Assume input is in mSv/hr
    let dose_rate_usv_hr_at_d1 = dose_rate_at_d1 * 1000.0;

    // d_safe = d1 × sqrt(I1/I_target)
    let d_safe_public = d1_m * (dose_rate_usv_hr_at_d1 / target_public_usv_hr).sqrt();
    let d_safe_worker = d1_m * (dose_rate_usv_hr_at_d1 / target_worker_usv_hr).sqrt();

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("HUKUM KUADRAT TERBALIK (INVERSE SQUARE LAW)\n");
    result.push_str("Ref: ICRP 103 (2007), BAPETEN\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("FORMULA: I₂ = I₁ × (d₁/d₂)²\n\n");

    result.push_str("INPUT:\n");
    result.push_str(&format!("• Laju dosis pada d₁   : {:.4} mSv/jam\n", dose_rate_at_d1));
    result.push_str(&format!("• Jarak d₁             : {:.2} m\n", d1_m));
    result.push_str(&format!("• Jarak d₂             : {:.2} m\n\n", d2_m));

    result.push_str("HASIL:\n");
    result.push_str(&format!("• Laju dosis pada d₂   : {:.6} mSv/jam\n", dose_rate_at_d2));
    result.push_str(&format!("                       = {:.4} µSv/jam\n", dose_rate_at_d2 * 1000.0));
    result.push_str(&format!("• Rasio (d₁/d₂)²       : {:.4}\n\n", (d1_m / d2_m).powi(2)));

    result.push_str("JARAK AMAN (asumsi okupansi 2000 jam/tahun):\n");
    result.push_str(&format!("• Publik  (1 mSv/thn)  : {:.2} m\n", d_safe_public));
    result.push_str(&format!("• Pekerja (20 mSv/thn) : {:.2} m\n\n", d_safe_worker));

    result.push_str("BATAS DOSIS (BAPETEN / ICRP 103):\n");
    result.push_str("• Pekerja radiasi  : 20 mSv/tahun (rata-rata 5 tahun)\n");
    result.push_str("• Masyarakat umum  : 1 mSv/tahun\n");
    result.push_str("• Lensa mata       : 20 mSv/tahun (pekerja)\n");
    result.push_str("• Kulit            : 500 mSv/tahun (pekerja)\n");
    result.push_str("\nCATATAN: Perhitungan mengasumsikan sumber titik tanpa\n");
    result.push_str("perisai. Untuk sumber terdistribusi, gunakan faktor koreksi.\n");
    result.push_str("══════════════════════════════════════════════\n");

    result
}
