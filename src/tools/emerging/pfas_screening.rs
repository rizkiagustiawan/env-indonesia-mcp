/// PFAS Risk Screening — EPA MCL + WHO + 2026 Treatment Comparison
///
/// IMPLEMENTES: EPA MCL (confirmed May 2025) + WHO 2024
/// + 2026 treatment technology comparison from latest papers
///
/// EPA MCL (2024, confirmed May 2025):
///   PFOA: 4 ng/L | PFOS: 4 ng/L
///   PFNA: 10 | PFHxS: 10 | GenX: 10 ng/L
///   Hazard Index: <=1 (mixture)
///
/// WHO (2024):
///   PFOA: 100 ng/L | PFOS: 200 ng/L
///
/// INDONESIA:
///   Belum ada baku mutu PFAS spesifik
///   Compare to EPA/WHO for precautionary assessment

pub fn assess(pfas_type: &str, conc_ng_l: f64, water_source: &str) -> String {
    let mut out = String::from("=== PFAS Risk Screening (2026) ===\n");
    out.push_str("Ref: EPA MCL 2024 (confirmed May 2025); WHO 2024\n\n");

    if conc_ng_l < 0.0 {
        return "ERROR [E102]: conc must be >= 0.".into();
    }

    let (limit_ng_l, source, health_effect) = match pfas_type.to_lowercase().as_str() {
        s if s.contains("pfoa") => (4.0, "EPA MCL", "Cancer, immune, developmental"),
        s if s.contains("pfos") => (4.0, "EPA MCL", "Cancer, immune, thyroid"),
        s if s.contains("pfna") => (10.0, "EPA MCL", "Liver, developmental"),
        s if s.contains("pfhxs") => (10.0, "EPA MCL", "Thyroid, immune"),
        s if s.contains("genx") || s.contains("hpfoda") => (10.0, "EPA MCL", "Liver, kidney"),
        s if s.contains("pfbs") => (0.0, "EPA HI component", "Thyroid, kidney"),
        s if s.contains("pfhxa") => (0.0, "EPA HI component", "Liver"),
        s if s.contains("pfba") => (0.0, "Not regulated", "Limited data"),
        _ => (0.0, "Not regulated", "Unknown"),
    };

    // ═══ Phase 1: Regulatory Limits ═══
    out.push_str("-- Phase 1: Regulatory Limits --\n\n");
    out.push_str(&format!("PFAS: {} (conc: {:.1} ng/L)\n", pfas_type, conc_ng_l));
    out.push_str(&format!("Water source: {}\n\n", water_source));

    out.push_str("EPA MCL (2024, confirmed May 2025):\n");
    out.push_str("  PFOA: 4 ng/L | PFOS: 4 ng/L\n");
    out.push_str("  PFNA: 10 | PFHxS: 10 | GenX: 10 ng/L\n");
    out.push_str("  Hazard Index: <=1 (mixture)\n");
    out.push_str("WHO (2024): PFOA 100 ng/L, PFOS 200 ng/L\n");
    out.push_str("Indonesia: BELUM ADA baku mutu PFAS spesifik\n\n");

    // ═══ Phase 2: Status Kepatuhan ═══
    if limit_ng_l > 0.0 {
        let ratio = conc_ng_l / limit_ng_l;
        out.push_str("-- STATUS KEPATUHAN --\n\n");
        out.push_str(&format!("  {} limit: {:.0} ng/L\n", source, limit_ng_l));
        out.push_str(&format!("  Measured: {:.1} ng/L\n", conc_ng_l));
        out.push_str(&format!("  Ratio: {:.2}x limit\n", ratio));
        out.push_str(if ratio <= 1.0 {"  PASS: MEMENUHI\n\n"} else {"  FAIL: MELEBIHI -- action required!\n\n"});
    } else {
        out.push_str("-- STATUS: Not individually regulated --\n");
        out.push_str("  Check Hazard Index if in mixture\n\n");
    }

    // ═══ Phase 3: Health Effects ═══
    out.push_str(&format!("-- Health Effects --\n  {}\n\n", health_effect));

    // ═══ Phase 4: 2026 Treatment Options ═══
    out.push_str("-- 2026 Treatment Options --\n\n");
    out.push_str("Technology             Removal    Energy(kWh/m3)  Cost    Best for\n");
    out.push_str("----------             -------    --------------  ----    --------\n");
    out.push_str("Electro-NF (E-NF)      90.4%      1.92            Med     Drinking water\n");
    out.push_str("GAC                    70-85%     0.10            Low     Long-chain\n");
    out.push_str("IEX                    80-95%     0.20            Med     Short+long chain\n");
    out.push_str("MOF (PCN-999)          >99%       0.05            High    All PFAS\n");
    out.push_str("SCWO                   >99.99%    15-50           High    Concentrate\n");
    out.push_str("Foam fractionation     85-99%     0.01            Low     Long-chain\n");
    out.push_str("Colloidal Carbon (CCP) >99.9%     N/A (in-situ)   Med     Groundwater\n");
    out.push_str("HIP+GAC pretreat       +350% BV   0.10            Low     Short-chain\n");
    out.push_str("Photocatalytic TiO2    80%        0.50            Low     Leachate\n");
    out.push_str("Electrochemical ox     95-99%     5-20            High    Concentrate\n\n");

    // ═══ Phase 5: Recommendation ═══
    out.push_str("-- RECOMMENDATION --\n\n");
    if conc_ng_l > limit_ng_l && limit_ng_l > 0.0 {
        out.push_str("  1. Treatment: select from table above based on water source\n");
        out.push_str("  2. Destruction: SCWO or electrochemical for concentrate\n");
        out.push_str("  3. Source investigation (AFFF, textile, electroplating)\n");
        out.push_str("  4. Alternative water supply if acute risk\n\n");

        if water_source.to_lowercase().contains("drinking") || water_source.to_lowercase().contains("air") {
            out.push_str("  >> Drinking water: recommend E-NF or GAC+IEX polish\n");
        } else if water_source.to_lowercase().contains("ground") || water_source.to_lowercase().contains("tanah") {
            out.push_str("  >> Groundwater: recommend CCP injection or pump+GAC\n");
        } else if water_source.to_lowercase().contains("leachate") {
            out.push_str("  >> Leachate: recommend foam fractionation + SCWO\n");
        } else {
            out.push_str("  >> General: GAC as first step, E-NF for strict limits\n");
        }
    } else {
        out.push_str("  Monitor -- annual testing recommended (EPA 1633)\n");
    }

    // ═══ PEMANTAUAN ═══
    out.push_str("\n-- PEMANTAUAN --\n");
    out.push_str("  Parameter: PFAS target list (EPA 1633, 40 compounds)\n");
    out.push_str("  Frekuensi: Annual (drinking), Quarterly (contaminated site)\n");
    out.push_str("  Metode: EPA 1633 (LC-MS/MS), LOQ 1-10 ng/L\n");
    out.push_str("  Alternatif: 19F-NMR (low-cost screening, Earl 2026)\n\n");

    // ═══ Indonesia Context ═══
    out.push_str("-- Indonesia Context --\n");
    out.push_str("  Sources: AFFF (airport/military), textile (Bandung),\n");
    out.push_str("    electroplating, food packaging, firefighting training\n");
    out.push_str("  Permen LH 12/2025: tekstil effluent (PFAS not yet regulated)\n");
    out.push_str("  Permen LH 6/2026: sanksi administratif (denda formula)\n");
    out.push_str("  Recommendation: adopt EPA MCL as precautionary standard\n\n");

    // ═══ Limitations ═══
    out.push_str("-- Limitations (honest) --\n");
    out.push_str("  • EPA MCL values (Indonesia may differ when regulated)\n");
    out.push_str("  • No bioaccumulation/biomagnification modeling\n");
    out.push_str("  • No mixture toxicity (Hazard Index not computed)\n");
    out.push_str("  • Treatment costs are approximate (site-specific)\n");
    out.push_str("  • Ref: EPA MCL 2024; WHO 2024; Hua 2026; Lee 2025\n");

    out
}
