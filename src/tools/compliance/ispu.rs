/// ISPU Calculator (Indeks Standar Pencemar Udara)
/// Ref: PermenLHK P.14/2020 (Indeks Standar Pencemar Udara)

pub fn calculate(
    pm10: Option<f64>,
    pm25: Option<f64>,
    so2: Option<f64>,
    co: Option<f64>,
    o3: Option<f64>,
    no2: Option<f64>,
) -> String {
    // Breakpoints: [concentration breakpoints] -> [ISPU breakpoints]
    let ispu_bp: [f64; 6] = [0.0, 50.0, 100.0, 200.0, 300.0, 400.0];

    let pm10_bp: [f64; 6] = [0.0, 50.0, 150.0, 350.0, 420.0, 500.0];
    let pm25_bp: [f64; 6] = [0.0, 15.5, 55.4, 150.4, 250.4, 500.0];
    let so2_bp: [f64; 6] = [0.0, 52.0, 180.0, 400.0, 800.0, 1200.0];
    let co_bp: [f64; 6] = [0.0, 4000.0, 10000.0, 17000.0, 34000.0, 46000.0];
    let o3_bp: [f64; 6] = [0.0, 120.0, 235.0, 400.0, 800.0, 1000.0];
    let no2_bp: [f64; 6] = [0.0, 80.0, 200.0, 1130.0, 2260.0, 3000.0];

    fn interpolate(cp: f64, bp: &[f64; 6], ip: &[f64; 6]) -> Result<f64, String> {
        if cp < 0.0 {
            return Err(format!("Konsentrasi ({:.1}) tidak boleh negatif", cp));
        }
        if cp > bp[5] {
            return Ok(ip[5] + ((cp - bp[5]) / (bp[5] - bp[4])) * (ip[5] - ip[4]));
        }
        for i in 0..5 {
            if cp >= bp[i] && cp <= bp[i + 1] {
                let ispu = ((ip[i + 1] - ip[i]) / (bp[i + 1] - bp[i])) * (cp - bp[i]) + ip[i];
                return Ok(ispu);
            }
        }
        Err("Konsentrasi di luar rentang breakpoint".into())
    }

    fn category(ispu: f64) -> &'static str {
        if ispu <= 50.0 {
            "Baik"
        } else if ispu <= 100.0 {
            "Sedang"
        } else if ispu <= 200.0 {
            "Tidak Sehat"
        } else if ispu <= 300.0 {
            "Sangat Tidak Sehat"
        } else {
            "Berbahaya"
        }
    }

    let mut out = String::from(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  ISPU Calculator\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    out.push_str("Ref: PermenLHK No. P.14 Tahun 2020\n");
    out.push_str("Rumus: ISPU = ((Ih-Il)/(BPh-BPl)) × (Cp - BPl) + Il\n\n");

    let mut results: Vec<(&str, f64)> = Vec::new();
    let mut any_input = false;

    let params: Vec<(&str, Option<f64>, &[f64; 6])> = vec![
        ("PM10", pm10, &pm10_bp),
        ("PM2.5", pm25, &pm25_bp),
        ("SO2", so2, &so2_bp),
        ("CO", co, &co_bp),
        ("O3", o3, &o3_bp),
        ("NO2", no2, &no2_bp),
    ];

    out.push_str(
        format!(
            "{:<8} | {:<12} | {:<8} | {}\n",
            "Param", "Konsentrasi", "ISPU", "Kategori"
        )
        .as_str(),
    );
    out.push_str("─────────┼──────────────┼──────────┼──────────────────\n");

    for (name, val, bp) in &params {
        if let Some(c) = val {
            any_input = true;
            match interpolate(*c, bp, &ispu_bp) {
                Ok(ispu) => {
                    let cat = category(ispu);
                    out.push_str(&format!(
                        "{:<8} | {:<12.1} | {:<8.1} | {}\n",
                        name, c, ispu, cat
                    ));
                    results.push((name, ispu));
                }
                Err(e) => {
                    return format!("ERROR: {} — {}", name, e);
                }
            }
        }
    }

    if !any_input {
        return "ERROR: Minimal satu parameter harus diisi (PM10, PM2.5, SO2, CO, O3, NO2).".into();
    }

    let max_ispu = results.iter().map(|(_, v)| *v).fold(f64::NAN, f64::max);
    let max_param = results
        .iter()
        .find(|(_, v)| (*v - max_ispu).abs() < 0.001)
        .map(|(n, _)| *n)
        .unwrap_or("?");

    out.push_str(&format!(
        "\nISPU Keseluruhan: {:.0} ({})\n",
        max_ispu,
        category(max_ispu)
    ));
    out.push_str(&format!("Parameter Dominan: {}\n\n", max_param));
    out.push_str("Kategori ISPU:\n");
    out.push_str("  0-50     : Baik\n");
    out.push_str("  51-100   : Sedang\n");
    out.push_str("  101-200  : Tidak Sehat\n");
    out.push_str("  201-300  : Sangat Tidak Sehat\n");
    out.push_str("  301+     : Berbahaya\n");
    out
}
