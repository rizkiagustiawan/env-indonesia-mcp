/// Acid Rain Risk Calculator
/// Ref: Critical Load methodology, EMEP

pub fn calculate(so2_ugm3: f64, nox_ugm3: f64, rainfall_mm_yr: f64) -> String {
    if so2_ugm3 < 0.0 || nox_ugm3 < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }

    let s_dep = so2_ugm3 * rainfall_mm_yr * 0.001 * 0.64; // simplified wet deposition (eq S/ha/yr)
    let n_dep = nox_ugm3 * rainfall_mm_yr * 0.001 * 0.30;

    // Critical loads (typical tropical forest)
    let cl_s = 500.0; // eq/ha/yr
    let cl_n = 700.0;

    let exceed_s = s_dep > cl_s;
    let exceed_n = n_dep > cl_n;

    let ph_rain = if so2_ugm3 + nox_ugm3 > 100.0 {
        4.0 + (100.0 / (so2_ugm3 + nox_ugm3)).log10()
    } else {
        5.6
    };

    let mut out = format!("=== Acid Rain Risk ===\n\nSO₂: {:.1} µg/m³\nNOx: {:.1} µg/m³\nCurah hujan: {:.0} mm/tahun\n\n", so2_ugm3, nox_ugm3, rainfall_mm_yr);
    out.push_str(&format!(
        "Deposisi S: {:.0} eq/ha/tahun {}\n",
        s_dep,
        if exceed_s {
            "❌ > Critical Load"
        } else {
            "✅"
        }
    ));
    out.push_str(&format!(
        "Deposisi N: {:.0} eq/ha/tahun {}\n",
        n_dep,
        if exceed_n {
            "❌ > Critical Load"
        } else {
            "✅"
        }
    ));
    out.push_str(&format!(
        "pH hujan estimasi: {:.1} (normal: 5.6, asam: <5.0)\n",
        ph_rain
    ));
    out
}
