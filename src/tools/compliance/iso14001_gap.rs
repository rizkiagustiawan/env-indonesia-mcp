/// ISO 14001:2015 Gap Analysis + PROPER Prediction
/// Ref: ISO 14001:2015 (HLS Clauses 4-10), PermenLHK P.1/2021 (PROPER)
/// 2026 SOTA: Falakh 2026 (ISO 14001 → PROPER mining), Febrian 2026 (Clause 7.3 gap),
///   Altarazi 2026 (ISO 14001 +28-35% compliance, +22-30% energy efficiency),
///   Izati 2026 (gap analysis per clause)
/// PDCA: Plan(4,5,6) → Do(7,8) → Check(9) → Act(10)

struct ClauseDef {
    id: &'static str,
    title: &'static str,
    pdca: &'static str,
    sub_reqs: &'static [&'static str],
}

const CLAUSES: &[ClauseDef] = &[
    ClauseDef { id: "4", title: "Context of the Organization", pdca: "Plan",
        sub_reqs: &["4.1 Internal/external issues", "4.2 Interested party needs", "4.3 Scope of EMS", "4.4 EMS processes"] },
    ClauseDef { id: "5", title: "Leadership", pdca: "Plan",
        sub_reqs: &["5.1 Top management commitment", "5.2 Environmental policy", "5.3 Roles & responsibilities"] },
    ClauseDef { id: "6", title: "Planning", pdca: "Plan",
        sub_reqs: &["6.1 Risks & opportunities", "6.1.2 Environmental aspects", "6.1.3 Compliance obligations", "6.2 Environmental objectives"] },
    ClauseDef { id: "7", title: "Support", pdca: "Do",
        sub_reqs: &["7.1 Resources", "7.2 Competence", "7.3 Awareness", "7.4 Communication", "7.5 Documented information"] },
    ClauseDef { id: "8", title: "Operation", pdca: "Do",
        sub_reqs: &["8.1 Operational control", "8.2 Emergency preparedness"] },
    ClauseDef { id: "9", title: "Performance Evaluation", pdca: "Check",
        sub_reqs: &["9.1 Monitoring & measurement", "9.2 Internal audit", "9.3 Management review"] },
    ClauseDef { id: "10", title: "Improvement", pdca: "Act",
        sub_reqs: &["10.1 General", "10.2 Nonconformity & corrective action", "10.3 Continual improvement"] },
];

pub fn assess(compliance_json: &str) -> String {
    let mut out = String::from("=== ISO 14001:2015 Gap Analysis + PROPER Prediction ===\n");
    out.push_str("Ref: ISO 14001:2015 (HLS), PermenLHK P.1/2021 (PROPER)\n");
    out.push_str("2026 SOTA: Falakh 2026; Febrian 2026; Altarazi 2026; Izati 2026\n");
    out.push_str("PDCA: Plan(4,5,6) → Do(7,8) → Check(9) → Act(10)\n\n");

    // Parse: [[clause_id, sub_req, level(1-5), evidence], ...]
    let items: Vec<(String, String, u8, String)> = match serde_json::from_str(compliance_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR [E102]: compliance_json parse: {}. Format: [[\"clause\",\"sub_req\",level(1-5),\"evidence\"],...]", e),
    };

    if items.is_empty() {
        // Provide template if empty
        out.push_str("ERROR: compliance_json kosong.\n\n");
        out.push_str("Template (level: 1=not implemented, 3=partial, 5=fully implemented):\n");
        out.push_str("[\n");
        for c in CLAUSES {
            for sub in c.sub_reqs {
                out.push_str(&format!("  [\"{}\", \"{}\", 3, \"evidence here\"],\n", c.id, sub));
            }
        }
        out.push_str("]\n\n");
        out.push_str("Isi level untuk setiap sub-requirement lalu jalankan lagi.\n");
        return out;
    }

    // Gap Matrix
    out.push_str("═══ GAP MATRIX ═══\n\n");
    out.push_str(&format!("{:<4} {:<6} {:<35} {:>6} {:<8} {}\n",
        "Cls", "PDCA", "Sub-requirement", "Level", "Gap", "Evidence"));
    out.push_str(&"-".repeat(95));
    out.push('\n');

    let mut clause_scores: Vec<(String, String, &str, f64, u32)> = Vec::new(); // (id, title, pdca, avg, count)
    let mut total_level = 0.0f64;
    let mut total_count = 0u32;

    for c in CLAUSES {
        let mut clause_total = 0.0f64;
        let mut clause_count = 0u32;

        for sub in c.sub_reqs {
            // Find matching item
            let found = items.iter().find(|(cid, s, _, _)| cid == c.id && s.contains(sub));
            let (level, evidence) = if let Some((_, _, l, e)) = found {
                (*l, e.as_str())
            } else {
                (1, "(not assessed)")
            };

            let gap = 5 - level;
            let gap_str = if gap == 0 { "✅ Full" }
                else if gap <= 1 { "🟡 Minor" }
                else if gap <= 2 { "🟠 Major" }
                else { "🔴 Critical" };

            out.push_str(&format!("{:<4} {:<6} {:<35} {:>6} {:<8} {}\n",
                c.id, c.pdca,
                &sub[..sub.len().min(34)],
                level,
                gap_str,
                &evidence[..evidence.len().min(25)]));

            clause_total += level as f64;
            clause_count += 1;
        }

        let avg = clause_total / clause_count as f64;
        clause_scores.push((c.id.into(), c.title.into(), c.pdca, avg, clause_count));
        total_level += clause_total;
        total_count += clause_count;
        out.push('\n');
    }

    // Clause Summary
    out.push_str("\n═══ CLAUSE COMPLIANCE SUMMARY ═══\n\n");
    out.push_str(&format!("{:<4} {:<30} {:<6} {:>8} {:>10}\n", "Cls", "Title", "PDCA", "Score", "Status"));
    out.push_str(&"-".repeat(60));
    out.push('\n');

    for (id, title, pdca, avg, _) in &clause_scores {
        let pct = (avg / 5.0) * 100.0;
        let status = if pct >= 90.0 { "🟢 Excellent" }
            else if pct >= 75.0 { "🟡 Good" }
            else if pct >= 50.0 { "🟠 Moderate" }
            else { "🔴 Poor" };
        out.push_str(&format!("{:<4} {:<30} {:<6} {:>7.1}% {}\n",
            id, &title[..title.len().min(29)], pdca, pct, status));
    }

    // Overall Score
    let overall_avg = total_level / total_count as f64;
    let overall_pct = (overall_avg / 5.0) * 100.0;
    out.push_str(&format!("\n  Overall Compliance: {:.1}% (avg level {:.2}/5)\n", overall_pct, overall_avg));

    // PDCA Balance
    out.push_str("\n═══ PDCA BALANCE ═══\n\n");
    let pdca_groups: [(char, &[&str]); 4] = [('P', &["4", "5", "6"]), ('D', &["7", "8"]), ('C', &["9"]), ('A', &["10"])];
    for (phase, clause_ids) in pdca_groups {
        let phase_scores: Vec<f64> = clause_scores.iter()
            .filter(|(id, _, _, _, _)| clause_ids.contains(&id.as_str()))
            .map(|(_, _, _, avg, _)| *avg)
            .collect();
        let phase_avg = if phase_scores.is_empty() { 0.0 } else {
            phase_scores.iter().sum::<f64>() / phase_scores.len() as f64
        };
        let phase_pct = (phase_avg / 5.0) * 100.0;
        let bar = "█".repeat((phase_pct / 5.0) as usize);
        out.push_str(&format!("  {} ({}): {:>5.1}% {}\n", phase, 
            if phase == 'P' { "Plan" } else if phase == 'D' { "Do" } else if phase == 'C' { "Check" } else { "Act" },
            phase_pct, bar));
    }

    // PROPER Prediction
    out.push_str("\n═══ PROPER PREDICTION ═══\n\n");
    out.push_str("Ref: Falakh 2026 (ISO 14001 → PROPER linkage)\n");
    out.push_str("Ref: PermenLHK P.1/2021 (PROPER rating criteria)\n\n");

    let (color, label, desc, requirements) = if overall_pct < 50.0 {
        ("⬛/🟥", "HITAM/MERAH",
         "Belum taat. Pelanggaran serius atau belum memenuhi persyaratan peraturan.",
         "Wajib: Izin lingkungan + pemenuhan 100% compliance minimal")
    } else if overall_pct < 75.0 {
        ("🟥", "MERAH",
         "Upaya pengelolaan LH telah dilakukan tetapi belum sesuai persyaratan.",
         "Tutup gap di clause dengan score <50% terlebih dahulu")
    } else if overall_pct < 90.0 {
        ("🔵", "BIRU",
         "Taats — telah melakukan upaya pengelolaan LH sesuai persyaratan peraturan.",
         "Capai >90% untuk upgrade ke HIJAU")
    } else if overall_pct < 95.0 {
        ("🟢", "HIJAU",
         "Beyond compliance — implementasi SML, efisiensi energi, 3R, CSR.",
         "Capai >95% + circular economy untuk EMAS")
    } else {
        ("🥇", "EMAS",
         "Environmental excellency — produksi bersih, ekonomi sirkular, pengembangan masyarakat.",
         "Pertahankan dengan continual improvement (Clause 10.3)")
    };

    out.push_str(&format!("  Predicted: {} {}\n", color, label));
    out.push_str(&format!("  Status: {}\n", desc));
    out.push_str(&format!("  Requirements: {}\n", requirements));

    // Improvement Roadmap
    out.push_str("\n═══ IMPROVEMENT ROADMAP ═══\n\n");
    let mut sorted_clauses = clause_scores.clone();
    sorted_clauses.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
    out.push_str("Priority (lowest score first):\n");
    for (i, (id, title, _, avg, _)) in sorted_clauses.iter().enumerate() {
        let pct = (avg / 5.0) * 100.0;
        if pct < 90.0 {
            out.push_str(&format!("  {}. Clause {} ({}) — {:.1}%\n", i + 1, id, 
                &title[..title.len().min(28)], pct));
        }
    }

    // ISO 14001:2026 note
    out.push_str("\n═══ ISO 14001:2026 UPDATE ═══\n\n");
    out.push_str("ISO 14001:2026 (published April 2026) — 4th edition.\n");
    out.push_str("Key changes from 2015:\n");
    out.push_str("  - Strengthened climate change adaptation (Clause 6.1)\n");
    out.push_str("  - Enhanced supply chain environmental aspects (Clause 8.1)\n");
    out.push_str("  - Digital documentation requirements (Clause 7.5)\n");
    out.push_str("  - SDG alignment reporting (Clause 9.3)\n");
    out.push_str("  - 3-year transition period from 2015 version\n");

    out
}
