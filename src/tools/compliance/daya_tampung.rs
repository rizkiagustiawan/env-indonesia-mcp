/// Daya Tampung Beban Pencemaran
/// Ref: PP 22/2021 (mencabut PP 82/2001)

pub fn calculate(
    q_river_m3s: f64,
    c_upstream_mgl: f64,
    c_standard_mgl: f64,
    q_waste_m3s: f64,
    c_waste_mgl: f64,
    parameter: &str,
) -> String {
    if q_river_m3s <= 0.0 {
        return format!("ERROR [E102]: Parameter harus > 0. {}", q_river_m3s);
    }
    if q_waste_m3s < 0.0 {
        return format!(
            "ERROR [E102]: Parameter tidak boleh negatif. {}",
            q_waste_m3s
        );
    }
    if c_standard_mgl <= 0.0 {
        return format!("ERROR [E102]: Parameter harus > 0. {}", c_standard_mgl);
    }

    // Mass balance: DTBP = Q_river * (C_standard - C_upstream) - Q_waste * C_waste
    // Convert m³/s to kg/day: Q(m³/s) * C(mg/L) * 86400 / 1000 = Q * C * 86.4 (kg/day)
    let factor = 86.4; // m³/s * mg/L -> kg/day

    let load_capacity = q_river_m3s * (c_standard_mgl - c_upstream_mgl) * factor;
    let load_waste = q_waste_m3s * c_waste_mgl * factor;
    let dtbp = load_capacity - load_waste;

    let max_allowable_load = q_river_m3s * (c_standard_mgl - c_upstream_mgl) * factor;
    let max_waste_conc = if q_waste_m3s > 0.0 {
        (q_river_m3s * (c_standard_mgl - c_upstream_mgl)) / q_waste_m3s
    } else {
        f64::INFINITY
    };

    let status = if dtbp > 0.0 {
        "Masih Ada Kapasitas ✅"
    } else {
        "Daya Tampung Terlampaui ❌"
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Daya Tampung Beban Pencemaran\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("Ref: PP No. 82 Tahun 2001\n\n");
    out.push_str(&format!("Parameter        : {}\n", parameter));
    out.push_str(&format!("Debit Sungai (Q) : {:.4} m³/s\n", q_river_m3s));
    out.push_str(&format!("C Hulu           : {:.2} mg/L\n", c_upstream_mgl));
    out.push_str(&format!("Baku Mutu (C_bm) : {:.2} mg/L\n", c_standard_mgl));
    out.push_str(&format!("Debit Limbah (q) : {:.4} m³/s\n", q_waste_m3s));
    out.push_str(&format!("C Limbah         : {:.2} mg/L\n\n", c_waste_mgl));
    out.push_str("Perhitungan Mass Balance:\n");
    out.push_str(&format!("  Kapasitas Beban = Q × (C_bm - C_hulu) × 86.4\n"));
    out.push_str(&format!(
        "                  = {:.4} × ({:.2} - {:.2}) × 86.4\n",
        q_river_m3s, c_standard_mgl, c_upstream_mgl
    ));
    out.push_str(&format!(
        "                  = {:.2} kg/hari\n\n",
        load_capacity
    ));
    out.push_str(&format!("  Beban Limbah    = q × C_limbah × 86.4\n"));
    out.push_str(&format!(
        "                  = {:.4} × {:.2} × 86.4\n",
        q_waste_m3s, c_waste_mgl
    ));
    out.push_str(&format!(
        "                  = {:.2} kg/hari\n\n",
        load_waste
    ));
    out.push_str(&format!(
        "  DTBP = {:.2} - {:.2} = {:.2} kg/hari\n\n",
        load_capacity, load_waste, dtbp
    ));
    out.push_str(&format!(
        "Maks. Beban Izin    : {:.2} kg/hari\n",
        max_allowable_load
    ));
    if max_waste_conc.is_finite() {
        out.push_str(&format!(
            "Maks. C Limbah Izin : {:.2} mg/L (pada debit limbah saat ini)\n\n",
            max_waste_conc
        ));
    } else {
        out.push_str("Maks. C Limbah Izin : ∞ (tidak ada debit limbah)\n\n");
    }
    out.push_str(&format!("Status: {}\n", status));
    out
}
