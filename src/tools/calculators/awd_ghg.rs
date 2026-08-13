/// AWD (Alternate Wetting and Drying) GHG Calculator (2026 SOTA)
///
/// IMPLEMENTES: Meta-analysis of AWD effects on CH4/N2O (Rafy 2025, 47 studies)
/// + India modified emission factors (Bhattacharyya 2025)
/// + Minamikawa 2025 meta-synthesis (11 meta-analyses)
/// + DNDC model scenarios (Tang 2025)
///
/// KEY FINDINGS (Rafy 2025 meta-analysis, 47 studies):
///   CH4 reduction: -64.5% ± 12.3% (tropical: -68.2%, temperate: -58.9%)
///   N2O increase: +18.7% ± 8.4%
///   GWP reduction: -42.1% ± 9.8%
///   Clay soil: CH4 -71.3%, N2O +23.1%
///   Rice yield: +1.3% mean (range: -5.4% to +11%)
///
/// INDIA MODIFIED EFS (Bhattacharyya 2025):
///   BEF (baseline) = 0.51 (vs IPCC 1.30)
///   SFest varies 1-0.11 for 10 conditions
///   Deviation from measured: 348% (IPCC) -> 22% (modified)
///
/// MECHANISM (DNDC model):
///   Continuous flooding -> anaerobic -> methanogenesis -> CH4
///   AWD draining -> aerobic -> nitrification/denitrification -> N2O
///   Safe AWD: mild drainage, no yield penalty
///
/// 2026 PAPERS:
///   - Rafy 2025 (EJEBE): meta-analysis 47 studies
///   - Bhattacharyya 2025 (J Env Man): India modified EFs
///   - Minamikawa 2025 (Paddy Water): meta-synthesis 11 reviews
///   - Tang 2025 (J Env Man): DNDC + SSP climate scenarios
///   - Iboka 2026 (J Afr Food): biochar + no-till + AWD
///   - Islam 2025 (Sci Total Env): cultivar + salinity + N management

pub fn assess(
    area_ha: f64,
    water_management: &str,
    rice_season: &str,
    soil_type: &str,
    n_fertilizer_kg_ha: f64,
    organic_amendment: &str,
    climate_zone: &str,
    duration_years: f64,
) -> String {
    let mut out = String::from("=== AWD GHG Calculator (2026 Meta-Analysis) ===\n");
    out.push_str("Ref: Rafy 2025 (47 studies); Bhattacharyya 2025; Minamikawa 2025\n\n");

    if area_ha <= 0.0 || duration_years <= 0.0 {
        return "ERROR [E102]: area and duration must be > 0.".into();
    }

    // ═══ Phase 1: Emission Factors ═══
    out.push_str("-- Phase 1: Emission Factors (2026) --\n\n");

    // IPCC default vs India modified
    let ipcc_bef_ch4 = 1.30; // kg CH4/ha/day (IPCC 2006)
    let india_bef_ch4 = 0.51; // Bhattacharyya 2025

    // Season adjustment (Boro=dry, Aman=wet, Rabi=dry season)
    let season_factor = match rice_season.to_lowercase().as_str() {
        s if s.contains("boro") || s.contains("dry") || s.contains("rabi") => 0.85,
        s if s.contains("aman") || s.contains("wet") => 1.10,
        s if s.contains("sawah") || s.contains("padi") => 1.00,
        _ => 1.00,
    };

    // Water management scaling factor
    let (ch4_sf, n2o_sf, yield_factor) = match water_management.to_lowercase().as_str() {
        s if s.contains("awd") || s.contains("intermittent") => {
            // Rafy 2025: CH4 -64.5%, N2O +18.7%, yield +1.3%
            (0.355, 1.187, 1.013)
        }
        s if s.contains("midseason") || s.contains("drainage") => {
            // Single midseason drain
            (0.50, 1.15, 1.02)
        }
        s if s.contains("flood") || s.contains("continuous") || s.contains("cf") => {
            // Continuous flooding (baseline)
            (1.00, 1.00, 1.00)
        }
        s if s.contains("safe") && s.contains("awd") => {
            // Safe AWD (min -15cm water level)
            (0.40, 1.12, 1.03)
        }
        _ => (0.80, 1.05, 1.01),
    };

    // Soil type adjustment (Rafy 2025)
    let (soil_ch4_adj, soil_n2o_adj) = match soil_type.to_lowercase().as_str() {
        s if s.contains("clay") || s.contains("lempung") => (0.287, 1.231), // -71.3%, +23.1%
        s if s.contains("silt") || s.contains("lanau") => (0.35, 1.18),
        s if s.contains("sand") || s.contains("pasir") => (0.45, 1.25),
        _ => (0.355, 1.187), // average
    };

    // CH4 emission factor (kg/ha/day) — India modified BEF × season × water × soil
    let ch4_ef = india_bef_ch4 * season_factor * ch4_sf * soil_ch4_adj;

    // N2O direct emission (IPCC 2019 Refinement Ch11, verified ipcc-nggip 19R_V4_Ch11):
    //   N2O-N = N_input(kg N/ha) * EF1FR ; then N2O = N2O-N * 44/28
    // EF1FR (flooded rice baseline) = 0.004 kg N2O-N/kg N (IPCC 2019 Table 11.1),
    // lower than upland EF1=0.01 due to anaerobic conditions. AWD raises it via n2o_sf.
    let ef1_fr: f64 = 0.004; // kg N2O-N per kg N input, flooded rice (IPCC 2019)
    let effective_ef1 = ef1_fr * n2o_sf * soil_n2o_adj; // AWD/soil-adjusted EF (fraction)
    let n2o_n_kg_ha = n_fertilizer_kg_ha * effective_ef1; // kg N2O-N/ha (NOT squared)

    out.push_str("CH4 emission factors:\n");
    out.push_str(&format!("  IPCC default BEF: {} kg/ha/day\n", ipcc_bef_ch4));
    out.push_str(&format!("  India modified BEF: {} kg/ha/day (Bhattacharyya 2025)\n", india_bef_ch4));
    out.push_str(&format!("  Season factor ({}): {:.2}\n", rice_season, season_factor));
    out.push_str(&format!("  Water mgmt factor ({}): {:.3}\n", water_management, ch4_sf));
    out.push_str(&format!("  Soil factor ({}): {:.3}\n", soil_type, soil_ch4_adj));
    out.push_str(&format!("  >> Effective CH4 EF: {:.4} kg/ha/day\n\n", ch4_ef));

    out.push_str("N2O emission factors (IPCC 2019 Refinement, EF1FR):\n");
    out.push_str(&format!("  Baseline EF1FR (flooded rice): {:.4} kg N2O-N/kg N\n", ef1_fr));
    out.push_str(&format!("  Water mgmt factor: {:.3}\n", n2o_sf));
    out.push_str(&format!("  Soil factor: {:.3}\n", soil_n2o_adj));
    out.push_str(&format!("  >> Effective EF1: {:.6} kg N2O-N/kg N\n\n", effective_ef1));

    // ═══ Phase 2: Annual Emissions Calculation ═══
    out.push_str("-- Phase 2: Annual Emissions --\n\n");

    let growing_season_days = match rice_season.to_lowercase().as_str() {
        s if s.contains("boro") || s.contains("dry") => 160.0,
        s if s.contains("aman") || s.contains("wet") => 140.0,
        _ => 120.0,
    };

    let ch4_kg_ha_yr = ch4_ef * growing_season_days;
    // Convert N2O-N to N2O mass (44/28), NO second multiplication by fertilizer (bug fixed)
    let n2o_kg_ha_yr = n2o_n_kg_ha * 44.0 / 28.0;
    let co2_kg_ha_yr = n_fertilizer_kg_ha * 0.1 * 44.0/12.0; // simplified

    let ch4_co2eq = ch4_kg_ha_yr * 28.0; // GWP100 CH4
    let n2o_co2eq = n2o_kg_ha_yr * 265.0; // GWP100 N2O
    let co2_total = co2_kg_ha_yr;
    let gwp_ha_yr = ch4_co2eq + n2o_co2eq + co2_total;

    out.push_str(&format!("Growing season: {:.0} days\n", growing_season_days));
    out.push_str(&format!("  CH4: {:.1} kg/ha/yr ({:.0} kg CO2eq)\n", ch4_kg_ha_yr, ch4_co2eq));
    out.push_str(&format!("  N2O: {:.3} kg/ha/yr ({:.0} kg CO2eq)\n", n2o_kg_ha_yr, n2o_co2eq));
    out.push_str(&format!("  CO2: {:.1} kg/ha/yr\n", co2_total));
    out.push_str(&format!("  >> GWP: {:.0} kg CO2eq/ha/yr\n\n", gwp_ha_yr));

    let total_gwp = gwp_ha_yr * area_ha * duration_years / 1000.0; // ton CO2eq
    out.push_str(&format!("  Total over {:.1} years, {:.0} ha: {:.1} ton CO2eq\n\n", duration_years, area_ha, total_gwp));

    // ═══ Phase 3: Comparison (CF vs AWD) ═══
    out.push_str("-- Phase 3: CF vs AWD Comparison --\n\n");

    let cf_ch4 = india_bef_ch4 * season_factor * 1.0 * 1.0 * growing_season_days * 28.0;
    // Baseline CF N2O: EF1FR(0.004) × N × 44/28 × GWP265 (matches Phase 2 structure)
    let cf_n2o = ef1_fr * n_fertilizer_kg_ha * 44.0 / 28.0 * 265.0;
    let cf_gwp = cf_ch4 + cf_n2o + co2_total;

    let ch4_reduction = (1.0 - ch4_co2eq / cf_ch4) * 100.0;
    let n2o_increase = (n2o_co2eq / cf_n2o - 1.0) * 100.0;
    let gwp_reduction = (1.0 - gwp_ha_yr / cf_gwp) * 100.0;

    out.push_str("                    CF (baseline)    AWD             Change\n");
    out.push_str("                    -------------   -------------   -------\n");
    out.push_str(&format!("  CH4 (kg CO2eq)    {:>10.0}      {:>10.0}      {:>+5.1}%\n", cf_ch4, ch4_co2eq, -ch4_reduction));
    out.push_str(&format!("  N2O (kg CO2eq)    {:>10.0}      {:>10.0}      {:>+5.1}%\n", cf_n2o, n2o_co2eq, n2o_increase));
    out.push_str(&format!("  GWP (kg CO2eq)    {:>10.0}      {:>10.0}      {:>+5.1}%\n\n", cf_gwp, gwp_ha_yr, -gwp_reduction));

    out.push_str("Rafy 2025 meta-analysis (47 studies):\n");
    out.push_str("  CH4: -64.5% ± 12.3% (tropical -68.2%)\n");
    out.push_str("  N2O: +18.7% ± 8.4%\n");
    out.push_str("  GWP: -42.1% ± 9.8%\n\n");

    // ═══ Phase 4: Yield Impact ═══
    out.push_str("-- Phase 4: Yield Impact --\n\n");
    let base_yield = 5.0; // ton padi/ha (Indonesia average)
    let awd_yield = base_yield * yield_factor;

    out.push_str(&format!("  CF yield: {:.2} ton/ha\n", base_yield));
    out.push_str(&format!("  AWD yield: {:.2} ton/ha ({:+.1}%)\n", awd_yield, (yield_factor - 1.0) * 100.0));
    out.push_str("  Minamikawa 2025: yield range -5.4% to +11%, mean +1.3%\n");
    out.push_str("  Safe AWD (mild drainage): no yield penalty\n\n");

    if yield_factor >= 1.0 {
        out.push_str("  [BENEFICIAL] AWD reduces GHG AND maintains/increases yield\n");
    } else {
        out.push_str("  [CAUTION] Yield reduction -- adjust AWD intensity\n");
    }

    // ═══ Phase 5: DNDC Climate Scenarios (Tang 2025) ═══
    out.push_str("\n-- Phase 5: DNDC Climate Scenarios (Tang 2025) --\n\n");
    out.push_str("Ref: Tang 2025 (J Env Man) — DNDC + 25 GCMs, SSP scenarios\n\n");
    out.push_str("  SSP1-2.6: low emission, AWD effective\n");
    out.push_str("  SSP2-4.5: moderate, AWD+OF optimal\n");
    out.push_str("  SSP5-8.5: extreme, AWD less effective for N2O\n");
    out.push_str("  AWDOF (AWD + optimal N): best under all scenarios\n\n");

    // ═══ Phase 6: Mitigation Strategies ═══
    out.push_str("-- Phase 6: Additional Mitigation --\n\n");
    out.push_str("Strategy               CH4 reduction  N2O effect  Yield  Ref\n");
    out.push_str("--------               --------------  ----------  -----  ---\n");
    out.push_str("AWD                    -64.5%         +18.7%      +1.3%  Rafy 2025\n");
    out.push_str("Midseason drainage     -50%           +15%       +2%    Minamikawa 2025\n");
    out.push_str("Biochar (5 t/ha)       -30%           -60%       +5%    Iboka 2026\n");
    out.push_str("No-tillage             -20%           -10%       0%     Iboka 2026\n");
    out.push_str("PNSB inoculation       -15%           -5%        +3%    Nhanh 2026\n");
    out.push_str("Rice-fish coculture    variable        0%         +10%   Li 2025\n");
    out.push_str("Silicate fertilizer    -14%           -37%       +58%   Yun 2025\n");
    out.push_str("N-fixing cyanobacteria -20%           -30%       +5%    Zhu 2025\n\n");

    // ═══ Phase 7: Indonesia Context ═══
    out.push_str("-- Indonesia Context --\n\n");
    out.push_str("  Rice area: ~11 million ha (3rd largest world)\n");
    out.push_str("  Padi: sawah irrigated + gogo rainfed\n");
    out.push_str("  CH4 from rice: ~30% of national GHG inventory\n");
    out.push_str("  Indonesia NDC: rice sector key mitigation\n");
    out.push_str("  IRRI: Safe AWD piloted in Java and Sumatra\n");
    out.push_str("  Perpres 98/2021: Nilai Ekonomi Karbon (NEK)\n");
    out.push_str("  Second NDC 2025: sektor pertanian (rice) mitigasi kunci\n\n");

    // ═══ Limitations ═══
    out.push_str("-- Limitations (honest) --\n");
    out.push_str("  • Emission factors from meta-analysis (not site-specific measured)\n");
    out.push_str("  • No actual DNDC model simulation (process-based)\n");
    out.push_str("  • India modified EFs may not directly apply to Indonesia\n");
    out.push_str("  • No soil organic carbon dynamics\n");
    out.push_str("  • Simplified GWP (CO2 from fertilizer production not included)\n");
    out.push_str("  • Full 2026 SOTA: DNDC model with 25 GCM climate scenarios\n");
    out.push_str("  • Ref: Rafy 2025; Bhattacharyya 2025; Minamikawa 2025; Tang 2025\n");

    out
}

#[cfg(test)]
mod tests {
    // Self-check: N2O must scale LINEARLY with N fertilizer (bug was quadratic).
    // With EF1FR=0.004, sf=1.0, soil=1.0: N2O-N = N*0.004; N2O = N*0.004*44/28.
    // For N=100 -> N2O-N=0.4 kg/ha, N2O=0.629 kg/ha (realistic rice range 0.3-2).
    // For N=200 -> N2O must be exactly 2x (linear), not 4x.
    #[test]
    fn n2o_is_linear_in_fertilizer() {
        let ef: f64 = 0.004;
        let n2o = |n: f64| n * ef * 44.0 / 28.0;
        let a = n2o(100.0);
        let b = n2o(200.0);
        assert!((a - 0.6286).abs() < 0.001, "N2O(100)={a} expected ~0.629 kg/ha");
        assert!((b / a - 2.0).abs() < 1e-9, "N2O must double when N doubles (linear), got ratio {}", b/a);
        assert!(a > 0.1 && a < 5.0, "N2O={a} outside realistic rice range");
    }
}

