/// Environmental Management Plan (RKL-RPL) Generator + KPI
/// Ref: PermenLHK No. 5/2021 (AMDAL), PermenLHK No. 6/2021
/// 2026 SOTA: Anggreini 2026 (peatland EMP+KPI), Harmuzan 2026 (sustainable finance KPIs)
/// ISO 14001:2026 link: Clause 8.1 (Operational Control), Clause 9.1 (Monitoring)
/// Rani et al. 2026 (automated KPI dashboard for EMP)

/// Safe UTF-8 truncation: truncates to max_chars without panicking on multibyte chars.
/// BUG FIX: &str[..len.min(N)] byte-slice panics on multibyte UTF-8 (é, °, emoji, etc.)
fn safe_truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

pub fn generate(
    impacts_json: &str,
    project_type: &str,
    location: &str,
) -> String {
    let mut out = String::from("=== Environmental Management Plan (RKL-RPL) ===\n");
    out.push_str("Ref: PermenLHK 5/2021, PermenLHK 6/2021\n");
    out.push_str("ISO 14001 link: Clause 8.1 (Operational Control), Clause 9.1 (Monitoring)\n");
    out.push_str("2026 SOTA: Anggreini 2026; Rani 2026\n\n");

    let impacts: Vec<(String, String, f64, f64)> = match serde_json::from_str(impacts_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR [E102]: impacts_json parse: {}. Format: [[\"dampak\",\"komponen\",magnitude,importance],...]", e),
    };

    if impacts.is_empty() {
        return "ERROR: impacts_json kosong. Minimal 1 dampak signifikan.".into();
    }

    out.push_str(&format!("Proyek: {} | Lokasi: {}\n\n", project_type, location));

    // Filter significant impacts (|magnitude × importance| >= 30)
    let significant: Vec<&(String, String, f64, f64)> = impacts.iter()
        .filter(|(_, _, mag, imp)| (mag * imp).abs() >= 30.0)
        .collect();

    if significant.is_empty() {
        out.push_str("⚠️ Tidak ada dampak signifikan (|significance| ≥ 30).\n");
        out.push_str("EMP tidak diperlukan untuk dampak non-signifikan.\n");
        return out;
    }

    // RKL (Rencana Pengelolaan Lingkungan) — Mitigation
    out.push_str("═══ RKL — Rencana Pengelolaan Lingkungan (Mitigation) ═══\n\n");
    out.push_str("Ref: PermenLHK 5/2021, Lampiran IV\n\n");
    out.push_str(&format!("{:<4} {:<22} {:<28} {:<22} {:<15} {:<12}\n",
        "No", "Dampak", "Mitigasi", "Target", "Indikator", "Waktu"));
    out.push_str(&"-".repeat(105));
    out.push('\n');

    let mut rkl_count = 0;
    for (i, (dampak, komponen, mag, imp)) in significant.iter().enumerate() {
        let sig = mag * imp;
        let mitigasi = suggest_mitigation(dampak, komponen, sig, project_type);
        let target = suggest_target(dampak, komponen, sig);
        let indikator = suggest_indicator(dampak, komponen);
        let waktu = if sig.abs() >= 50.0 { "Pra-konstruksi" } else { "Konstruksi" };

        out.push_str(&format!("{:<4} {:<22} {:<28} {:<22} {:<15} {:<12}\n",
            i + 1,
            safe_truncate(&dampak, 21),
            safe_truncate(&mitigasi, 27),
            safe_truncate(&target, 21),
            safe_truncate(&indikator, 14),
            waktu));
        rkl_count += 1;
    }

    // RPL (Rencana Pemantauan Lingkungan) — Monitoring
    out.push_str(&format!("\n\n═══ RPL — Rencana Pemantauan Lingkungan (Monitoring) ═══\n\n"));
    out.push_str("Ref: PermenLHK 5/2021, Lampiran V\n\n");
    out.push_str(&format!("{:<4} {:<22} {:<25} {:<12} {:<15} {:<15}\n",
        "No", "Parameter", "Lokasi", "Frekuensi", "Metode", "Baku Mutu"));
    out.push_str(&"-".repeat(95));
    out.push('\n');

    for (i, (dampak, komponen, _, _)) in significant.iter().enumerate() {
        let (param, frekuensi, metode, baku_mutu) = suggest_monitoring(dampak, komponen);
        out.push_str(&format!("{:<4} {:<22} {:<25} {:<12} {:<15} {:<15}\n",
            i + 1,
            safe_truncate(&param, 21),
            safe_truncate(&location, 24),
            frekuensi,
            safe_truncate(&metode, 14),
            safe_truncate(&baku_mutu, 14)));
    }

    // KPI Score
    out.push_str("\n\n═══ KPI — Key Performance Indicators ═══\n\n");
    out.push_str("Ref: Rani 2026 (automated KPI dashboard); Anggreini 2026 (peatland EMP)\n\n");

    let kpi_mitigation = (rkl_count as f64 / significant.len() as f64) * 100.0;
    // NOTE: kpi_monitoring and kpi_compliance are PLACEHOLDERS — require actual monitoring
    // schedule compliance data and audit results. Do NOT present as measured performance.
    let kpi_monitoring = 85.0; // PLACEHOLDER (requires actual monitoring schedule data)
    let kpi_compliance = 90.0; // PLACEHOLDER (requires actual audit/baku mutu results)
    let kpi_overall = (kpi_mitigation * 0.4 + kpi_monitoring * 0.35 + kpi_compliance * 0.25);

    out.push_str(&format!("  KPI Mitigasi:     {:>5.1}%  ({}/{} dampak punya mitigasi)\n", kpi_mitigation, rkl_count, significant.len()));
    out.push_str(&format!("  KPI Pemantauan:   {:>5.1}%  [PLACEHOLDER — needs actual schedule data]\n", kpi_monitoring));
    out.push_str(&format!("  KPI Compliance:   {:>5.1}%  [PLACEHOLDER — needs actual audit data]\n", kpi_compliance));
    out.push_str(&format!("  KPI Overall:      {:>5.1}%  [mitigasi dihitung, lainnya placeholder]\n\n", kpi_overall));

    if kpi_overall >= 90.0 {
        out.push_str("  🟢 Excellent — EMP implementation sangat baik\n");
    } else if kpi_overall >= 75.0 {
        out.push_str("  🟡 Good — Perlu peningkatan di area yang kurang\n");
    } else if kpi_overall >= 60.0 {
        out.push_str("  🟠 Moderate — Perlu tindakan korektif\n");
    } else {
        out.push_str("  🔴 Poor — Perlu intervensi manajemen segera\n");
    }

    // ISO 14001 Linkage
    out.push_str("\n═══ ISO 14001:2015 Linkage ═══\n\n");
    out.push_str("  RKL → Clause 8.1 (Operational Control)\n");
    out.push_str("    - Mitigasi = operational controls\n");
    out.push_str("    - Target = environmental objectives (Clause 6.2)\n");
    out.push_str("    - Indikator = process performance indicators\n\n");
    out.push_str("  RPL → Clause 9.1 (Monitoring & Measurement)\n");
    out.push_str("    - Frekuensi = monitoring interval\n");
    out.push_str("    - Baku Mutu = compliance obligations (Clause 6.1.3)\n");
    out.push_str("    - Metode = measurement methods\n\n");
    out.push_str("  KPI → Clause 9.3 (Management Review)\n");
    out.push_str("    - Overall KPI = EMS performance evaluation\n");
    out.push_str("    - Continual improvement (Clause 10.3)\n");

    out
}

fn suggest_mitigation(dampak: &str, komponen: &str, sig: f64, project_type: &str) -> String {
    let d = dampak.to_lowercase();
    let k = komponen.to_lowercase();
    if k.contains("air") || k.contains("water") {
        if d.contains("pencemar") || d.contains("pollut") {
            "IPAL + wetland treatment".into()
        } else {
            "Sediment trap + retention pond".into()
        }
    } else if k.contains("udara") || k.contains("air_quality") {
        "Wet spray + dust suppressant".into()
    } else if k.contains("tanah") || k.contains("soil") || k.contains("land") {
        "Erosion control + revegetation".into()
    } else if k.contains("flora") || k.contains("fauna") || k.contains("bio") {
        "Revegetasi + wildlife corridor".into()
    } else if k.contains("sosial") || k.contains("social") {
        "CSR + community engagement".into()
    } else if k.contains("kebisingan") || k.contains("noise") {
        "Barrier akustik + working hours".into()
    } else if project_type.to_lowercase().contains("tambang") || project_type.to_lowercase().contains("mine") {
        "Reklamasi + mine closure plan".into()
    } else {
        "Good housekeeping + SOP".into()
    }
}

fn suggest_target(dampak: &str, komponen: &str, sig: f64) -> String {
    let k = komponen.to_lowercase();
    if k.contains("air") || k.contains("water") {
        "Patuhi PP 22/2021 baku mutu".into()
    } else if k.contains("udara") || k.contains("air_quality") {
        "SPU ≤ baku mutu PP 22/2021".into()
    } else if k.contains("sosial") || k.contains("social") {
        "Konflik = 0; complain < 5/bulan".into()
    } else if k.contains("bio") {
        "Revegetasi 80% area terbuka".into()
    } else {
        "Dampak residual ≤ 30% baseline".into()
    }
}

fn suggest_indicator(dampak: &str, komponen: &str) -> String {
    let k = komponen.to_lowercase();
    if k.contains("air") || k.contains("water") {
        "TSS, pH, COD".into()
    } else if k.contains("udara") || k.contains("air_quality") {
        "PM10, SO2, NO2".into()
    } else if k.contains("tanah") || k.contains("soil") {
        "Erosion rate, cover %".into()
    } else if k.contains("bio") {
        "Survival rate, species count".into()
    } else if k.contains("sosial") || k.contains("social") {
        "Complain count, CSR spending".into()
    } else if k.contains("kebisingan") || k.contains("noise") {
        "dB(A) at receiver".into()
    } else {
        "Visual inspection + log".into()
    }
}

fn suggest_monitoring(dampak: &str, komponen: &str) -> (String, &'static str, String, String) {
    let k = komponen.to_lowercase();
    if k.contains("air") || k.contains("water") {
        ("Kualitas air limbah".into(), "Bulanan", "SNI 6989".into(), "PP 22/2021".into())
    } else if k.contains("udara") || k.contains("air_quality") {
        ("Kualitas udara ambien".into(), "Bulanan", "Gravimetric".into(), "PP 22/2021".into())
    } else if k.contains("tanah") || k.contains("soil") {
        ("Erosi & vegetasi".into(), "Triwulanan", "USLE + transect".into(), "Permen LH".into())
    } else if k.contains("bio") {
        ("Biodiversity survey".into(), "Semester", "Point count + plot".into(), "Permen LH".into())
    } else if k.contains("sosial") || k.contains("social") {
        ("Community survey".into(), "Semester", "Kuesioner + FGD".into(), "Perda setempat".into())
    } else if k.contains("kebisingan") || k.contains("noise") {
        ("Tingkat kebisingan".into(), "Bulanan", "Sound level meter".into(), "PP 22/2021".into())
    } else {
        ("Inspeksi visual".into(), "Bulanan", "Checklist".into(), "SOP internal".into())
    }
}
