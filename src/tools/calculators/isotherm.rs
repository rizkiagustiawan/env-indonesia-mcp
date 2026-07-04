/// Adsorption Isotherm: Freundlich & Langmuir
/// Freundlich: qe = Kf × Ce^(1/n)
/// Langmuir: qe = (qmax × KL × Ce) / (1 + KL × Ce)
/// Ref: Metcalf & Eddy (2003), Weber (1972)

pub fn calculate(model: &str, ce: f64, kf: f64, n_exp: f64, qmax: f64, kl: f64, volume_l: f64, c0: f64) -> String {
    let mut out = String::from("=== Isoterm Adsorpsi ===\n");
    out.push_str("Ref: Metcalf & Eddy (2003), Weber (1972)\n\n");

    if ce < 0.0 { return "ERROR: Konsentrasi kesetimbangan (Ce) tidak boleh negatif.".into(); }

    let model_lower = model.to_lowercase();

    match model_lower.as_str() {
        "freundlich" => {
            if kf <= 0.0 { return "ERROR: Kf harus > 0.".into(); }
            if n_exp <= 0.0 { return "ERROR: n harus > 0.".into(); }

            let qe = kf * ce.powf(1.0 / n_exp);

            out.push_str("Model: Freundlich\n");
            out.push_str(&format!("  qe = Kf × Ce^(1/n)\n\n"));
            out.push_str(&format!("Input:\n  Ce = {:.2} mg/L\n  Kf = {:.4}\n  n = {:.2}\n  1/n = {:.4}\n\n", ce, kf, n_exp, 1.0 / n_exp));
            out.push_str(&format!("Hasil:\n  qe = {:.4} × {:.2}^{:.4}\n", kf, ce, 1.0 / n_exp));
            out.push_str(&format!("  qe = {:.4} mg/g\n\n", qe));

            // Favorability
            if 1.0 / n_exp < 1.0 {
                out.push_str("  1/n < 1 → Adsorpsi favorable ✅\n");
            } else {
                out.push_str("  1/n ≥ 1 → Adsorpsi unfavorable ⚠️\n");
            }

            // Mass of adsorbent needed
            if volume_l > 0.0 && c0 > ce {
                let mass_removed = (c0 - ce) * volume_l / 1000.0; // mg → g
                let adsorbent_g = mass_removed / qe;
                out.push_str(&format!("\nMassa adsorben diperlukan:\n  Volume air = {:.1} L\n  C₀ = {:.2} mg/L\n  Massa kontaminan = {:.2} g\n  Adsorben = {:.2} g ({:.3} kg)\n",
                    volume_l, c0, mass_removed, adsorbent_g, adsorbent_g / 1000.0));
            }
        }
        "langmuir" => {
            if qmax <= 0.0 { return "ERROR: qmax harus > 0.".into(); }
            if kl <= 0.0 { return "ERROR: KL harus > 0.".into(); }

            let qe = (qmax * kl * ce) / (1.0 + kl * ce);

            // Separation factor RL = 1/(1+KL×C0)
            let rl = if c0 > 0.0 { 1.0 / (1.0 + kl * c0) } else { 0.0 };

            out.push_str("Model: Langmuir\n");
            out.push_str("  qe = (qmax × KL × Ce) / (1 + KL × Ce)\n\n");
            out.push_str(&format!("Input:\n  Ce = {:.2} mg/L\n  qmax = {:.2} mg/g\n  KL = {:.4} L/mg\n\n", ce, qmax, kl));
            out.push_str(&format!("Hasil:\n  qe = ({:.2} × {:.4} × {:.2}) / (1 + {:.4} × {:.2})\n", qmax, kl, ce, kl, ce));
            out.push_str(&format!("  qe = {:.4} mg/g\n\n", qe));

            // Separation factor
            if c0 > 0.0 {
                out.push_str(&format!("Faktor separasi:\n  RL = 1/(1+KL×C₀) = 1/(1+{:.4}×{:.2}) = {:.4}\n", kl, c0, rl));
                let assessment = if rl <= 0.0 {
                    "Irreversible"
                } else if rl < 1.0 {
                    "Favorable ✅"
                } else if (rl - 1.0).abs() < 1e-10 {
                    "Linear"
                } else {
                    "Unfavorable ⚠️"
                };
                out.push_str(&format!("  Penilaian: {} (0<RL<1 = favorable)\n", assessment));
            }

            // Mass of adsorbent
            if volume_l > 0.0 && c0 > ce {
                let mass_removed = (c0 - ce) * volume_l / 1000.0;
                let adsorbent_g = mass_removed / qe;
                out.push_str(&format!("\nMassa adsorben diperlukan:\n  Volume air = {:.1} L\n  C₀ = {:.2} mg/L\n  Massa kontaminan = {:.2} g\n  Adsorben = {:.2} g ({:.3} kg)\n",
                    volume_l, c0, mass_removed, adsorbent_g, adsorbent_g / 1000.0));
            }
        }
        _ => return format!("ERROR: Model '{}' tidak dikenali. Pilihan: freundlich, langmuir.", model),
    }

    out
}
