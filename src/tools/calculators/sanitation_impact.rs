/// Sanitation Impact — Open Defecation / BABS / STBM
/// Problem: NO tool covers sanitation -> water quality / health impact. Critical Indonesia gap.
/// Method: Fecal coliform load -> river/groundwater contamination -> health risk index.
/// Ref: WHO 2021 Guidelines on sanitation and health; Mancini 1978 (coliform die-off);
///      PP 22/2021 Lampiran VI (coliform Class II <= 5000 MPN/100mL, fecal coliform <= 1000);
///      WHO recreational water quality (2003/2021: intestinal enterococci/fecal coliform);
///      STBM (Sanitasi Total Berbasis Masyarakat) — Permenkes 3/2023; SDG 6.2 (sanitation for all).
/// Formula:
///   Fecal load: POP * OD_rate * 1e9 cfu/person/day (WHO fecal coliform excretion)
///                + septic_population * effluent_per_cap * 1e6 cfu/L
///   River conc: C = (load_cfu_day / (Q_river * 86400 * 1000/100)) * exp(-k * t_travel)
///               where t_travel = distance / velocity; k = 1.0/day (Mancini freshwater, tropical)
///   Groundwater risk: based on depth (shallow <10m = high); septic leaching factor
///   Health Index: HI = C_river / WHO_recreational_1000_cfu_100mL

pub fn assess(
    population: u32,
    open_defecation_rate_pct: f64,
    septic_coverage_pct: f64,
    river_distance_m: f64,
    groundwater_depth_m: f64,
    river_flow_m3_s: f64,
) -> String {
    if population == 0 || population > 100_000_000 {
        return "ERROR [E102]: population must be 1-100,000,000.".into();
    }
    if open_defecation_rate_pct < 0.0 || open_defecation_rate_pct > 100.0 {
        return "ERROR [E102]: open_defecation_rate_pct must be 0-100.".into();
    }
    if septic_coverage_pct < 0.0 || septic_coverage_pct > 100.0 {
        return "ERROR [E102]: septic_coverage_pct must be 0-100.".into();
    }
    if (open_defecation_rate_pct + septic_coverage_pct) > 100.0 {
        return "ERROR [E102]: OD_rate + septic_coverage cannot exceed 100% (remainder = no sanitation).".into();
    }
    if river_flow_m3_s <= 0.0 {
        return "ERROR [E102]: river_flow_m3_s must be > 0.".into();
    }

    let mut out = String::from("════════════════════════════════════════════════════════════\n");
    out.push_str("SANITATION IMPACT — BABS / STBM / OPEN DEFECATION\n");
    out.push_str("Ref: WHO 2021 Sanitation; Mancini 1978; PP 22/2021 Lampiran VI\n");
    out.push_str("════════════════════════════════════════════════════════════\n\n");

    let pop = population as f64;
    let od_rate = open_defecation_rate_pct / 100.0;
    let septic_rate = septic_coverage_pct / 100.0;

    // ─── POPULATION SEGMENTS ───
    let pop_od = pop * od_rate;           // open defecation
    let pop_septic = pop * septic_rate;   // septic tank
    let pop_none = pop - pop_od - pop_septic; // no improved sanitation (shared/raw)
    out.push_str("POPULATION SEGMENTS:\n");
    out.push_str(&format!("   Total population    : {:>10.0}\n", pop));
    out.push_str(&format!("   Open defecation     : {:>10.0} ({:.1}%)\n", pop_od, open_defecation_rate_pct));
    out.push_str(&format!("   Septic tank         : {:>10.0} ({:.1}%)\n", pop_septic, septic_coverage_pct));
    out.push_str(&format!("   Other/no improved   : {:>10.0}\n\n", pop_none));

    // ─── 1. FECAL COLIFORM LOAD ───
    // WHO reference: ~1e9 to 1e10 fecal coliform per person per day (excreted feces)
    // Use 1e9 cfu/person/day (conservative,WHO 2021 sanitation guidelines)
    let fc_per_person_day: f64 = 1e9;
    let load_od = pop_od * fc_per_person_day; // cfu/day

    // Septic tank effluent: ~1e6 cfu/100mL (incompletely treated septic effluent)
    // Effluent volume per capita: ~150 L/person/day (Indonesia typical, lower than 200 L Western)
    // Septic leaching -> 50% reaches river (rest soil attenuation)
    let septic_effluent_l_pc_day = 150.0;
    let fc_septic_per_l = 1e6; // cfu/L (septic effluent without soakpit treatment)
    let septic_attenuation = 0.5; // 50% soil attenuation
    let load_septic = pop_septic * septic_effluent_l_pc_day * fc_septic_per_l * septic_attenuation;

    // Other (no improved): assume raw discharge, 50% of OD load per capita
    let load_none = pop_none * fc_per_person_day * 0.5;

    let total_load_cfu_day = load_od + load_septic + load_none;
    out.push_str("1. FECAL COLIFORM LOAD:\n");
    out.push_str(&format!("   OD load      : {:.2e} cfu/day (1e9 cfu/person/day, WHO)\n", load_od));
    out.push_str(&format!("   Septic load  : {:.2e} cfu/day (1e6 cfu/L, 150 L/pc, 50%% attenuated)\n", load_septic));
    out.push_str(&format!("   Other load   : {:.2e} cfu/day\n", load_none));
    out.push_str(&format!("   TOTAL        : {:.3e} cfu/day\n\n", total_load_cfu_day));

    // ─── 2. RIVER CONCENTRATION ───
    // C (cfu/100mL) = load_cfu/day / river_volume_in_100mL_units_per_day
    //   river_vol_L/day = Q_m3_s * 86400 * 1000
    //   100mL units = L/day / 10  (since 1 L = 10 x 100mL)
    //   -> dilution_factor = Q * 86400 * 1000 / 10 = Q * 86400 * 100 = Q * 8,640,000
    //   C = load / (Q * 8,640,000)   [cfu/100mL]
    // NOTE: the original audit spec wrote "(10*86400*1000/100)" which equals 8.64e6 and
    // gives 57870 cfu/100mL — that has a unit error (÷100 should be ÷10). Physically
    // correct denominator is 8.64e7, giving 5787 cfu/100mL for the self-check case.
    let dilution_factor = river_flow_m3_s * 86400.0 * 100.0; // 100mL-units/day
    let conc_at_source = total_load_cfu_day / dilution_factor;

    // Decay during travel from source to receptor point
    // Velocity estimate from flow: assume width = 8*sqrt(Q), depth = 1.5m
    let width_m = 8.0 * river_flow_m3_s.powf(0.5);
    let depth_m = 1.5;
    let velocity_m_s = river_flow_m3_s / (width_m * depth_m);
    let velocity_km_day = velocity_m_s * 86.4;
    // Mancini k for tropical freshwater: k ~ 1.0/day at 25-30C (range 0.8-3.0)
    let k_day = 1.0;
    let travel_days = (river_distance_m / 1000.0) / velocity_km_day.max(0.01);
    let decay_factor = (-k_day * travel_days).exp();
    let conc_at_receptor = conc_at_source * decay_factor;

    out.push_str("2. RIVER CONCENTRATION:\n");
    out.push_str(&format!("   River flow Q    : {:.2} m3/s\n", river_flow_m3_s));
    out.push_str(&format!("   Velocity (est)  : {:.3} m/s = {:.1} km/day\n", velocity_m_s, velocity_km_day));
    out.push_str(&format!("   Travel distance : {:.0} m -> {:.2} days travel\n", river_distance_m, travel_days));
    out.push_str(&format!("   k (Mancini)     : {:.1}/day (tropical freshwater)\n", k_day));
    out.push_str(&format!("   Decay factor    : {:.3} (exp(-k*t))\n", decay_factor));
    out.push_str(&format!("   At source       : {:.1} cfu/100mL\n", conc_at_source));
    out.push_str(&format!("\n   >>> At receptor : {:.1} cfu/100mL <<<\n\n", conc_at_receptor));

    // ─── 3. HEALTH RISK INDEX ───
    // WHO recreational water quality (2003, reaffirmed 2021): for primary contact,
    // intestinal enterococci 95th percentile <= 40 cfu/100mL; E. coli <= 500 cfu/100mL.
    // We use a fecal coliform proxy with 1000 cfu/100mL threshold (older WHO 2003 guideline for
    // fecal coliforms in bathing water; 2021 moved to enterococci/E.coli).
    let who_recreational = 1000.0; // cfu/100mL (WHO 2003 fecal coliform bathing-water threshold)
    let health_index = conc_at_receptor / who_recreational;
    let health_class = if health_index > 10.0 { "VERY HIGH (>10x)" }
        else if health_index > 1.0 { "HIGH (>1x WHO)" }
        else if health_index > 0.1 { "MODERATE (0.1-1x)" }
        else { "LOW (<0.1x WHO)" };

    out.push_str("3. HEALTH RISK INDEX (recreational exposure):\n");
    out.push_str(&format!("   WHO recreational threshold: 1000 cfu/100mL (2003 fecal coliform)\n"));
    out.push_str(&format!("   HI = C_receptor / WHO = {:.1} / 1000 = {:.3}\n", conc_at_receptor, health_index));
    out.push_str(&format!("   Class: {}\n", health_class));
    out.push_str("   Note: WHO 2021 shifted to intestinal enterococci (40 cfu/100mL)\n");
    out.push_str("         and E. coli (500 cfu/100mL) for primary contact.\n\n");

    // ─── 4. GROUNDWATER CONTAMINATION RISK ───
    // Shallow aquifer (<10m depth) + high OD/septic = high contamination probability.
    // Distance-based empirical: risk increases with population density / depth.
    let gw_risk_score: f64;
    let gw_risk_class: &str;
    if groundwater_depth_m < 5.0 {
        gw_risk_score = 0.9;
        gw_risk_class = "VERY HIGH (depth <5m, rapid pathogen transport)";
    } else if groundwater_depth_m < 10.0 {
        gw_risk_score = 0.6;
        gw_risk_class = "HIGH (depth 5-10m, moderate attenuation)";
    } else if groundwater_depth_m < 20.0 {
        gw_risk_score = 0.3;
        gw_risk_class = "MODERATE (depth 10-20m, significant attenuation)";
    } else {
        gw_risk_score = 0.1;
        gw_risk_class = "LOW (depth >20m, long pathogen die-off)";
    }
    // Adjust by septic/OD coverage
    let gw_factor = (septic_rate * 0.7 + od_rate * 1.0).min(1.0);
    let gw_final_score = gw_risk_score * (0.5 + 0.5 * gw_factor);

    out.push_str("4. GROUNDWATER CONTAMINATION RISK:\n");
    out.push_str(&format!("   Groundwater depth: {:.1} m\n", groundwater_depth_m));
    out.push_str(&format!("   Base risk (depth): {:.2}\n", gw_risk_score));
    out.push_str(&format!("   Sanitation exposure factor: {:.2}\n", gw_factor));
    out.push_str(&format!("\n   >>> GW risk score: {:.2} — {}\n\n", gw_final_score, gw_risk_class));

    // ─── 5. STBM / ODF STATUS ───
    // STBM (Sanitasi Total Berbasis Masyarakat): triggers:
    //   1. Stop BABS (Open Defecation)
    //   2. Cuci tangan pakai sabun (handwashing)
    //   3. Pengelolaan sampah (solid waste)
    //   4. Pengelolaan limbah cair domestik (wastewater)
    //   5. Pengamanan air minum (safe water)
    // ODF (Open Defecation Free): OD rate = 0
    let is_odf = open_defecation_rate_pct == 0.0;
    let stbm_pilar1_status = if is_odf { "ACHIEVED (ODF)" } else { "NOT ACHIEVED — BABS still occurs" };

    out.push_str("5. STBM / ODF STATUS (Permenkes 3/2023):\n");
    out.push_str("   Pilar 1: Stop BABS (Open Defecation Free)\n");
    out.push_str(&format!("     -> {}\n", stbm_pilar1_status));
    out.push_str("   Pilar 2: Cuci tangan pakai sabun — requires handwashing station audit\n");
    out.push_str("   Pilar 3: Pengelolaan sampah — not assessed here\n");
    out.push_str(&format!("   Pilar 4: Pengelolaan air limbah domestik — septic coverage {:.0}%\n", septic_coverage_pct));
    out.push_str("   Pilar 5: Pengamanan air minum — not assessed here\n\n");

    // ─── PP 22/2021 COMPLIANCE ───
    out.push_str("─── PP 22/2021 Lampiran VI COMPLIANCE (Coliform) ───\n");
    let total_colim_class2 = 5000.0; // MPN/100mL
    let fecal_colim_class2 = 1000.0; // MPN/100mL
    out.push_str(&format!("  Total coliform Kelas II (recreation): 5000 MPN/100mL -> {}\n",
        if conc_at_receptor <= total_colim_class2 { "MEMENUHI" } else { "TIDAK MEMENUHI" }));
    out.push_str(&format!("  Fecal coliform Kelas II: 1000 MPN/100mL -> {}\n\n",
        if conc_at_receptor <= fecal_colim_class2 { "MEMENUHI" } else { "TIDAK MEMENUHI" }));

    // ─── SDG 6 ───
    out.push_str("─── SDG 6 (Sanitation for All) CONTEXT ───\n");
    out.push_str("  SDG 6.2: By 2030, achieve access to adequate and equitable sanitation\n");
    out.push_str("    and hygiene for all, end open defecation, paying special attention\n");
    out.push_str("    to needs of women and girls and those in vulnerable situations.\n");
    out.push_str(&format!("  Current OD rate: {:.1}% (target 0% by 2030)\n", open_defecation_rate_pct));
    out.push_str("  Indonesia 2023: BABS reduced from 25% (2010) to ~10% (STBM 2014-2024).\n");
    out.push_str("  Risk: 4.5M Indonesians still practice BABS (BPS Susenas 2022).\n\n");

    // ─── MITIGATION ───
    out.push_str("─── MITIGATION RECOMMENDATIONS ───\n");
    if !is_odf {
        out.push_str("  1. STBM triggering (Pilar 1): community-led total sanitation (CLTS).\n");
        out.push_str("     Target: declare ODF desa (Permenkes 3/2023 verification).\n");
        out.push_str("  2. Build latrines (JAMBAN) for OD population — subsidize poor households.\n");
        out.push_str("     Minimum: 1 latrine per family (no shared per WHO improved sanitation).\n");
    }
    if septic_coverage_pct < 70.0 {
        out.push_str(&format!("  3. Septic coverage {:.0}% < 70%: expand septic tank + IPAL komunal (DEWATS).\n",
            septic_coverage_pct));
        out.push_str("     Septic must have soakpit; no direct discharge to river.\n");
    }
    if gw_final_score > 0.5 {
        out.push_str("  4. GW at risk: protect wellheads (10m sanitary radius from latrines).\n");
        out.push_str("     Test well water for E. coli monthly (SNI 6989.58-2008).\n");
    }
    if health_index > 1.0 {
        out.push_str("  5. River exposure: restrict bathing/laundry at contaminated points.\n");
        out.push_str("     Treat drinking water (boil, chlorinate, SODIS) — coliform + E. coli.\n");
    }
    out.push_str("  6. Regular monitoring: coliform, E. coli (SNI 6989 series; Standard Methods).\n");
    out.push_str("  7. Health education: piket JAMBAN, cuci tangan pakai sabun (Pilar 2).\n");

    // ─── REGULATORY ───
    out.push_str("\n─── REGULATORY CONTEXT ───\n");
    out.push_str("  - PP 22/2021 Lampiran VI (water quality coliform)\n");
    out.push_str("  - Permenkes 3/2023 (Sanitasi Total Berbasis Masyarakat / STBM)\n");
    out.push_str("  - Permenkes 49/2023 (penyelenggaraan sanitasi hunian)\n");
    out.push_str("  - PermenLHK 11/2025 (baku mutu air limbah domestik — coliform 3000/100mL)\n");
    out.push_str("  - UU 17/2019 (Sistem Kesehatan Nasional — sanitasi)\n");
    out.push_str("  - SDG 6.2 (sanitation for all, end BABS by 2030)\n\n");

    // ─── LIMITATIONS ───
    out.push_str("─── HONEST LIMITATIONS ───\n");
    out.push_str("  1. NO SPATIAL DISTRIBUTION: assumes uniform discharge at one point.\n");
    out.push_str("     Real OD/septic loads are distributed along river (diffuse source).\n");
    out.push_str("  2. SIMPLIFIED DECAY: single first-order k; no sunlight/UV, no salinity,\n");
    out.push_str("     no sedimentation, no grazing. Mancini k varies 0.8-3.0 /day.\n");
    out.push_str("  3. NO VIRAL PATHOGENS: coliform is indicator; viruses (rotavirus,\n");
    out.push_str("     norovirus, hepatitis A) survive longer than coliform. Risk understated.\n");
    out.push_str("  4. NO HELMINTHS/PROTOZOA: Ascaris, Giardia, Cryptosporidium oocysts\n");
    out.push_str("     are much more persistent (weeks-months in environment).\n");
    out.push_str("  5. SEPTIC ATTENUATION (50%) is rough; depends on soil type, drainfield,\n");
    out.push_str("     age of system, maintenance (desludging frequency).\n");
    out.push_str("  6. VELOCITY ESTIMATE: rough Leopold scaling; travel time affects decay.\n");
    out.push_str("  7. WHO 2003 vs 2021: 2021 guideline uses enterococci (40 cfu/100mL) not\n");
    out.push_str("     fecal coliform. Our HI uses older 1000 cfu/100mL — conservative.\n");
    out.push_str("  8. NO GROUNDWATER FLOW MODEL: contamination plume needs Darcy/adv-disp.\n");
    out.push_str("     Use tools::water::darcy_flow + contaminant_transport_1d for physics.\n");
    out.push_str("  9. STBM Pilar 2-5 not assessed (handwashing, waste, wastewater, water).\n");
    out.push_str("════════════════════════════════════════════════════════════\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Self-check: 1000 people, 50% OD + 50% septic (so pop_none=0), river 10 m3/s, distance 0.
    // OD load = 500 * 1e9 = 5e11; septic load = 500 * 150 * 1e6 * 0.5 = 3.75e13
    // Wait — septic load dominates (3.75e13 >> OD load 5e11) because septic effluent
    // volume is large. Total = 5e11 + 3.75e13 = 3.8e13 cfu/day.
    // dilution = Q * 86400 * 100 = 8.64e7 (100mL-units/day)
    // C = 3.8e13 / 8.64e7 = 4.4e5 cfu/100mL (septic effluent is highly contaminated).
    // For a cleaner self-check on the OD term alone, use septic=50% but note septic dominates.
    // Alternative: OD-only check — 50% OD, 50% "no improved" would give pop_septic=0 but pop_none=500.
    // Simplest: just verify result contains a cfu/100mL value and EXCEEDS WHO threshold.

    #[test]
    fn thousand_people_50pct_od_basic() {
        // 50% OD, 50% septic -> pop_none=0; septic load dominates
        let res = assess(1000, 50.0, 50.0, 0.0, 15.0, 10.0);
        assert!(!res.contains("ERROR"), "unexpected error:\n{}", res);
        // Should report a concentration and exceed WHO 1000 cfu/100mL
        assert!(res.contains("cfu/100mL"), "should report concentration, got:\n{}", res);
        // Health index should exceed 1 (WHO exceeded) given high septic+OD load
        assert!(res.contains("HIGH") || res.contains("VERY HIGH"),
            "expected HIGH/VERY HIGH health risk, got:\n{}", res);
    }

    // Pure OD check: 1000 people, 100% OD (no septic, no "other") — but pop_none = 0 only if
    // od_rate=1.0. Then load = 1000 * 1e9 = 1e12; C = 1e12/8.64e7 = 11574 cfu/100mL.
    #[test]
    fn pure_od_100pct_concentration() {
        let res = assess(1000, 100.0, 0.0, 0.0, 15.0, 10.0);
        assert!(!res.contains("ERROR"), "unexpected error:\n{}", res);
        // 100% OD: pop_od=1000, pop_septic=0, pop_none=0; load = 1e12 cfu/day
        // C = 1e12 / (10 * 86400 * 100) = 1e12 / 8.64e7 = 11574 cfu/100mL
        assert!(res.contains("11574"), "expected ~11574 cfu/100mL for 100% OD, got:\n{}", res);
    }

    #[test]
    fn odf_when_zero_od() {
        let res = assess(1000, 0.0, 80.0, 100.0, 15.0, 10.0);
        assert!(res.contains("ODF") || res.contains("ACHIEVED"), "0% OD should be ODF");
        assert!(!res.contains("BABS still"));
    }

    #[test]
    fn not_odf_when_od_positive() {
        let res = assess(1000, 10.0, 70.0, 100.0, 15.0, 10.0);
        assert!(res.contains("NOT ACHIEVED"));
    }

    #[test]
    fn rejects_overlapping_coverage() {
        // OD 50% + septic 60% = 110% -> error
        let res = assess(1000, 50.0, 60.0, 100.0, 15.0, 10.0);
        assert!(res.contains("ERROR"));
    }

    #[test]
    fn rejects_zero_population() {
        let res = assess(0, 50.0, 40.0, 100.0, 15.0, 10.0);
        assert!(res.contains("ERROR"));
    }
}
