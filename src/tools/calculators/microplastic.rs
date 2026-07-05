/// Microplastic Risk Scoring
/// Ref: Emerging contaminant classification

pub fn score(water_type: &str, mp_particles_per_liter: f64) -> String {
    if mp_particles_per_liter < 0.0 { return "ERROR [E102]: Parameter tidak boleh negatif.".into(); }

    let risk = if mp_particles_per_liter < 1.0 { "Rendah" } else if mp_particles_per_liter < 10.0 { "Sedang" } else if mp_particles_per_liter < 100.0 { "Tinggi" } else { "Sangat Tinggi" };

    let mut out = format!("=== Microplastic Risk Scoring ===\n\nTipe air: {}\nKonsentrasi: {:.1} partikel/L\nRisiko: {}\n\n", water_type, mp_particles_per_liter, risk);
    out.push_str("⚠️ Belum ada baku mutu resmi Indonesia untuk mikroplastik.\n");
    out.push_str("Referensi WHO (2019): < 1 partikel/L dianggap aman untuk air minum.\n");
    out
}
