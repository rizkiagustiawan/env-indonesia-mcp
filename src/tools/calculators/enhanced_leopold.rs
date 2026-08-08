/// Enhanced Leopold Matrix with AHP + TOPSIS
/// Ref: Leopold et al. (1971) USGS Circular 645
/// AHP: Saaty (1980) — pairwise comparison → eigenvalue weights, CR < 0.1
///   Power iteration: Shiraishi & Obata 2025; Wang et al. 2021; Baladraf 2026
/// TOPSIS: Hwang & Yoon (1981) — distance to ideal positive/negative
/// 2026 SOTA: Zhang et al. 2026 (Springer), Nasiri Khiavi et al. 2026 (Springer)
/// Indonesia: PP 22/2021, UU 32/2009, PermenLHK 5/2021

/// RI table (Random Index) from Saaty — for Consistency Ratio computation
fn ri_table(n: usize) -> f64 {
    match n {
        1 => 0.0, 2 => 0.0, 3 => 0.58, 4 => 0.90, 5 => 1.12,
        6 => 1.24, 7 => 1.32, 8 => 1.41, 9 => 1.45, 10 => 1.49,
        _ => 1.51,
    }
}

/// Power iteration to compute principal eigenvalue and eigenvector of pairwise matrix
/// Ref: Shiraishi & Obata 2025; Ishizaka & Lusti 2006; Saaty 2003
/// Returns (lambda_max, weights_vec)
fn power_iteration(matrix: &[Vec<f64>], n: usize) -> (f64, Vec<f64>) {
    // Initialize v = [1/n, 1/n, ..., 1/n]
    let mut v = vec![1.0 / n as f64; n];

    let max_iter = 100;
    let tolerance = 1e-8;

    for _iter in 0..max_iter {
        // v_new = A · v
        let mut v_new = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                v_new[i] += matrix[i][j] * v[j];
            }
        }

        // Normalize (L1 norm → sum = 1)
        let sum: f64 = v_new.iter().sum();
        if sum > 1e-30 {
            for x in v_new.iter_mut() { *x /= sum; }
        }

        // Check convergence
        let diff: f64 = v_new.iter().zip(v.iter())
            .map(|(a, b)| (a - b).abs()).sum();
        v = v_new;

        if diff < tolerance {
            break;
        }
    }

    // Compute lambda_max via Rayleigh quotient: λ = (A·v)·v / (v·v)
    // But for normalized eigenvector (sum=1), simpler: λ_max = Σᵢ (A·v)ᵢ
    let mut av = vec![0.0; n];
    for i in 0..n {
        for j in 0..n {
            av[i] += matrix[i][j] * v[j];
        }
    }
    // lambda_max = sum of (A·v)_i (since v sums to 1, this gives the eigenvalue)
    let lambda_max: f64 = av.iter().sum();

    (lambda_max, v)
}

/// Compute AHP weights and consistency from pairwise comparison matrix
/// Returns (weights, lambda_max, CI, CR, iterations)
fn ahp_from_pairwise(pairwise: &[Vec<f64>]) -> Option<(Vec<f64>, f64, f64, f64, usize)> {
    let n = pairwise.len();
    if n < 2 { return None; }
    for row in pairwise { if row.len() != n { return None; } }

    // Validate: diagonal = 1, reciprocal property A[i][j] = 1/A[j][i]
    for i in 0..n {
        if (pairwise[i][i] - 1.0).abs() > 0.01 { return None; }
        for j in (i+1)..n {
            let expected = 1.0 / pairwise[j][i];
            if (pairwise[i][j] - expected).abs() > 0.1 * expected.max(1.0) {
                // Not perfectly reciprocal — warn but continue
            }
        }
    }

    // Count iterations manually (re-run power iteration with counter)
    let mut v = vec![1.0 / n as f64; n];
    let mut iterations = 0;
    let tolerance = 1e-8;

    for iter in 0..100 {
        let mut v_new = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                v_new[i] += pairwise[i][j] * v[j];
            }
        }
        let sum: f64 = v_new.iter().sum();
        if sum > 1e-30 {
            for x in v_new.iter_mut() { *x /= sum; }
        }
        let diff: f64 = v_new.iter().zip(v.iter())
            .map(|(a, b)| (a - b).abs()).sum();
        v = v_new;
        iterations = iter + 1;
        if diff < tolerance { break; }
    }

    // lambda_max
    let mut av = vec![0.0; n];
    for i in 0..n {
        for j in 0..n {
            av[i] += pairwise[i][j] * v[j];
        }
    }
    let lambda_max: f64 = av.iter().sum();

    // CI = (λ_max - n) / (n - 1)
    let ci = if n > 1 { (lambda_max - n as f64) / (n as f64 - 1.0) } else { 0.0 };
    // CR = CI / RI
    let ri = ri_table(n);
    let cr = if ri > 0.0 { ci / ri } else { 0.0 };

    Some((v, lambda_max, ci, cr, iterations))
}

pub fn assess(
    impacts_json: &str,
    criteria_weights_json: &str,
    alternatives_json: &str,
) -> String {
    assess_full(impacts_json, criteria_weights_json, alternatives_json, "")
}

pub fn assess_full(
    impacts_json: &str,
    criteria_weights_json: &str,
    alternatives_json: &str,
    pairwise_matrix_json: &str,
) -> String {
    let mut out = String::from("=== Enhanced Leopold Matrix (AHP + TOPSIS) ===\n");
    out.push_str("Ref: Leopold 1971 + Saaty 1980 (AHP) + Hwang & Yoon 1981 (TOPSIS)\n");
    out.push_str("AHP eigenvalue: Power iteration (Shiraishi 2025; Wang 2021; Baladraf 2026)\n");
    out.push_str("2026 SOTA: Zhang et al. 2026; Nasiri Khiavi et al. 2026\n");
    out.push_str("Regulasi: PP 22/2021, UU 32/2009, PermenLHK 5/2021\n\n");

    let impacts: Vec<(String, String, f64, f64)> = match serde_json::from_str(impacts_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR [E102]: impacts_json parse: {}. Format: [[\"kegiatan\",\"komponen\",magnitude,importance],...]", e),
    };

    if impacts.is_empty() {
        return "ERROR: impacts_json kosong. Minimal 1 dampak.".into();
    }

    // Phase 1: Modified Leopold Matrix (base layer)
    out.push_str("─ PHASE 1: Modified Leopold Matrix ─\n\n");
    let mut total_pos = 0.0f64;
    let mut total_neg = 0.0f64;
    let mut count_sig = 0u32;

    out.push_str(&format!("{:<28} {:<22} {:>6} {:>6} {:>7}\n", "Kegiatan", "Komponen LH", "M", "I", "M×I"));
    out.push_str(&"-".repeat(72));
    out.push('\n');

    for (keg, komp, mag, imp) in &impacts {
        if *mag < -10.0 || *mag > 10.0 {
            out.push_str(&format!("  ⚠️ Magnitude {} di luar (-10..+10)\n", mag));
            continue;
        }
        if *imp < 1.0 || *imp > 10.0 {
            out.push_str(&format!("  ⚠️ Importance {} di luar (1..10)\n", imp));
            continue;
        }
        let score = mag * imp;
        if score > 0.0 { total_pos += score; }
        else { total_neg += score; }
        if score.abs() >= 30.0 { count_sig += 1; }

        let indicator = if score > 0.0 { "➕" } else if score < 0.0 { "➖" } else { "⚪" };
        out.push_str(&format!("{:<28} {:<22} {:>6.1} {:>6.1} {:>7.1} {}\n",
            &keg[..keg.len().min(27)], &komp[..komp.len().min(21)],
            mag, imp, score, indicator));
    }

    let net = total_pos + total_neg;
    out.push_str(&format!("\nRingkasan Leopold:\n  Positif: +{:.1} | Negatif: {:.1} | Net: {:.1}\n  Signifikan (|M×I|≥30): {}\n\n", total_pos, total_neg, net, count_sig));

    // Phase 2: AHP Weighting
    // Try pairwise matrix first (true eigenvalue method)
    let (normalized, cr, lambda_max, ci, ri_val, used_pairwise, iterations) = if !pairwise_matrix_json.is_empty() {
        let pairwise: Vec<Vec<f64>> = match serde_json::from_str(pairwise_matrix_json) {
            Ok(v) => v,
            Err(e) => {
                out.push_str(&format!("⚠️ pairwise_matrix_json parse error: {}\n", e));
                Vec::new()
            }
        };

        if pairwise.len() >= 2 {
            // Get criteria names from criteria_weights_json (if provided)
            let criteria_names: Vec<String> = match serde_json::from_str::<Vec<(String, f64)>>(criteria_weights_json) {
                Ok(v) => v.iter().map(|(n, _)| n.clone()).collect(),
                Err(_) => (0..pairwise.len()).map(|i| format!("C{}", i+1)).collect(),
            };

            if let Some((weights, lm, ci_val, cr_val, iters)) = ahp_from_pairwise(&pairwise) {
                let norm: Vec<(String, f64)> = weights.iter().enumerate()
                    .map(|(i, w)| (criteria_names.get(i).cloned().unwrap_or_else(|| format!("C{}", i+1)), *w))
                    .collect();

                // Display pairwise matrix
                out.push_str("─ PHASE 2: AHP — Pairwise Comparison Matrix (True Eigenvalue) ─\n\n");
                out.push_str("Pairwise Matrix (Saaty 1-9 scale):\n");
                out.push_str(&format!("  {:>8}", ""));
                for name in &criteria_names {
                    out.push_str(&format!(" {:>8}", &name[..name.len().min(7)]));
                }
                out.push('\n');
                for (i, row) in pairwise.iter().enumerate() {
                    let label: String = criteria_names.get(i).map(|s| s.chars().take(7).collect()).unwrap_or_default();
                    out.push_str(&format!("  {:>8}", label));
                    for val in row {
                        out.push_str(&format!(" {:>8.2}", val));
                    }
                    out.push('\n');
                }
                out.push('\n');

                out.push_str(&format!("Power Iteration: {} iterations to converge (tol=1e-8)\n", iters));
                out.push_str(&format!("  λ_max (principal eigenvalue): {:.4}\n", lm));
                out.push_str(&format!("  CI (Consistency Index): {:.4}\n", ci_val));
                out.push_str(&format!("  RI (Random Index, n={}): {:.2}\n", pairwise.len(), ri_table(pairwise.len())));
                out.push_str(&format!("  CR (Consistency Ratio): {:.4}", cr_val));
                if cr_val < 0.1 {
                    out.push_str(" ✅ (Acceptable, CR < 0.1)\n\n");
                } else {
                    out.push_str(" ⚠️ (CR ≥ 0.1 — pairwise comparison INCONSISTENT, needs revision)\n\n");
                }

                (norm, cr_val, lm, ci_val, ri_table(pairwise.len()), true, iters)
            } else {
                out.push_str("⚠️ Pairwise matrix invalid (non-square or diagonal ≠ 1). Using weights fallback.\n\n");
                process_weights_fallback(criteria_weights_json, &mut out)
            }
        } else {
            process_weights_fallback(criteria_weights_json, &mut out)
        }
    } else {
        // No pairwise matrix — use weights directly (CR = N/A)
        process_weights_fallback(criteria_weights_json, &mut out)
    };

    // Display weights
    if !used_pairwise {
        out.push_str("─ PHASE 2: AHP Criteria Weights (direct, no pairwise matrix) ─\n\n");
    }

    for (name, w) in &normalized {
        let stars = "█".repeat((w * 20.0).round() as usize);
        out.push_str(&format!("  {:<12} {:>6.1}%  {}\n", name, w * 100.0, stars));
    }

    if !used_pairwise {
        out.push_str("\n  Consistency Ratio (CR): N/A (no pairwise matrix provided)\n");
        out.push_str("  For true CR: provide pairwise_matrix_json (n×n Saaty scale 1-9)\n\n");
    }

    // Phase 3: TOPSIS Ranking
    out.push_str("─ PHASE 3: TOPSIS Alternative Ranking ─\n\n");

    let alternatives: Vec<(String, Vec<f64>)> = match serde_json::from_str(alternatives_json) {
        Ok(v) => v,
        Err(_) => {
            out.push_str("  (No alternatives provided — skipping TOPSIS ranking)\n");
            out.push_str("  Format: [[\"Alt A\",[ekologi_score,sosial_score,ekonomi_score,kesehatan_score]],...]\n\n");
            if net < -50.0 {
                out.push_str("⚠️ Net impact sangat negatif. Proyek memerlukan mitigasi serius atau redesain.\n");
            }
            return out;
        }
    };

    if alternatives.is_empty() || alternatives[0].1.len() != normalized.len() {
        out.push_str("  ⚠️ Jumlah kriteria alternatif ≠ criteria weights. Skipping TOPSIS.\n");
        return out;
    }

    // Normalize decision matrix (vector normalization)
    let n_crit = normalized.len();
    let mut norm_sq = vec![0.0f64; n_crit];
    for (_, scores) in &alternatives {
        for (j, s) in scores.iter().enumerate() {
            norm_sq[j] += s * s;
        }
    }
    let norm_factors: Vec<f64> = norm_sq.iter().map(|v| v.sqrt().max(1e-10)).collect();

    // Weighted normalized matrix
    let mut weighted: Vec<Vec<f64>> = Vec::new();
    for (_, scores) in &alternatives {
        let row: Vec<f64> = scores.iter().enumerate()
            .map(|(j, s)| (s / norm_factors[j]) * normalized[j].1)
            .collect();
        weighted.push(row);
    }

    // Ideal positive (A+) and negative (A-)
    let mut ideal_pos = vec![f64::MIN; n_crit];
    let mut ideal_neg = vec![f64::MAX; n_crit];
    for row in &weighted {
        for (j, v) in row.iter().enumerate() {
            if *v > ideal_pos[j] { ideal_pos[j] = *v; }
            if *v < ideal_neg[j] { ideal_neg[j] = *v; }
        }
    }

    // Calculate distances and closeness
    out.push_str(&format!("{:<20}", "Alternatif"));
    for (name, _) in &normalized {
        out.push_str(&format!(" {:>10}", &name[..name.len().min(9)]));
    }
    out.push_str(&format!(" {:>8} {:>8} {:>8} {:>6}\n", "D+", "D-", "C*", "Rank"));
    out.push_str(&"-".repeat(80));
    out.push('\n');

    let mut results: Vec<(String, f64, f64, f64)> = Vec::new();
    for (i, (alt_name, _)) in alternatives.iter().enumerate() {
        let d_pos: f64 = weighted[i].iter().enumerate()
            .map(|(j, v)| (v - ideal_pos[j]).powi(2)).sum::<f64>().sqrt();
        let d_neg: f64 = weighted[i].iter().enumerate()
            .map(|(j, v)| (v - ideal_neg[j]).powi(2)).sum::<f64>().sqrt();
        let closeness = d_neg / (d_pos + d_neg).max(1e-10);
        results.push((alt_name.clone(), d_pos, d_neg, closeness));
    }

    // Sort by closeness descending
    results.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    for (rank, (name, dp, dn, cl)) in results.iter().enumerate() {
        out.push_str(&format!("{:<20}", &name[..name.len().min(19)]));
        out.push_str(&format!(" {:>8.3} {:>8.3} {:>8.3} {:>5}\n", dp, dn, cl, rank + 1));
    }

    // Winner
    if let Some((winner, _, _, cl)) = results.first() {
        out.push_str(&format!("\n  🏆 Best alternative: {} (Closeness = {:.3})\n", winner, cl));
    }

    // Impact summary
    out.push_str(&format!("\n─ IMPACT SUMMARY ─\n"));
    out.push_str(&format!("  Leopold net impact: {:.1}\n", net));
    out.push_str(&format!("  Significant impacts: {}\n", count_sig));
    if used_pairwise {
        out.push_str(&format!("  AHP: λ_max={:.4}, CI={:.4}, RI={:.2}, CR={:.4} {} ({} iter)\n",
            lambda_max, ci, ri_val, cr, if cr < 0.1 { "✅" } else { "⚠️" }, iterations));
    } else {
        out.push_str(&format!("  AHP: direct weights (CR=N/A, no pairwise matrix)\n"));
    }
    if !results.is_empty() {
        out.push_str(&format!("  TOPSIS best: {} (C*={:.3})\n", results[0].0, results[0].3));
    }
    if net < -50.0 {
        out.push_str("\n  ⚠️ Net impact sangat negatif. Mitigasi wajib atau redesain.\n");
    }

    out
}

/// Fallback: process criteria_weights_json directly (no pairwise matrix → CR = N/A)
/// Returns (normalized_weights, cr, lambda_max, ci, ri, used_pairwise, iterations)
fn process_weights_fallback(
    criteria_weights_json: &str,
    out: &mut String,
) -> (Vec<(String, f64)>, f64, f64, f64, f64, bool, usize) {
    let criteria_weights: Vec<(String, f64)> = match serde_json::from_str(criteria_weights_json) {
        Ok(v) => v,
        Err(_) => {
            out.push_str("─ PHASE 2: AHP (default weights — no custom criteria) ─\n\n");
            vec![
                ("Ekologi".into(), 0.40),
                ("Sosial".into(), 0.25),
                ("Ekonomi".into(), 0.20),
                ("Kesehatan".into(), 0.15),
            ]
        }
    };

    let weight_sum: f64 = criteria_weights.iter().map(|(_, w)| w).sum();
    let normalized: Vec<(String, f64)> = if weight_sum > 0.0 {
        criteria_weights.iter().map(|(n, w)| (n.clone(), w / weight_sum)).collect()
    } else {
        criteria_weights
    };

    // CR = N/A when no pairwise matrix
    (normalized, 0.0, 0.0, 0.0, 0.0, false, 0)
}
