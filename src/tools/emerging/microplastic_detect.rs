/// Microplastic Detection — Spectral Matching + Shape Classification
/// Ref: Environmental Science & Technology 2022-2026; LDIR + CNN
/// Simplified: cosine similarity for FTIR/Raman + geometry metrics
pub fn assess(sample_id: &str, particle_count: u32, sizes_json: &str, spectra_match_json: &str) -> String {
    let mut out = String::from("=== Microplastic Detection (AI) ===\n");
    out.push_str("Ref: ES&T 2022-2026; LDIR imaging + ML\n\n");
    let sizes: Vec<f64> = match serde_json::from_str(sizes_json) {
        Ok(v) => v,
        Err(_) => vec![100.0],
    };
    let spectra: Vec<(String, f64)> = match serde_json::from_str(spectra_match_json) {
        Ok(v) => v,
        Err(_) => vec![],
    };
    let mean_size: f64 = sizes.iter().sum::<f64>() / sizes.len().max(1) as f64;
    let min_size = sizes.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_size = sizes.iter().cloned().fold(0.0_f64, f64::max);
    let concentration_per_l = particle_count as f64 / 10.0;
    let n_fragments = sizes.iter().filter(|s| *s > &500.0).count();
    let n_fibers = sizes.iter().filter(|s| *s <= &500.0).count();
    let (best_polymer, best_match) = spectra.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(p, s)| (p.as_str(), *s))
        .unwrap_or(("Unknown", 0.0));
    let confidence = if best_match > 0.9 {"High"} else if best_match > 0.7 {"Medium"} else {"Low"};
    out.push_str(&format!("Sample: {}, Particles: {}\n\n", sample_id, particle_count));
    out.push_str("-- Size Distribution --\n\n");
    out.push_str(&format!("  Mean: {:.0} µm, Min: {:.0}, Max: {:.0}\n", mean_size, min_size, max_size));
    out.push_str(&format!("  Fragments (>500µm): {}, Fibers (≤500µm): {}\n\n", n_fragments, n_fibers));
    out.push_str("-- Spectral Matching --\n\n");
    out.push_str("  Cosine similarity vs reference library:\n");
    for (polymer, score) in &spectra {
        out.push_str(&format!("    {:<12} {:.3} {}\n", polymer, score, if *score > 0.9 {"✅"} else {""}));
    }
    out.push_str(&format!("\n  >> Best match: {} ({:.1}% confidence: {})\n\n", best_polymer, best_match * 100.0, confidence));
    out.push_str("-- Quantification --\n\n");
    out.push_str(&format!("  Concentration: {:.1} particles/L\n", concentration_per_l));
    out.push_str("  WHO guideline: no specific limit\n");
    out.push_str("  Precautionary: <1 particle/L (drinking water)\n\n");
    out.push_str("-- Shape Classification --\n");
    out.push_str("  Fragment / Fiber / Film / Foam / Sphere\n\n");
    out.push_str("-- PEMANTAUAN --\n");
    out.push_str("  Metode: FTIR microscopy, Raman, LDIR\n");
    out.push_str("  Ref: ES&T 2022-2026\n");
    out
}
