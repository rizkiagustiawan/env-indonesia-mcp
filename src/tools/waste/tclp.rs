/// Skrining TCLP (Toxicity Characteristic Leaching Procedure)
/// Ref: PP 101/2014 tentang Pengelolaan Limbah Bahan Berbahaya dan Beracun

pub fn screen(parameters_json: &str) -> String {
    let params: Vec<serde_json::Value> = match serde_json::from_str(parameters_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parsing JSON: {}", e),
    };

    if params.is_empty() {
        return "ERROR: Array parameter TCLP kosong.".into();
    }

    // TCLP regulatory limits per PP 101/2014 Lampiran (mg/L)
    struct TclpLimit {
        name: &'static str,
        cas: &'static str,
        limit_mgl: f64,
    }

    let limits = [
        TclpLimit { name: "As", cas: "7440-38-2", limit_mgl: 5.0 },
        TclpLimit { name: "Ba", cas: "7440-39-3", limit_mgl: 100.0 },
        TclpLimit { name: "Cd", cas: "7440-43-9", limit_mgl: 1.0 },
        TclpLimit { name: "Cr", cas: "7440-47-3", limit_mgl: 5.0 },
        TclpLimit { name: "Pb", cas: "7439-92-1", limit_mgl: 5.0 },
        TclpLimit { name: "Hg", cas: "7439-97-6", limit_mgl: 0.2 },
        TclpLimit { name: "Se", cas: "7782-49-2", limit_mgl: 1.0 },
        TclpLimit { name: "Ag", cas: "7440-22-4", limit_mgl: 5.0 },
        TclpLimit { name: "F", cas: "16984-48-8", limit_mgl: 150.0 },
    ];

    let mut rows = Vec::new();
    let mut any_fail = false;
    let mut fail_params = Vec::new();

    for param in &params {
        let name = match param.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => {
                rows.push("│ (?)  │ ERROR: field 'name' tidak ada               │".to_string());
                continue;
            }
        };
        let concentration = match param.get("concentration_mgl").and_then(|v| v.as_f64()) {
            Some(c) => c,
            None => {
                rows.push(format!("│ {:4} │ ERROR: field 'concentration_mgl' tidak ada   │", name));
                continue;
            }
        };

        let name_upper = name.to_uppercase();
        let name_trimmed = name_upper.trim();

        // Find matching limit
        let matched_limit = limits.iter().find(|l| l.name.to_uppercase() == name_trimmed);

        let (limit_str, status, is_fail) = match matched_limit {
            Some(lim) => {
                if concentration > lim.limit_mgl {
                    (format!("{:.1}", lim.limit_mgl), "GAGAL ✗", true)
                } else {
                    (format!("{:.1}", lim.limit_mgl), "LULUS ✓", false)
                }
            }
            None => ("-".to_string(), "N/A   ", false),
        };

        if is_fail {
            any_fail = true;
            fail_params.push(name.to_string());
        }

        rows.push(format!(
            "│ {:4} │ {:>12.4} │ {:>10} │ {} │",
            name_trimmed, concentration, limit_str, status
        ));
    }

    let classification = if any_fail { "LIMBAH B3" } else { "NON-B3" };

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("SKRINING TCLP — KARAKTERISTIK TOKSISITAS\n");
    result.push_str("Ref: PP 101/2014 Lampiran, US EPA SW-846 Method 1311\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str("┌──────┬──────────────┬────────────┬────────┐\n");
    result.push_str("│ Para │  Konsentrasi │ Baku Mutu  │ Status │\n");
    result.push_str("│ meter│    (mg/L)    │   (mg/L)   │        │\n");
    result.push_str("├──────┼──────────────┼────────────┼────────┤\n");
    for row in &rows {
        result.push_str(row);
        result.push('\n');
    }
    result.push_str("└──────┴──────────────┴────────────┴────────┘\n\n");

    result.push_str(&format!("KLASIFIKASI: {}\n\n", classification));

    if any_fail {
        result.push_str(&format!("Parameter melebihi baku mutu: {}\n\n", fail_params.join(", ")));
        result.push_str("KONSEKUENSI (PP 101/2014):\n");
        result.push_str("• Wajib dikelola sebagai limbah B3\n");
        result.push_str("• Wajib TPS B3 berizin\n");
        result.push_str("• Pengelolaan oleh pihak berizin (pengumpul/pengolah/penimbun B3)\n");
        result.push_str("• Pelaporan ke DLHK provinsi setiap 3 bulan\n");
    } else {
        result.push_str("Semua parameter di bawah baku mutu TCLP.\n");
        result.push_str("Limbah dapat dikelola sebagai limbah non-B3 (tetap perlu verifikasi\n");
        result.push_str("karakteristik lain: mudah meledak, mudah terbakar, reaktif, infeksius,\n");
        result.push_str("dan korosif sesuai PP 101/2014).\n");
    }

    result.push_str("\nBATAS TCLP PP 101/2014 (LENGKAP):\n");
    result.push_str("  As: 5.0 | Ba: 100 | Cd: 1.0 | Cr: 5.0 | Pb: 5.0\n");
    result.push_str("  Hg: 0.2 | Se: 1.0 | Ag: 5.0 | F: 150\n");
    result.push_str("══════════════════════════════════════════════\n");

    result
}
