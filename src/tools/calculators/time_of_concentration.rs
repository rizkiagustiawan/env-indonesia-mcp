/// Time of Concentration — Multiple Methods
/// Kirpich, Bransby-Williams, SCS Lag
/// Ref: Kirpich (1940), SCS TR-55 (1986)

pub fn calculate(method: &str, l_m: f64, s_slope: f64, a_km2: f64, cn: f64) -> String {
    let mut out = String::from("=== Waktu Konsentrasi (tc) ===\n");
    out.push_str("Ref: Kirpich (1940), Bransby-Williams, SCS TR-55 (1986)\n\n");

    let method_lower = method.to_lowercase();

    match method_lower.as_str() {
        "kirpich" => {
            // tc = 0.0195 × L^0.77 × S^(-0.385) (tc in min, L in m, S in m/m)
            if l_m <= 0.0 { return "ERROR: Panjang saluran (L) harus > 0.".into(); }
            if s_slope <= 0.0 || s_slope >= 1.0 { return "ERROR: Kemiringan (S) harus antara 0 dan 1 (m/m).".into(); }

            let tc_min = 0.0195 * l_m.powf(0.77) * s_slope.powf(-0.385);
            let tc_hr = tc_min / 60.0;

            out.push_str("Metode: Kirpich (1940)\n");
            out.push_str("  tc = 0.0195 × L^0.77 × S^(-0.385)\n\n");
            out.push_str(&format!("Input:\n  L = {:.0} m\n  S = {:.4} m/m ({:.2}%)\n\n", l_m, s_slope, s_slope * 100.0));
            out.push_str(&format!("Hasil:\n  tc = {:.2} menit ({:.2} jam)\n", tc_min, tc_hr));
        }
        "bransby-williams" | "bransby_williams" => {
            // tc = (58 × L) / (A^0.1 × S^0.2) (L in km, A in km², S in %, tc in min)
            if l_m <= 0.0 { return "ERROR: Panjang saluran (L) harus > 0.".into(); }
            if s_slope <= 0.0 { return "ERROR: Kemiringan (S) harus > 0.".into(); }
            if a_km2 <= 0.0 { return "ERROR: Luas DAS (A) harus > 0 untuk Bransby-Williams.".into(); }

            let l_km = l_m / 1000.0;
            let s_pct = s_slope * 100.0;

            let tc_min = (58.0 * l_km) / (a_km2.powf(0.1) * s_pct.powf(0.2));
            let tc_hr = tc_min / 60.0;

            out.push_str("Metode: Bransby-Williams\n");
            out.push_str("  tc = (58 × L) / (A^0.1 × S^0.2)\n");
            out.push_str("  L dalam km, A dalam km², S dalam %\n\n");
            out.push_str(&format!("Input:\n  L = {:.0} m ({:.2} km)\n  A = {:.2} km²\n  S = {:.4} m/m ({:.2}%)\n\n",
                l_m, l_km, a_km2, s_slope, s_pct));
            out.push_str(&format!("Hasil:\n  tc = {:.2} menit ({:.2} jam)\n", tc_min, tc_hr));
        }
        "scs" | "scs_lag" => {
            // tc = L^0.8 × (S_ret+1)^0.7 / (1140 × Y^0.5)
            // where S_ret = (1000/CN) - 10 (in inches), L in feet, Y in %
            if l_m <= 0.0 { return "ERROR: Panjang saluran (L) harus > 0.".into(); }
            if s_slope <= 0.0 { return "ERROR: Kemiringan (S) harus > 0.".into(); }
            if cn <= 0.0 || cn > 100.0 { return "ERROR: CN harus antara 0-100.".into(); }

            let l_ft = l_m * 3.281; // m to feet
            let y_pct = s_slope * 100.0;
            let s_ret = (1000.0 / cn) - 10.0; // retention in inches

            let tc_hr = l_ft.powf(0.8) * (s_ret + 1.0).powf(0.7) / (1140.0 * y_pct.powf(0.5));
            let tc_min = tc_hr * 60.0;

            out.push_str("Metode: SCS Lag\n");
            out.push_str("  tc = L^0.8 × (S+1)^0.7 / (1140 × Y^0.5)\n");
            out.push_str("  S = (1000/CN) - 10\n\n");
            out.push_str(&format!("Input:\n  L = {:.0} m ({:.0} ft)\n  S (slope) = {:.4} m/m ({:.2}%)\n  CN = {:.0}\n  S (retention) = {:.2} in\n\n",
                l_m, l_ft, s_slope, y_pct, cn, s_ret));
            out.push_str(&format!("Hasil:\n  tc = {:.2} menit ({:.2} jam)\n", tc_min, tc_hr));
        }
        "semua" | "all" => {
            // Calculate all methods and compare
            if l_m <= 0.0 { return "ERROR: Panjang saluran (L) harus > 0.".into(); }
            if s_slope <= 0.0 { return "ERROR: Kemiringan (S) harus > 0.".into(); }

            out.push_str("Perbandingan semua metode:\n\n");

            // Kirpich
            if s_slope < 1.0 {
                let tc_kirpich = 0.0195 * l_m.powf(0.77) * s_slope.powf(-0.385);
                out.push_str(&format!("  Kirpich:          tc = {:.2} menit ({:.2} jam)\n", tc_kirpich, tc_kirpich / 60.0));
            }

            // Bransby-Williams
            if a_km2 > 0.0 {
                let l_km = l_m / 1000.0;
                let s_pct = s_slope * 100.0;
                let tc_bw = (58.0 * l_km) / (a_km2.powf(0.1) * s_pct.powf(0.2));
                out.push_str(&format!("  Bransby-Williams: tc = {:.2} menit ({:.2} jam)\n", tc_bw, tc_bw / 60.0));
            }

            // SCS
            if cn > 0.0 && cn <= 100.0 {
                let l_ft = l_m * 3.281;
                let y_pct = s_slope * 100.0;
                let s_ret = (1000.0 / cn) - 10.0;
                let tc_scs_hr = l_ft.powf(0.8) * (s_ret + 1.0).powf(0.7) / (1140.0 * y_pct.powf(0.5));
                out.push_str(&format!("  SCS Lag:          tc = {:.2} menit ({:.2} jam)\n", tc_scs_hr * 60.0, tc_scs_hr));
            }

            out.push_str("\n→ Gunakan tc terbesar untuk desain konservatif (input ke kurva IDF)\n");
            return out;
        }
        _ => return format!("ERROR: Metode '{}' tidak dikenali.\nPilihan: kirpich, bransby-williams, scs, semua/all.", method),
    }

    out.push_str("\n→ Gunakan tc sebagai input durasi pada kurva IDF untuk menentukan intensitas hujan desain.\n");

    out
}
