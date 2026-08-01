/// CT Disinfection Calculator
/// CT = C (mg/L) × t (min)
/// Ref: EPA Guidance Manual for Compliance with Surface Water Treatment Rules (GDR)

pub fn calculate(
    disinfectant: &str,
    concentration_mgl: f64,
    contact_time_min: f64,
    target_pathogen: &str,
) -> String {
    let mut out = String::from("=== Kalkulator CT Disinfeksi ===\n");
    out.push_str("Ref: EPA Guidance Manual for Disinfectant Residuals (GDR)\n\n");

    if concentration_mgl <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }
    if contact_time_min <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    let ct_achieved = concentration_mgl * contact_time_min;

    let disinfectant_lower = disinfectant.to_lowercase();
    let pathogen_lower = target_pathogen.to_lowercase();

    // CT tables per EPA GDR (pH 7, 25°C)
    // [1-log, 2-log, 3-log, 4-log]
    let (ct_table, unit_label): (Option<[f64; 4]>, &str) =
        match (disinfectant_lower.as_str(), pathogen_lower.as_str()) {
            ("chlorine", "giardia") => (Some([2.0, 6.0, 12.0, 22.0]), "mg·min/L"),
            ("chlorine", "virus") => (Some([1.0, 3.0, 4.0, 6.0]), "mg·min/L"),
            ("chlorine", "crypto") => (Some([430.0, 860.0, 1290.0, 1720.0]), "mg·min/L"),
            ("ozone", "giardia") => (Some([0.48, 0.95, 1.43, 1.9]), "mg·min/L"),
            ("ozone", "virus") => (Some([0.3, 0.6, 0.9, 1.2]), "mg·min/L"),
            ("ozone", "crypto") => (Some([6.0, 12.0, 18.0, 24.0]), "mg·min/L"),
            ("uv", "virus") => (Some([12.0, 21.0, 36.0, 90.0]), "mJ/cm²"),
            ("uv", "giardia") => (Some([1.0, 2.0, 3.0, 5.0]), "mJ/cm²"),
            ("uv", "crypto") => (Some([3.0, 6.0, 12.0, 22.0]), "mJ/cm²"),
            ("chloramine", "giardia") => (Some([63.0, 126.0, 189.0, 252.0]), "mg·min/L"),
            ("chloramine", "virus") => (Some([214.0, 428.0, 642.0, 856.0]), "mg·min/L"),
            _ => (None, "mg·min/L"),
        };

    out.push_str(&format!("Input:\n  Disinfektan = {}\n  Konsentrasi = {:.2} mg/L\n  Waktu kontak = {:.1} menit\n  Patogen target = {}\n\n",
        disinfectant, concentration_mgl, contact_time_min, target_pathogen));

    if disinfectant_lower == "uv" {
        out.push_str(&format!(
            "Dosis UV = {:.1} mJ/cm² (konsentrasi × waktu sebagai dosis)\n\n",
            ct_achieved
        ));
    } else {
        out.push_str(&format!(
            "CT tercapai = {:.2} × {:.1} = {:.2} mg·min/L\n\n",
            concentration_mgl, contact_time_min, ct_achieved
        ));
    }

    match ct_table {
        Some(table) => {
            out.push_str(&format!(
                "Tabel CT untuk {} terhadap {} (pH 7, 25°C):\n",
                disinfectant, target_pathogen
            ));
            let log_labels = ["1-log", "2-log", "3-log", "4-log"];
            let pct_labels = ["90%", "99%", "99.9%", "99.99%"];
            for i in 0..4 {
                let status = if ct_achieved >= table[i] {
                    "✅"
                } else {
                    "❌"
                };
                out.push_str(&format!(
                    "  {} ({} removal): CT = {:.2} {} {}\n",
                    log_labels[i], pct_labels[i], table[i], unit_label, status
                ));
            }

            // Determine achieved log removal
            let log_removal = if ct_achieved >= table[3] {
                4.0
            } else if ct_achieved >= table[2] {
                3.0 + (ct_achieved - table[2]) / (table[3] - table[2])
            } else if ct_achieved >= table[1] {
                2.0 + (ct_achieved - table[1]) / (table[2] - table[1])
            } else if ct_achieved >= table[0] {
                1.0 + (ct_achieved - table[0]) / (table[1] - table[0])
            } else {
                ct_achieved / table[0]
            };

            out.push_str(&format!(
                "\nLog removal tercapai ≈ {:.2}-log\n",
                log_removal
            ));

            // Compliance
            let compliant = ct_achieved >= table[2]; // 3-log typically required
            out.push_str(&format!(
                "Status kepatuhan (min 3-log): {}\n",
                if compliant {
                    "✅ MEMENUHI"
                } else {
                    "❌ TIDAK MEMENUHI — tingkatkan C atau t"
                }
            ));
        }
        None => {
            out.push_str(&format!(
                "ERROR: Kombinasi disinfektan '{}' dan patogen '{}' tidak tersedia.\n",
                disinfectant, target_pathogen
            ));
            out.push_str("Pilihan disinfektan: chlorine, ozone, uv, chloramine\n");
            out.push_str("Pilihan patogen: giardia, virus, crypto\n");
        }
    }

    out
}
