/// ASGM Mercury Assessment
/// Hg mass balance + health risk for Artisanal Small-scale Gold Mining
/// Ref: Agustiani et al. 2025 (Sukabumi); Desmaiani et al. 2026 (W. Kalimantan)

pub fn assess(
    hg_conc_water: f64,
    hg_conc_sediment: f64,
    gold_production_kg_yr: f64,
    population_exposed: u32,
) -> String {
    let mut out = String::new();
    out.push_str("═══════════════════════════════════════════════\n");
    out.push_str("ASGM Mercury Assessment\n");
    out.push_str("Ref: Agustiani et al. 2025; Desmaiani et al. 2026; UNEP 2013\n\n");

    let hg_ratio = 1.5;
    let hg_total = gold_production_kg_yr * hg_ratio;
    let hg_recovered = hg_total * 0.20;
    let hg_atmosphere = hg_total * 0.60;
    let hg_tailings = hg_total * 0.20;

    out.push_str("MERCURY MASS BALANCE:\n");
    out.push_str(&format!("  Gold production: {:.1} kg/year\n", gold_production_kg_yr));
    out.push_str(&format!("  Hg:Au ratio: {} (typical ASGM: 1-3)\n", hg_ratio));
    out.push_str(&format!("  Total Hg used: {:.1} kg/year\n\n", hg_total));

    out.push_str("Hg Distribution (UNEP typical):\n");
    out.push_str(&format!("  Recovered (amalgam): 20% = {:.1} kg/yr\n", hg_recovered));
    out.push_str(&format!("  Atmosphere (burning): 60% = {:.1} kg/yr\n", hg_atmosphere));
    out.push_str(&format!("  Tailings (soil/water): 20% = {:.1} kg/yr\n\n", hg_tailings));

    let bm_hg_water = 0.002;
    let bm_hg_class4 = 0.005;

    out.push_str("WATER QUALITY:\n");
    out.push_str(&format!("  Hg in water: {:.4} mg/L\n", hg_conc_water));
    out.push_str(&format!("  Baku mutu PP 22/2021 Class III: {} mg/L\n", bm_hg_water));
    out.push_str(&format!("  Baku mutu PP 22/2021 Class IV: {} mg/L\n", bm_hg_class4));
    if hg_conc_water > bm_hg_water {
        out.push_str("  ⚠️ EXCEEDS baku mutu Class III (fishery/irrigation)\n");
    }
    if hg_conc_water > bm_hg_class4 {
        out.push_str("  🔴 EXCEEDS baku mutu Class IV (agriculture)\n");
    }
    let exceed_ratio = if bm_hg_water > 0.0 { hg_conc_water / bm_hg_water } else { 0.0 };
    out.push_str(&format!("  Exceedance factor: {:.1}x\n\n", exceed_ratio));

    out.push_str("SEDIMENT:\n");
    out.push_str(&format!("  Hg in sediment: {:.4} mg/kg\n", hg_conc_sediment));
    let sediment_threshold = 0.3;
    if hg_conc_sediment > sediment_threshold {
        out.push_str("  ⚠️ Exceeds sediment quality (0.3 mg/kg, NOAA SQG)\n");
    }

    out.push_str("\nHEALTH RISK:\n");
    let bw = 60.0;
    let ir = 2.0;
    let ef = 365.0;
    let ed = 30.0;
    let at = 70.0 * 365.0;
    let rfd_mehg = 1.0e-4;
    let sf_mehg = 1.0e-1;

    let cdi = (hg_conc_water * ir * ef * ed) / (bw * at);
    let hq = cdi / rfd_mehg;
    let ilcr = (hg_conc_water * ir * ef * ed * sf_mehg) / (bw * at);

    out.push_str(&format!("  CDI: {:.2e} mg/kg/day\n", cdi));
    out.push_str(&format!("  HQ: {:.2} (threshold 1.0)\n", hq));
    out.push_str(&format!("  ILCR: {:.2e} (threshold 1e-6 to 1e-4)\n", ilcr));
    out.push_str(&format!("  Population exposed: {}\n", population_exposed));

    let at_risk = (population_exposed as f64) * (ilcr / 1e-4).max(0.0).min(1.0);
    out.push_str(&format!("  Estimated excess cancer cases: {:.1}\n\n", at_risk));

    out.push_str("MITIGATION:\n");
    out.push_str("  1. Replace Hg with cyanide-free alternatives (borax method)\n");
    out.push_str("  2. Closed-system amalgam burning (retort)\n");
    out.push_str("  3. Tailings containment (no river disposal)\n");
    out.push_str("  4. Health monitoring for miners (urine Hg test)\n");
    out.push_str("  5. Minamata Convention compliance\n\n");

    out.push_str("LIMITATION:\n");
    out.push_str("  - Hg:Au ratio (1.5) is average — varies 0.5-5\n");
    out.push_str("  - Distribution (20/60/20) is UNEP default, site-specific\n");
    out.push_str("  - Health risk assumes oral exposure only (no inhalation of Hg vapor)\n");
    out.push_str("  - Methylmercury (MeHg) is 10x more toxic than inorganic Hg\n");
    out.push_str("  - Bioaccumulation in fish not modeled\n");
    out.push_str("═══════════════════════════════════════════════\n");
    out
}
