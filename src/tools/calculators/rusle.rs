/// RUSLE: Revised Universal Soil Loss Equation (Renard et al., 1997)
/// A = R × K × LS × C × P (ton/ha/tahun)

pub fn calculate(r: f64, k: f64, ls: f64, c: f64, p: f64) -> String {
    let mut out = String::from("=== RUSLE Soil Erosion Calculator ===\n");
    out.push_str("Ref: USDA Agriculture Handbook 703 (Renard et al., 1997)\n\n");

    // Validate
    if r < 0.0 { return format!("ERROR [E102]: Parameter tidak boleh negatif. {}", r); }
    if k < 0.0 || k > 1.0 { return format!("ERROR: K-erodibility ({}) harus 0-1.", k); }
    if ls < 0.0 { return format!("ERROR [E102]: Parameter tidak boleh negatif. {}", ls); }
    if c < 0.0 || c > 1.0 { return format!("ERROR: C-cover ({}) harus 0-1.", c); }
    if p < 0.0 || p > 1.0 { return format!("ERROR: P-practice ({}) harus 0-1.", p); }

    let a = r * k * ls * c * p;

    out.push_str(&format!("Input:\n  R (erosivitas hujan) = {:.1} MJ·mm/(ha·hr·yr)\n  K (erodibilitas tanah) = {:.3}\n  LS (slope-length) = {:.2}\n  C (tutupan lahan) = {:.3}\n  P (konservasi) = {:.3}\n\n", r, k, ls, c, p));
    out.push_str(&format!("A = R × K × LS × C × P = {:.2} ton/ha/tahun\n\n", a));

    let kelas = if a < 15.0 { "I - Sangat Ringan" } else if a < 60.0 { "II - Ringan" } else if a < 180.0 { "III - Sedang" } else if a < 480.0 { "IV - Berat" } else { "V - Sangat Berat" };
    out.push_str(&format!("Klasifikasi Erosi (Hammer 1981): {}\n", kelas));

    if a > 60.0 {
        out.push_str("\n⚠️ REKOMENDASI: Erosi melebihi ambang sedang.\n");
        out.push_str("  - Terapkan teras bangku (P=0.04-0.15)\n");
        out.push_str("  - Tanam penutup tanah/agroforestri (C=0.01-0.10)\n");
    }
    out
}

pub fn compute_ls(slope_pct: f64, length_m: f64) -> String {
    let theta = (slope_pct / 100.0).atan();
    let m = if slope_pct < 1.0 { 0.2 } else if slope_pct < 3.0 { 0.3 } else if slope_pct < 5.0 { 0.4 } else { 0.5 };
    let ls = (length_m / 22.13_f64).powf(m) * (65.41 * theta.sin().powi(2) + 4.56 * theta.sin() + 0.065);
    format!("=== LS Factor ===\nSlope: {:.1}%\nLength: {:.1} m\nm-exponent: {:.1}\nLS = {:.3}\n", slope_pct, length_m, m, ls)
}
