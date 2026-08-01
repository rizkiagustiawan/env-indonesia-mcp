/// Validasi QA/QC Data Lingkungan
/// Ref: US EPA QA/QC Guidance, SNI 6989-series

pub fn validate(data_json: &str) -> String {
    let samples: Vec<serde_json::Value> = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: Gagal parsing JSON: {}", e),
    };

    if samples.is_empty() {
        return "ERROR: Array data QA/QC kosong.".into();
    }

    let mut rows = Vec::new();
    let mut total_samples = 0_usize;
    let mut rpd_pass = 0_usize;
    let mut rpd_fail = 0_usize;
    let mut spike_pass = 0_usize;
    let mut spike_fail = 0_usize;
    let mut blank_pass = 0_usize;
    let mut blank_fail = 0_usize;
    let mut flags: Vec<String> = Vec::new();

    for sample in &samples {
        total_samples += 1;

        let sample_id = sample.get("sample").and_then(|v| v.as_str()).unwrap_or("?");
        let value = sample.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let mut sample_row = format!("Sampel: {} (nilai: {:.4})\n", sample_id, value);

        // RPD check (duplicate)
        if let Some(dup) = sample.get("duplicate").and_then(|v| v.as_f64()) {
            let avg = (value + dup) / 2.0;
            let rpd = if avg > 1e-12 {
                ((value - dup).abs() / avg) * 100.0
            } else {
                0.0
            };

            // Acceptable: <20% water, <30% soil (use 20% as default)
            let rpd_limit = 20.0;
            let rpd_ok = rpd <= rpd_limit;

            if rpd_ok {
                rpd_pass += 1;
            } else {
                rpd_fail += 1;
            }

            sample_row.push_str(&format!(
                "  RPD: |{:.4} - {:.4}| / {:.4} × 100 = {:.1}% {} (batas: {}%)\n",
                value,
                dup,
                avg,
                rpd,
                if rpd_ok { "LULUS ✓" } else { "GAGAL ✗" },
                rpd_limit
            ));

            if !rpd_ok {
                flags.push(format!(
                    "{}: RPD {:.1}% melebihi batas {}%",
                    sample_id, rpd, rpd_limit
                ));
            }
        }

        // Spike recovery
        if let (Some(spike), Some(spike_amt)) = (
            sample.get("spike").and_then(|v| v.as_f64()),
            sample.get("spike_amount").and_then(|v| v.as_f64()),
        ) {
            let recovery = if spike_amt > 1e-12 {
                ((spike - value) / spike_amt) * 100.0
            } else {
                0.0
            };

            let recovery_ok = recovery >= 80.0 && recovery <= 120.0;

            if recovery_ok {
                spike_pass += 1;
            } else {
                spike_fail += 1;
            }

            sample_row.push_str(&format!(
                "  Spike Recovery: ({:.4} - {:.4}) / {:.4} × 100 = {:.1}% {} (batas: 80-120%)\n",
                spike,
                value,
                spike_amt,
                recovery,
                if recovery_ok {
                    "LULUS ✓"
                } else {
                    "GAGAL ✗"
                }
            ));

            if !recovery_ok {
                flags.push(format!(
                    "{}: Recovery {:.1}% di luar 80-120%",
                    sample_id, recovery
                ));
            }
        }

        // Blank check
        if let Some(blank) = sample.get("blank").and_then(|v| v.as_f64()) {
            // MDL typically ~10% of sample value or instrument-specific
            let mdl_estimate = value * 0.1;
            let blank_ok = blank < mdl_estimate.max(0.01); // at least 0.01

            if blank_ok {
                blank_pass += 1;
            } else {
                blank_fail += 1;
            }

            sample_row.push_str(&format!(
                "  Blank: {:.4} {} (MDL estimasi: {:.4})\n",
                blank,
                if blank_ok {
                    "LULUS ✓"
                } else {
                    "GAGAL ✗ — kontaminasi terdeteksi"
                },
                mdl_estimate.max(0.01)
            ));

            if !blank_ok {
                flags.push(format!("{}: Blank {:.4} melebihi MDL", sample_id, blank));
            }
        }

        rows.push(sample_row);
    }

    // Overall assessment
    let total_checks = rpd_pass + rpd_fail + spike_pass + spike_fail + blank_pass + blank_fail;
    let total_pass = rpd_pass + spike_pass + blank_pass;
    let pass_pct = if total_checks > 0 {
        (total_pass as f64 / total_checks as f64) * 100.0
    } else {
        100.0
    };

    let overall = if flags.is_empty() {
        "LULUS — Semua parameter QA/QC memenuhi kriteria"
    } else if pass_pct >= 80.0 {
        "LULUS BERSYARAT — Sebagian besar memenuhi, ada catatan"
    } else {
        "GAGAL — Data tidak memenuhi kriteria QA/QC, perlu pengulangan"
    };

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("VALIDASI QA/QC DATA LINGKUNGAN\n");
    result.push_str("Ref: US EPA QA/QC Guidance, SNI 6989-series\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str(&format!("Jumlah sampel divalidasi: {}\n\n", total_samples));

    result.push_str("KRITERIA:\n");
    result.push_str("• RPD (Relative Percent Difference) : < 20% (air), < 30% (tanah)\n");
    result.push_str("• Spike Recovery                    : 80–120%\n");
    result.push_str("• Blank                             : < MDL (Method Detection Limit)\n\n");

    result.push_str("────────────────────────────────────────\n");
    result.push_str("DETAIL PER SAMPEL:\n");
    result.push_str("────────────────────────────────────────\n");
    for row in &rows {
        result.push_str(row);
        result.push('\n');
    }

    result.push_str("────────────────────────────────────────\n");
    result.push_str("RINGKASAN:\n");
    result.push_str(&format!(
        "• RPD          : {} lulus, {} gagal\n",
        rpd_pass, rpd_fail
    ));
    result.push_str(&format!(
        "• Spike Recov. : {} lulus, {} gagal\n",
        spike_pass, spike_fail
    ));
    result.push_str(&format!(
        "• Blank        : {} lulus, {} gagal\n",
        blank_pass, blank_fail
    ));
    result.push_str(&format!(
        "• Total        : {}/{} lulus ({:.0}%)\n\n",
        total_pass, total_checks, pass_pct
    ));

    if !flags.is_empty() {
        result.push_str("FLAG (CATATAN):\n");
        for flag in &flags {
            result.push_str(&format!("  ⚠ {}\n", flag));
        }
        result.push('\n');
    }

    result.push_str(&format!("PENILAIAN KESELURUHAN: {}\n", overall));
    result.push_str("══════════════════════════════════════════════\n");

    result
}
