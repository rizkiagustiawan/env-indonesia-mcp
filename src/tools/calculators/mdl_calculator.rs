/// MDL/LOQ Calculator
/// Ref: EPA 40 CFR 136; Standard Methods 23rd Ed
pub fn assess(replicate_concs_json: &str, spike_level_mg_l: f64) -> String {
    let mut out = String::from("=== MDL/LOQ Calculator ===\n");
    out.push_str("Ref: EPA 40 CFR 136; Standard Methods 23rd Ed\n\n");
    let concs: Vec<f64> = match serde_json::from_str(replicate_concs_json) {
        Ok(v) => v, Err(e) => return format!("ERROR: {}. Format: [c1,c2,...]", e),
    };
    if concs.len() < 7 { return "ERROR: Need >= 7 replicates for MDL.".into(); }
    let n = concs.len();
    let mean: f64 = concs.iter().sum::<f64>() / n as f64;
    let sd: f64 = ((concs.iter().map(|x| (x-mean).powi(2)).sum::<f64>()) / (n-1) as f64).sqrt();
    // t-value for n-1 df, 99% one-sided (approximate)
    let t_val = match n { 7 => 3.143, 8 => 2.998, 9 => 2.896, 10 => 2.821, 11 => 2.764, _ => 2.624 };
    let mdl = t_val * sd;
    let loq = 10.0 * sd;
    let pql = 5.0 * sd;
    let recovery = mean / spike_level_mg_l * 100.0;
    out.push_str(&format!("Replicates: {}, Spike: {:.3} mg/L\n\n", n, spike_level_mg_l));
    out.push_str(&format!("  Mean: {:.4}, SD: {:.4}\n", mean, sd));
    out.push_str(&format!("  Recovery: {:.1}%\n\n", recovery));
    out.push_str(&format!("  >> MDL = t*SD = {:.4}*{:.4} = {:.4} mg/L\n", t_val, sd, mdl));
    out.push_str(&format!("  >> LOQ = 10*SD = {:.4} mg/L\n", loq));
    out.push_str(&format!("  >> PQL = 5*SD = {:.4} mg/L\n\n", pql));
    if recovery < 70.0 || recovery > 130.0 { out.push_str("  [WARN] Recovery outside 70-130%\n"); }
    else { out.push_str("  [OK] Recovery acceptable\n"); }
    out.push_str("\n  Ref: EPA 40 CFR 136 App. B\n");
    out
}
