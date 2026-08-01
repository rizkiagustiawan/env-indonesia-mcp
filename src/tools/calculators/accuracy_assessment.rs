use std::collections::{BTreeMap, BTreeSet};

/// Classification Accuracy Assessment
/// Ref: SNI 8202:2015, Congalton & Green (2009)
pub fn calculate(predicted_json: &str, actual_json: &str) -> String {
    // Parse JSON arrays
    let predicted: Vec<String> = match serde_json::from_str(predicted_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR parsing predicted JSON: {}", e),
    };
    let actual: Vec<String> = match serde_json::from_str(actual_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR parsing actual JSON: {}", e),
    };

    if predicted.len() != actual.len() {
        return format!(
            "ERROR: predicted ({}) and actual ({}) arrays must have same length",
            predicted.len(),
            actual.len()
        );
    }

    let n = predicted.len();
    if n == 0 {
        return "ERROR: empty arrays".to_string();
    }

    // Collect all unique classes (sorted)
    let mut classes = BTreeSet::new();
    for p in &predicted {
        classes.insert(p.clone());
    }
    for a in &actual {
        classes.insert(a.clone());
    }
    let class_list: Vec<String> = classes.into_iter().collect();
    let nc = class_list.len();

    // Build class index map
    let class_idx: BTreeMap<&str, usize> = class_list
        .iter()
        .enumerate()
        .map(|(i, c)| (c.as_str(), i))
        .collect();

    // Build confusion matrix
    let mut matrix = vec![vec![0u64; nc]; nc];
    for i in 0..n {
        let row = class_idx[actual[i].as_str()]; // actual = row
        let col = class_idx[predicted[i].as_str()]; // predicted = col
        matrix[row][col] += 1;
    }

    // Row totals (actual totals), column totals (predicted totals)
    let row_totals: Vec<u64> = matrix.iter().map(|r| r.iter().sum()).collect();
    let col_totals: Vec<u64> = (0..nc).map(|j| matrix.iter().map(|r| r[j]).sum()).collect();
    let total: u64 = row_totals.iter().sum();

    // Diagonal sum (correct classifications)
    let diagonal: u64 = (0..nc).map(|i| matrix[i][i]).sum();

    // Overall Accuracy
    let oa = diagonal as f64 / total as f64;

    // Cohen's Kappa
    let pe: f64 = (0..nc)
        .map(|i| row_totals[i] as f64 * col_totals[i] as f64)
        .sum::<f64>()
        / (total as f64 * total as f64);
    let kappa = (oa - pe) / (1.0 - pe);

    // Producer's and User's accuracy per class
    let producer_acc: Vec<f64> = (0..nc)
        .map(|i| {
            if row_totals[i] > 0 {
                matrix[i][i] as f64 / row_totals[i] as f64
            } else {
                0.0
            }
        })
        .collect();
    let user_acc: Vec<f64> = (0..nc)
        .map(|j| {
            if col_totals[j] > 0 {
                matrix[j][j] as f64 / col_totals[j] as f64
            } else {
                0.0
            }
        })
        .collect();

    // Format output
    let mut out = String::from("=== Classification Accuracy Assessment ===\n");
    out.push_str("Ref: SNI 8202:2015, Congalton & Green (2009)\n\n");

    // Confusion Matrix header
    out.push_str("--- Confusion Matrix ---\n");
    out.push_str(&format!("{:<15}", "Actual\\Pred"));
    for c in &class_list {
        out.push_str(&format!("{:>10}", truncate_class(c, 10)));
    }
    out.push_str(&format!("{:>10}\n", "Total"));

    // Matrix rows
    for i in 0..nc {
        out.push_str(&format!("{:<15}", truncate_class(&class_list[i], 14)));
        for j in 0..nc {
            out.push_str(&format!("{:>10}", matrix[i][j]));
        }
        out.push_str(&format!("{:>10}\n", row_totals[i]));
    }

    // Column totals
    out.push_str(&format!("{:<15}", "Total"));
    for j in 0..nc {
        out.push_str(&format!("{:>10}", col_totals[j]));
    }
    out.push_str(&format!("{:>10}\n\n", total));

    // Per-class accuracy
    out.push_str("--- Per-Class Accuracy ---\n");
    out.push_str(&format!(
        "{:<15}{:>15}{:>15}\n",
        "Class", "Producer Acc", "User Acc"
    ));
    for i in 0..nc {
        out.push_str(&format!(
            "{:<15}{:>14.1}%{:>14.1}%\n",
            truncate_class(&class_list[i], 14),
            producer_acc[i] * 100.0,
            user_acc[i] * 100.0
        ));
    }

    // Summary
    out.push_str(&format!("\n--- Summary Statistics ---\n"));
    out.push_str(&format!("Total Samples     : {}\n", total));
    out.push_str(&format!(
        "Overall Accuracy  : {:.2}% ({}/{})\n",
        oa * 100.0,
        diagonal,
        total
    ));
    out.push_str(&format!("Kappa Coefficient : {:.4}\n\n", kappa));

    // SNI 8202:2015 compliance
    out.push_str("--- SNI 8202:2015 Compliance ---\n");
    let oa_pass = oa >= 0.85;
    let kappa_pass = kappa >= 0.75;
    out.push_str(&format!(
        "Overall Accuracy >= 85%: {} ({})\n",
        if oa_pass { "LULUS" } else { "TIDAK LULUS" },
        format!("{:.1}%", oa * 100.0)
    ));
    out.push_str(&format!(
        "Kappa >= 0.75          : {} ({})\n",
        if kappa_pass { "LULUS" } else { "TIDAK LULUS" },
        format!("{:.4}", kappa)
    ));

    if oa_pass && kappa_pass {
        out.push_str("\nStatus: MEMENUHI standar SNI 8202:2015\n");
    } else {
        out.push_str("\nStatus: BELUM MEMENUHI standar SNI 8202:2015\n");
        out.push_str("Saran: Perbaiki training data, tambah sampel kelas dengan akurasi rendah.\n");
    }

    out
}

fn truncate_class(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}.", &s[..max - 1])
    } else {
        s.to_string()
    }
}
