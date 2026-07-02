/// Biodiversity Index Calculator
/// Shannon-Wiener H' = -Σ(pi × ln(pi))
/// Simpson D = 1 - Σ(pi²)

pub fn calculate(species_counts: &[u64]) -> String {
    if species_counts.is_empty() { return "ERROR: Masukkan jumlah individu per spesies.".into(); }

    let total: u64 = species_counts.iter().sum();
    if total == 0 { return "ERROR: Total individu = 0.".into(); }

    let n_species = species_counts.len();
    let total_f = total as f64;

    // Shannon-Wiener
    let shannon: f64 = species_counts.iter()
        .filter(|&&n| n > 0)
        .map(|&n| { let p = (n as f64) / total_f; -p * p.ln() })
        .sum();

    // Simpson
    let simpson: f64 = 1.0 - species_counts.iter()
        .map(|&n| { let p = (n as f64) / total_f; p * p })
        .sum::<f64>();

    // Evenness (Pielou)
    let h_max = (n_species as f64).ln();
    let evenness = if h_max > 0.0 { shannon / h_max } else { 0.0 };

    let mut out = String::from("=== Biodiversity Index Calculator ===\n");
    out.push_str("Ref: Shannon & Weaver (1949), Simpson (1949)\n\n");
    out.push_str(&format!("Data:\n  Jumlah spesies (S) = {}\n  Total individu (N) = {}\n\n", n_species, total));

    out.push_str("Distribusi:\n");
    for (i, &n) in species_counts.iter().enumerate() {
        let p = (n as f64) / total_f;
        out.push_str(&format!("  Spesies {}: {} individu ({:.1}%)\n", i + 1, n, p * 100.0));
    }

    out.push_str(&format!("\nIndeks:\n  Shannon-Wiener (H') = {:.4}\n  Simpson (1-D) = {:.4}\n  Evenness (E) = {:.4}\n\n", shannon, simpson, evenness));

    let kat_h = if shannon < 1.0 { "Rendah (ekosistem tertekan)" } else if shannon < 2.0 { "Sedang" } else if shannon < 3.0 { "Tinggi (ekosistem stabil)" } else { "Sangat Tinggi (ekosistem pristine)" };
    out.push_str(&format!("Kategori keanekaragaman: {}\n", kat_h));
    out
}
