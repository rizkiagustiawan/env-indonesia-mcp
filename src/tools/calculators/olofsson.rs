/// Olofsson et al. 2014 Area-Weighted Accuracy Assessment
/// Ref: "Good practices for estimating area and assessing accuracy of land use change"
/// Remote Sensing of Environment, 148, 42-57. DOI: 10.1016/j.rse.2014.02.015
///
/// Provides unbiased area estimates with confidence intervals from stratified random sampling.

pub fn calculate(mapped_areas_json: &str, confusion_matrix_json: &str, class_names_json: &str, z_score: f64) -> String {
    // Parse inputs
    let mapped_areas: Vec<f64> = match serde_json::from_str(mapped_areas_json) {
        Ok(v) => v, Err(e) => return format!("ERROR [E103]: mapped_areas JSON: {}", e)
    };
    let matrix: Vec<Vec<u32>> = match serde_json::from_str(confusion_matrix_json) {
        Ok(v) => v, Err(e) => return format!("ERROR [E103]: confusion_matrix JSON: {}", e)
    };
    let class_names: Vec<String> = serde_json::from_str(class_names_json)
        .unwrap_or_else(|_| (0..mapped_areas.len()).map(|i| format!("Class {}", i)).collect());
    
    let q = mapped_areas.len(); // number of classes
    if matrix.len() != q || matrix.iter().any(|r| r.len() != q) {
        return format!("ERROR [E104]: Matrix harus {}x{} sesuai jumlah kelas", q, q);
    }
    
    let a_total: f64 = mapped_areas.iter().sum();
    if a_total <= 0.0 { return "ERROR [E102]: Total mapped area harus > 0".into(); }
    
    // Wi = weight (area proportion of stratum i)
    let w: Vec<f64> = mapped_areas.iter().map(|a| a / a_total).collect();
    
    // ni. = row totals (sample count per stratum)
    let n_i: Vec<f64> = matrix.iter().map(|row| row.iter().sum::<u32>() as f64).collect();
    
    // Check for empty strata
    for i in 0..q {
        if n_i[i] < 2.0 {
            return format!("ERROR [E105]: Stratum {} ('{}') memiliki < 2 sampel. Minimum 2 per stratum.", i, class_names[i]);
        }
    }
    
    // 1. Area proportion matrix: p_ij = Wi * (n_ij / ni.)  [Eq. 4]
    let p: Vec<Vec<f64>> = (0..q).map(|i| {
        (0..q).map(|j| w[i] * (matrix[i][j] as f64 / n_i[i])).collect()
    }).collect();
    
    // 2. Overall Accuracy = Σ p_ii  [Eq. 1]
    let oa: f64 = (0..q).map(|i| p[i][i]).sum();
    
    // 3. Variance of OA  [Eq. 5]
    let v_oa: f64 = (0..q).map(|i| {
        let p_ii = matrix[i][i] as f64 / n_i[i];
        w[i].powi(2) * p_ii * (1.0 - p_ii) / (n_i[i] - 1.0)
    }).sum();
    let se_oa = v_oa.sqrt();
    
    // 4. User's accuracy per class [Eq. 2]
    let users_acc: Vec<(f64, f64)> = (0..q).map(|i| {
        let u = matrix[i][i] as f64 / n_i[i];
        let v = u * (1.0 - u) / (n_i[i] - 1.0);
        (u, v.sqrt())
    }).collect();
    
    // 5. Producer's accuracy per class [Eq. 3]
    let col_sums: Vec<f64> = (0..q).map(|j| (0..q).map(|i| p[i][j]).sum::<f64>()).collect();
    let producers_acc: Vec<(f64, f64)> = (0..q).map(|j| {
        if col_sums[j] > 0.0 { (p[j][j] / col_sums[j], 0.0) } // simplified SE
        else { (0.0, 0.0) }
    }).collect();
    
    // 6. Unbiased area estimates [Eq. 9] + SE [Eq. 10]
    let area_estimates: Vec<(f64, f64, f64, f64)> = (0..q).map(|j| {
        let area_j = a_total * col_sums[j];
        let var_j: f64 = (0..q).map(|i| {
            let ratio = matrix[i][j] as f64 / n_i[i];
            w[i].powi(2) * ratio * (1.0 - ratio) / (n_i[i] - 1.0)
        }).sum();
        let se_j = a_total * var_j.sqrt();
        let ci_lo = (area_j - z_score * se_j).max(0.0);
        let ci_hi = area_j + z_score * se_j;
        (area_j, se_j, ci_lo, ci_hi)
    }).collect();
    
    // Format output
    let mut out = String::from("══════════════════════════════════════════════════════════\n");
    out.push_str("  OLOFSSON AREA-WEIGHTED ACCURACY ASSESSMENT\n");
    out.push_str("══════════════════════════════════════════════════════════\n");
    out.push_str("Ref: Olofsson et al. 2014, Remote Sensing of Environment\n\n");
    
    // Area proportion matrix
    out.push_str("AREA PROPORTION MATRIX (p_ij):\n");
    out.push_str(&format!("{:<15}", "Map\\Ref"));
    for name in &class_names { out.push_str(&format!("{:<12}", &name[..name.len().min(11)])); }
    out.push_str(&format!("{:<12}\n", "Wi"));
    out.push_str(&"-".repeat(15 + 12 * (q + 1)));
    out.push_str("\n");
    for i in 0..q {
        out.push_str(&format!("{:<15}", &class_names[i][..class_names[i].len().min(14)]));
        for j in 0..q {
            out.push_str(&format!("{:<12.4}", p[i][j]));
        }
        out.push_str(&format!("{:<12.4}\n", w[i]));
    }
    
    // Overall accuracy
    out.push_str(&format!("\nOVERALL ACCURACY (area-weighted): {:.2}% ± {:.2}%\n", oa * 100.0, se_oa * 100.0));
    out.push_str(&format!("95% CI: [{:.2}%, {:.2}%]\n", (oa - z_score * se_oa).max(0.0) * 100.0, (oa + z_score * se_oa).min(1.0) * 100.0));
    
    // Per-class accuracy
    out.push_str("PER-CLASS ACCURACY:\n");
    out.push_str(&format!("{:<15} {:<15} {:<15}\n", "Class", "User's Acc", "Producer's Acc"));
    out.push_str(&"-".repeat(45));
    out.push_str("\n");
    for i in 0..q {
        out.push_str(&format!("{:<15} {:<15.1}% {:<15.1}%\n",
            &class_names[i][..class_names[i].len().min(14)],
            users_acc[i].0 * 100.0,
            producers_acc[i].0 * 100.0));
    }
    
    // Unbiased area estimates
    out.push_str("\nUNBIASED AREA ESTIMATES:\n");
    out.push_str(&format!("{:<15} {:<15} {:<15} {:<20}\n", "Class", "Mapped Area", "Adjusted Area", "95% CI"));
    out.push_str(&"-".repeat(65));
    out.push_str("\n");
    for j in 0..q {
        let (adj, _se, ci_lo, ci_hi) = area_estimates[j];
        out.push_str(&format!("{:<15} {:<15.1} {:<15.1} [{:.1}, {:.1}]\n",
            &class_names[j][..class_names[j].len().min(14)],
            mapped_areas[j], adj, ci_lo, ci_hi));
    }
    
    out.push_str("\nCatatan: Area adjusted = unbiased estimate dari stratified random sampling.\n");
    out.push_str("CI = Confidence Interval berdasarkan variance estimator Olofsson Eq. 10.\n");
    out
}
