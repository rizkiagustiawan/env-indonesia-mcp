/// Spatial Interpolation: IDW (Inverse Distance Weighting)
/// Pure Rust implementation

pub fn idw(points: &[(f64, f64, f64)], target_x: f64, target_y: f64, power: f64) -> String {
    if points.is_empty() {
        return "ERROR: Minimal 1 titik data.".into();
    }

    let mut weight_sum = 0.0_f64;
    let mut value_sum = 0.0_f64;

    for &(x, y, val) in points {
        let dist = ((target_x - x).powi(2) + (target_y - y).powi(2)).sqrt();
        if dist < 1e-10 {
            return format!(
                "=== IDW Interpolation ===\nTitik target tepat pada data point.\nNilai = {:.4}\n",
                val
            );
        }
        let w = 1.0 / dist.powf(power);
        weight_sum += w;
        value_sum += w * val;
    }

    let result = value_sum / weight_sum;

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  IDW Spatial Interpolation\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str(&format!("Target: ({:.6}, {:.6})\n", target_x, target_y));
    out.push_str(&format!("Power: {:.1}\n", power));
    out.push_str(&format!("Data points: {}\n\n", points.len()));
    for (i, &(x, y, v)) in points.iter().enumerate() {
        let d = ((target_x - x).powi(2) + (target_y - y).powi(2)).sqrt();
        out.push_str(&format!(
            "  P{}: ({:.4}, {:.4}) val={:.2} dist={:.4}\n",
            i + 1,
            x,
            y,
            v,
            d
        ));
    }
    out.push_str(&format!("\nHasil interpolasi: {:.4}\n", result));
    out
}
