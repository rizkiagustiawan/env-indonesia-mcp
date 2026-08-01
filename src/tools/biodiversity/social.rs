/// Social Impact Assessment Matrix for AMDAL
/// Ref: PermenLH 17/2012 (Partisipasi Masyarakat)

/// Parse impacts JSON and generate Leopold-style social impact matrix
pub fn impact_matrix(impacts_json: &str) -> String {
    let impacts: Vec<serde_json::Value> = match serde_json::from_str(impacts_json) {
        Ok(v) => v,
        Err(e) => return format!("Error parsing JSON: {}", e),
    };

    if impacts.is_empty() {
        return "Error: Daftar dampak kosong.".to_string();
    }

    let valid_components = [
        "ekonomi",
        "sosial_budaya",
        "kesehatan",
        "demografi",
        "infrastruktur",
    ];

    let mut total_positive = 0.0_f64;
    let mut total_negative = 0.0_f64;
    let mut rows: Vec<String> = Vec::new();
    let mut component_scores: std::collections::HashMap<String, (f64, f64)> =
        std::collections::HashMap::new();

    for (i, impact) in impacts.iter().enumerate() {
        let component = impact
            .get("component")
            .and_then(|v| v.as_str())
            .unwrap_or("lainnya");
        let impact_desc = impact
            .get("impact")
            .and_then(|v| v.as_str())
            .unwrap_or("Tidak diketahui");
        let magnitude = impact
            .get("magnitude")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let importance = impact
            .get("importance")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let score = magnitude * importance;
        let sifat = if magnitude >= 0.0 {
            "Positif (+)"
        } else {
            "Negatif (-)"
        };

        if magnitude >= 0.0 {
            total_positive += score;
        } else {
            total_negative += score;
        }

        let entry = component_scores
            .entry(component.to_string())
            .or_insert((0.0, 0.0));
        if magnitude >= 0.0 {
            entry.0 += score;
        } else {
            entry.1 += score;
        }

        let component_valid = if valid_components.contains(&component) {
            component.to_string()
        } else {
            format!("{} (!)", component)
        };

        rows.push(format!(
            "│ {:>2} │ {:15} │ {:40} │ {:>6.1} │ {:>6.1} │ {:>8.1} │ {:10} │",
            i + 1,
            component_valid,
            impact_desc,
            magnitude,
            importance,
            score,
            sifat
        ));
    }

    let total_score = total_positive + total_negative;
    let assessment = if total_score > 20.0 {
        "LAYAK - Dampak sosial positif dominan"
    } else if total_score > 0.0 {
        "LAYAK BERSYARAT - Dampak positif sedikit lebih besar, perlu pengelolaan"
    } else if total_score > -20.0 {
        "PERLU PERHATIAN - Dampak negatif cukup signifikan, perlu mitigasi intensif"
    } else {
        "PERLU KAJIAN ULANG - Dampak sosial negatif dominan"
    };

    let mut result = String::new();
    result.push_str(
        "══════════════════════════════════════════════════════════════════════════════\n",
    );
    result.push_str("MATRIKS DAMPAK SOSIAL (TIPE LEOPOLD)\n");
    result.push_str("Ref: PermenLH 17/2012 tentang Partisipasi Masyarakat dalam AMDAL\n");
    result.push_str(
        "══════════════════════════════════════════════════════════════════════════════\n\n",
    );
    result.push_str("Skala magnitude : -10 (sangat negatif) s/d +10 (sangat positif)\n");
    result.push_str("Skala importance: 1 (rendah) s/d 10 (sangat penting)\n");
    result.push_str("Skor = Magnitude x Importance\n\n");
    result.push_str("┌────┬─────────────────┬──────────────────────────────────────────┬────────┬────────┬──────────┬────────────┐\n");
    result.push_str("│ No │ Komponen        │ Dampak                                   │ Magn.  │ Import.│ Skor     │ Sifat      │\n");
    result.push_str("├────┼─────────────────┼──────────────────────────────────────────┼────────┼────────┼──────────┼────────────┤\n");

    for row in &rows {
        result.push_str(row);
        result.push('\n');
    }

    result.push_str("└────┴─────────────────┴──────────────────────────────────────────┴────────┴────────┴──────────┴────────────┘\n\n");

    result.push_str("REKAPITULASI PER KOMPONEN:\n");
    for (comp, (pos, neg)) in &component_scores {
        result.push_str(&format!(
            "  {} : Positif={:.1}, Negatif={:.1}, Netto={:.1}\n",
            comp,
            pos,
            neg,
            pos + neg
        ));
    }

    result.push_str(&format!(
        "\nTOTAL SKOR:\n  Positif : +{:.1}\n  Negatif : {:.1}\n  Netto   : {:.1}\n\n\
         PENILAIAN: {}\n",
        total_positive, total_negative, total_score, assessment
    ));

    result.push_str(
        "\n══════════════════════════════════════════════════════════════════════════════\n",
    );
    result
}

/// Simple exposure pathway health impact analysis
/// Calculate Average Daily Dose (ADD) and Hazard Quotient (HQ)
pub fn health_impact(
    population: u64,
    pollutant: &str,
    concentration: f64,
    exposure_hours: f64,
) -> String {
    // Reference data for common pollutants
    struct PollutantRef {
        name: &'static str,
        unit: &'static str,
        rfd: f64,
        rfd_unit: &'static str,
        cancer_slope: Option<f64>,
        ir: f64,
        description: &'static str,
    }

    let pollutants = vec![
        PollutantRef {
            name: "pm25",
            unit: "µg/m³",
            rfd: 0.0035,
            rfd_unit: "mg/kg/hari",
            cancer_slope: None,
            ir: 20.0,
            description: "PM2.5 (Partikulat Halus)",
        },
        PollutantRef {
            name: "pm10",
            unit: "µg/m³",
            rfd: 0.005,
            rfd_unit: "mg/kg/hari",
            cancer_slope: None,
            ir: 20.0,
            description: "PM10 (Partikulat Kasar)",
        },
        PollutantRef {
            name: "so2",
            unit: "µg/m³",
            rfd: 0.02,
            rfd_unit: "mg/kg/hari",
            cancer_slope: None,
            ir: 20.0,
            description: "Sulfur Dioksida (SO₂)",
        },
        PollutantRef {
            name: "no2",
            unit: "µg/m³",
            rfd: 0.02,
            rfd_unit: "mg/kg/hari",
            cancer_slope: None,
            ir: 20.0,
            description: "Nitrogen Dioksida (NO₂)",
        },
        PollutantRef {
            name: "co",
            unit: "mg/m³",
            rfd: 0.057,
            rfd_unit: "mg/kg/hari",
            cancer_slope: None,
            ir: 20.0,
            description: "Karbon Monoksida (CO)",
        },
        PollutantRef {
            name: "benzene",
            unit: "µg/m³",
            rfd: 0.004,
            rfd_unit: "mg/kg/hari",
            cancer_slope: Some(0.029),
            ir: 20.0,
            description: "Benzena (C₆H₆)",
        },
        PollutantRef {
            name: "toluene",
            unit: "µg/m³",
            rfd: 0.08,
            rfd_unit: "mg/kg/hari",
            cancer_slope: None,
            ir: 20.0,
            description: "Toluena (C₇H₈)",
        },
        PollutantRef {
            name: "pb",
            unit: "µg/m³",
            rfd: 0.00036,
            rfd_unit: "mg/kg/hari",
            cancer_slope: Some(0.042),
            ir: 20.0,
            description: "Timbal (Pb)",
        },
        PollutantRef {
            name: "h2s",
            unit: "µg/m³",
            rfd: 0.003,
            rfd_unit: "mg/kg/hari",
            cancer_slope: None,
            ir: 20.0,
            description: "Hidrogen Sulfida (H₂S)",
        },
        PollutantRef {
            name: "nh3",
            unit: "µg/m³",
            rfd: 0.1,
            rfd_unit: "mg/kg/hari",
            cancer_slope: None,
            ir: 20.0,
            description: "Amonia (NH₃)",
        },
    ];

    let query = pollutant.to_lowercase();
    let pol = match pollutants.iter().find(|p| p.name == query) {
        Some(p) => p,
        None => {
            let available: Vec<&str> = pollutants.iter().map(|p| p.name).collect();
            return format!(
                "Polutan '{}' tidak ditemukan.\nPolutan tersedia: {}",
                pollutant,
                available.join(", ")
            );
        }
    };

    // Parameters for inhalation exposure
    let bw = 70.0_f64; // body weight kg (adult)
    let at_noncarc = 365.0 * 30.0; // averaging time non-carcinogenic (30 years, days)
    let at_carc = 365.0 * 70.0; // averaging time carcinogenic (lifetime, days)
    let ef = 350.0; // exposure frequency (days/year)
    let ed = 30.0; // exposure duration (years)
    let et = exposure_hours; // exposure time (hours/day)
    let ir = pol.ir; // inhalation rate (m³/day)

    // Convert concentration to mg/m³ if in µg/m³
    let conc_mg = if pol.unit == "µg/m³" {
        concentration / 1000.0
    } else {
        concentration
    };

    // ADD = C × IR × (ET/24) × EF × ED / (BW × AT)
    let add_noncarc = conc_mg * ir * (et / 24.0) * ef * ed / (bw * at_noncarc);
    let add_carc = conc_mg * ir * (et / 24.0) * ef * ed / (bw * at_carc);

    // Hazard Quotient
    let hq = add_noncarc / pol.rfd;

    let risk_level = if hq > 4.0 {
        "SANGAT TINGGI - Risiko kesehatan signifikan, tindakan segera diperlukan"
    } else if hq > 1.0 {
        "TINGGI - Melebihi batas aman, mitigasi diperlukan"
    } else if hq > 0.5 {
        "SEDANG - Mendekati batas aman, perlu pemantauan ketat"
    } else {
        "RENDAH - Di bawah batas aman"
    };

    let affected_estimate = if hq > 1.0 {
        (population as f64 * (1.0 - 1.0 / hq).min(0.5)) as u64
    } else {
        0
    };

    let mut result = format!(
        "══════════════════════════════════════════════\n\
         ANALISIS RISIKO KESEHATAN LINGKUNGAN\n\
         Ref: PermenLH 17/2012, US EPA IRIS Database\n\
         ══════════════════════════════════════════════\n\n\
         POLUTAN: {}\n\
         Konsentrasi: {:.4} {}\n\
         Populasi terpapar: {} jiwa\n\
         Durasi paparan: {:.1} jam/hari\n\n\
         PARAMETER PAPARAN:\n\
         • Laju inhalasi (IR): {:.1} m³/hari\n\
         • Berat badan (BW): {:.0} kg\n\
         • Frekuensi paparan (EF): {:.0} hari/tahun\n\
         • Durasi paparan (ED): {:.0} tahun\n\n\
         HASIL PERHITUNGAN:\n\
         • ADD (non-karsinogenik): {:.6} mg/kg/hari\n\
         • Reference Dose (RfD): {:.6} {}\n\
         • Hazard Quotient (HQ): {:.4}\n",
        pol.description,
        concentration,
        pol.unit,
        population,
        exposure_hours,
        ir,
        bw,
        ef,
        ed,
        add_noncarc,
        pol.rfd,
        pol.rfd_unit,
        hq
    );

    // Cancer risk if applicable
    if let Some(csf) = pol.cancer_slope {
        let cancer_risk = add_carc * csf;
        let cancer_level = if cancer_risk > 1e-4 {
            "TIDAK DAPAT DITERIMA (> 10⁻⁴)"
        } else if cancer_risk > 1e-6 {
            "PERLU PERHATIAN (10⁻⁶ - 10⁻⁴)"
        } else {
            "DAPAT DITERIMA (< 10⁻⁶)"
        };
        result.push_str(&format!(
            "\nRISIKO KARSINOGENIK:\n\
             • ADD (karsinogenik): {:.8} mg/kg/hari\n\
             • Cancer Slope Factor: {:.4} (mg/kg/hari)⁻¹\n\
             • Excess Cancer Risk (ECR): {:.2e}\n\
             • Kategori: {}\n",
            add_carc, csf, cancer_risk, cancer_level
        ));
    }

    result.push_str(&format!("\nTINGKAT RISIKO: {}\n", risk_level));

    if affected_estimate > 0 {
        result.push_str(&format!(
            "Estimasi penduduk terdampak: ~{} jiwa\n",
            affected_estimate
        ));
    }

    result.push_str("\nREKOMENDASI:\n");
    if hq > 1.0 {
        result.push_str("• Kurangi emisi di sumber (pengendalian polusi)\n");
        result.push_str("• Tingkatkan jarak buffer antara sumber dan permukiman\n");
        result.push_str("• Program pemeriksaan kesehatan berkala bagi masyarakat terpapar\n");
        result.push_str("• Pertimbangkan relokasi reseptor sensitif (sekolah, RS)\n");
    } else {
        result.push_str("• Lakukan pemantauan berkala untuk memastikan konsentrasi tetap aman\n");
        result.push_str("• Pertahankan jarak buffer yang ada\n");
    }

    result.push_str("\n══════════════════════════════════════════════\n");
    result
}
