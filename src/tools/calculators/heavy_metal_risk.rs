/// Heavy Metal Risk Assessment
/// HPI (Heavy metal Pollution Index) + US EPA RAGS health risk
/// Ref: Mohsen & Bhatt 1989 (HPI); US EPA RAGS (health risk)
/// Baku mutu: PP 22/2021

pub fn assess(
    pb: f64, cd: f64, hg: f64, as_: f64, cr: f64,
    body_weight_kg: f64,
    intake_l_per_day: f64,
    exposure_years: f64,
) -> String {
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("Heavy Metal Risk Assessment\n");
    out.push_str("Ref: Mohsen & Bhatt 1989 (HPI); US EPA RAGS\n");
    out.push_str("Baku mutu: PP 22/2021 Kelas II (air minum)\n\n");

    let standards: [(&str, f64, f64, f64, f64); 5] = [
        ("Pb", pb, 0.05, 0.0035, 0.5),
        ("Cd", cd, 0.01, 0.0005, 0.5),
        ("Hg", hg, 0.001, 0.00003, 0.3),
        ("As", as_, 0.05, 0.01, 0.0003),
        ("Cr", cr, 0.05, 0.05, 0.003),
    ];

    let mut total_w: f64 = 0.0;
    let mut total_wq: f64 = 0.0;

    out.push_str(&format!("{:<5} {:>10} {:>10} {:>10} {:>10}\n", "Metal", "Conc(mg/L)", "Std(mg/L)", "Qi", "Wi"));
    out.push_str(&"-".repeat(45).to_string());
    out.push('\n');

    for &(name, conc, std, _rfd, _sf) in standards.iter() {
        let qi = (conc / std) * 100.0;
        let wi = 1.0 / std;
        total_w += wi;
        total_wq += wi * qi;
        out.push_str(&format!("{:<5} {:>10.4} {:>10.4} {:>10.1} {:>10.1}\n", name, conc, std, qi, wi));
    }

    let hpi = total_wq / total_w;

    out.push_str(&format!("\nHPI = {:.2}\n", hpi));
    let hpi_class: &str = if hpi < 100.0 { "Good" } else if hpi < 500.0 { "Lightly-Moderately Polluted" } else { "Heavily Polluted" };
    out.push_str(&format!("HPI Class: {}\n\n", hpi_class));

    out.push_str("HEALTH RISK (US EPA RAGS):\n");

    let bw = body_weight_kg;
    let ir = intake_l_per_day;
    let ed = exposure_years;
    let ef = 365.0;
    let at_nocarc = ed * 365.0;
    let at_carc = 70.0 * 365.0;

    let mut total_hq = 0.0;
    let mut total_ilcr = 0.0;

    for &(name, conc, _std, rfd, sf) in standards.iter() {
        let cdi = (conc * ir * ef * ed) / (bw * at_nocarc);
        let hq = cdi / rfd;
        let ilcr = (conc * ir * ef * ed * sf) / (bw * at_carc);
        total_hq += hq;
        total_ilcr += ilcr;
        out.push_str(&format!("  {}: HQ={:.3}, ILCR={:.2e}\n", name, hq, ilcr));
    }

    out.push_str(&format!("\nTotal HQ (non-carcinogenic): {:.3}\n", total_hq));
    out.push_str(&format!("Total ILCR (carcinogenic): {:.2e}\n\n", total_ilcr));

    let risk_class = if total_hq < 1.0 { "Acceptable (HQ<1)" } else { "Unacceptable (HQ>=1) - action needed" };
    let cancer_class = if total_ilcr < 1e-6 { "Negligible" } else if total_ilcr < 1e-4 { "Acceptable" } else { "Unacceptable" };

    out.push_str(&format!("Non-cancer risk: {}\n", risk_class));
    out.push_str(&format!("Cancer risk: {} (ILCR threshold 1e-6 to 1e-4)\n\n", cancer_class));

    out.push_str("LIMITATION:\n");
    out.push_str("  - HPI weights are inverse of standard (Mohsen 1989)\n");
    out.push_str("  - Assumes oral ingestion only (no inhalation/dermal)\n");
    out.push_str("  - Body weight default 70kg, intake 2L/day (adjustable)\n");
    out.push_str("  - Slope factors are US EPA defaults — may differ for Indonesia\n");
    out.push_str("  - Methylmercury (MeHg) is more toxic than total Hg — tool uses total Hg\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}
