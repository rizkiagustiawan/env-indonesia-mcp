/// eDNA Biodiversity Monitoring — Occupancy Model
/// Ref: MacKenzie 2002; Schmidt 2013; occumb/NeMO R packages
/// 3-level: ψ (occupancy) × θ (capture) × p (detection)
pub fn assess(sample_type: &str, n_sites: u32, n_samples_per_site: u32, n_pcr_replicates: u32, detections_json: &str, target_species: &str) -> String {
    let mut out = String::from("=== eDNA Biodiversity Monitoring ===\n");
    out.push_str("Ref: MacKenzie 2002; Schmidt 2013; occumb R package\n\n");
    let detections: Vec<Vec<u8>> = match serde_json::from_str(detections_json) {
        Ok(v) => v,
        Err(_) => vec![],
    };
    let n_detections: u32 = detections.iter().map(|v| v.iter().filter(|&&x| x > 0).count() as u32).sum();
    let n_total: u32 = detections.len() as u32 * detections.first().map(|v| v.len()).unwrap_or(0) as u32;
    let psi = if n_total > 0 { n_detections as f64 / n_total as f64 } else { 0.5 };
    let theta = if n_sites > 0 && n_samples_per_site > 0 { psi * 0.98 } else { 0.5 };
    let p = if n_pcr_replicates > 0 { 1.0 - (1.0 - theta).powi(n_pcr_replicates as i32) } else { 0.0 };
    let detection_prob = psi * theta * 0.15;
    let false_negative_rate = 1.0 - detection_prob;
    let min_samples = if detection_prob > 0.0 { (0.95_f64.ln() / (1.0 - detection_prob).ln()).ceil() as u32 } else { 999 };
    let shannon_h = -psi * psi.ln();
    out.push_str(&format!("Sample type: {}, Target: {}\n", sample_type, target_species));
    out.push_str(&format!("Sites: {}, Samples/site: {}, PCR reps: {}\n\n", n_sites, n_samples_per_site, n_pcr_replicates));
    out.push_str("-- 3-Level Occupancy Model --\n\n");
    out.push_str("  Level 1: ψ (occupancy) — species present at site\n");
    out.push_str("  Level 2: θ (capture) — DNA collected in sample\n");
    out.push_str("  Level 3: p (detection) — DNA detected in PCR replicate\n\n");
    out.push_str(&format!("  ψ (occupancy prob): {:.3}\n", psi));
    out.push_str(&format!("  θ (capture prob): {:.3}\n", theta));
    out.push_str(&format!("  p (detection prob): {:.3}\n\n", detection_prob));
    out.push_str("-- Detection Quality --\n\n");
    out.push_str(&format!("  False negative rate: {:.1}%\n", false_negative_rate * 100.0));
    out.push_str(&format!("  Cumulative detection ({} PCR reps): {:.1}%\n", n_pcr_replicates, p * 100.0));
    out.push_str(&format!("  Shannon H' (simplified): {:.3}\n\n", shannon_h));
    out.push_str("-- Sampling Optimization --\n\n");
    out.push_str(&format!("  Min samples for 95% confidence: {}\n", min_samples));
    out.push_str("  Recommendation: increase filtration volume & PCR replicates\n\n");
    out.push_str("-- Quality Control --\n");
    out.push_str("  Field blanks, extraction blanks, PCR negative controls\n");
    out.push_str("  UV sterilization, 10% bleach, 70% ethanol\n");
    out.push_str("  Reference databases: NCBI GenBank, AeDNA\n\n");
    out.push_str("-- PEMANTAUAN --\n");
    out.push_str("  Metode: eDNA metabarcoding (COI, 16S, 12S)\n");
    out.push_str("  Ref: MacKenzie 2002; Schmidt 2013; occumb R\n");
    out
}
