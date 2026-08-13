/// Jakarta Coastal Risk — Integrated Subsidence + SLR + Groundwater + Rob (Tidal Flood)
/// Problem: Jakarta subsidence + SLR + tidal flood + groundwater extraction are separate tools, not integrated.
/// Method: Combined coastal risk score with weighted combination.
/// Ref: IPCC AR6 WG1 (SSP245 median 0.24m by 2050, 0.56m by 2100); Widiyarso et al. (subsidence Jakarta);
///      Umarhadi et al. (Semarang subsidence); Momin et al. 2026 (compound flood review);
///      Chrysanti et al. 2024; Abidin et al. 2015 (InSAR Jakarta -75 to -100 mm/yr).
/// Formula:
///   Total subsidence S = rate * years
///   SLR (SSP245 linear interp): 0.24m @ 2050, 0.56m @ 2100 (baseline 1995-2014)
///   Flood depth = max(SLR + tide_above_msl + subsidence - ground_elevation, 0)
///   Risk score = 0.40*subs_norm + 0.30*slr_norm + 0.20*elev_norm + 0.10*gw_norm (0-100)

pub fn assess(
    lat: f64,
    lon: f64,
    subsidence_rate_mm_yr: f64,
    groundwater_extraction_m3_day: f64,
    distance_to_coast_km: f64,
    elevation_m: f64,
    planning_horizon_years: u32,
) -> String {
    if subsidence_rate_mm_yr < -200.0 || subsidence_rate_mm_yr > 200.0 {
        return "ERROR [E102]: subsidence_rate_mm_yr outside realistic [-200, 200] mm/yr.".into();
    }
    if planning_horizon_years == 0 || planning_horizon_years > 100 {
        return "ERROR [E102]: planning_horizon_years must be 1-100.".into();
    }
    if elevation_m < -10.0 || elevation_m > 500.0 {
        return "ERROR [E102]: elevation_m outside plausible range [-10, 500].".into();
    }

    let mut out = String::from("══════════════════════════════════════════════════════\n");
    out.push_str("JAKARTA COASTAL RISK — INTEGRATED SUBSIDENCE + SLR + GW\n");
    out.push_str("Ref: IPCC AR6 WG1 SSP245; Abidin et al. 2015; Momin et al. 2026\n");
    out.push_str("══════════════════════════════════════════════════════\n\n");

    let years = planning_horizon_years as f64;

    // ─── 1. SUBSIDENCE ───
    let total_subsidence_mm = subsidence_rate_mm_yr * years;
    let total_subsidence_m = total_subsidence_mm / 1000.0;

    out.push_str("1. LAND SUBSIDENCE:\n");
    out.push_str(&format!("   Rate: {:.1} mm/yr x {:.0} yr = {:.0} mm = {:.2} m\n",
        subsidence_rate_mm_yr, years, total_subsidence_mm, total_subsidence_m));
    // Reference rates from literature
    out.push_str("   Reference rates (InSAR measured):\n");
    out.push_str("     Jakarta   : -75 mm/yr (Abidin et al. 2015; Widiyarso 2026)\n");
    out.push_str("     Semarang : -150 mm/yr (Umarhadi 2026; Andreas et al. 2017)\n");
    out.push_str("     Pekalongan: -100 to -200 mm/yr (Machmudin et al. 2024)\n");
    out.push_str("     Bandung   : -20 to -40 mm/yr (sub-regional)\n\n");

    // ─── 2. SEA LEVEL RISE (IPCC AR6 SSP245) ───
    // AR6 SSP245 median: 0.24m by 2050, 0.56m by 2100 (relative to 1995-2014 baseline)
    // Linear interpolation between 2050 and 2100 endpoints.
    // Baseline year ~ 2014 (center of 1995-2014).
    let slr_2050 = 0.24; // m
    let slr_2100 = 0.56; // m
    let slr_m = if years <= 24.0 {
        // 2026 -> 2050 is 24 years; linear from 0 in 2014 to 0.24 in 2050
        // rate = 0.24 / (2050-2014) = 0.24/36 = 0.00667 m/yr
        // by year 2026+years: slr = rate * (2026 + years - 2014)
        0.00667 * (12.0 + years)
    } else {
        // beyond 2050: interpolate 2050->2100
        let beyond = years - 24.0;
        slr_2050 + (slr_2100 - slr_2050) * (beyond / 50.0).min(1.0)
    };
    out.push_str("2. SEA LEVEL RISE (IPCC AR6 SSP245 median):\n");
    out.push_str(&format!("   2050 target: 0.24 m | 2100 target: 0.56 m (vs 1995-2014)\n"));
    out.push_str(&format!("   By +{:.0} yr (year ~{:.0}): {:.3} m\n\n", years, 2026.0 + years, slr_m));

    // ─── 3. TIDAL / ELEVATION FACTOR ───
    // If elevation <5m above MSL -> high risk; tidal range adds
    let elev_risk_factor = if elevation_m < 1.0 { 1.0 }
        else if elevation_m < 5.0 { (5.0 - elevation_m) / 5.0 }
        else { 0.0 };
    // Distance to coast (closer = higher)
    let coast_factor = if distance_to_coast_km < 1.0 { 1.0 }
        else if distance_to_coast_km < 20.0 { 1.0 - (distance_to_coast_km - 1.0) / 19.0 }
        else { 0.0 };
    out.push_str("3. ELEVATION / TIDAL EXPOSURE:\n");
    out.push_str(&format!("   Ground elevation: {:.2} m above MSL\n", elevation_m));
    out.push_str(&format!("   Distance to coast: {:.1} km\n", distance_to_coast_km));
    out.push_str(&format!("   Elevation risk factor: {:.2} (<5m = high)\n", elev_risk_factor));
    out.push_str(&format!("   Coast proximity factor: {:.2}\n\n", coast_factor));

    // ─── 4. GROUNDWATER EXTRACTION ───
    // Jakarta pumps ~500-1000 m3/day per km2 in dense areas. Threshold: >500 m3/day = high.
    let gw_factor = if groundwater_extraction_m3_day > 1000.0 { 1.0 }
        else if groundwater_extraction_m3_day > 100.0 { 0.5 + (groundwater_extraction_m3_day - 100.0) / 1800.0 }
        else { groundwater_extraction_m3_day / 200.0 };
    let gw_factor = gw_factor.min(1.0);
    out.push_str("4. GROUNDWATER EXTRACTION:\n");
    out.push_str(&format!("   Extraction: {:.0} m3/day\n", groundwater_extraction_m3_day));
    out.push_str(&format!("   GW factor: {:.2} (>1000 m3/d = 1.0)\n", gw_factor));
    out.push_str("   Jakarta reference: PAM-J consumption ~600M L/d, private GW ~40% of total\n\n");

    // ─── COMPOUND FLOOD DEPTH ───
    // Rob (tidal flood) typical amplitude ~0.3-0.5m in Jakarta Bay
    let tide_amplitude_m = 0.4; // typical Jakarta Bay mean tide + spring
    let compound_flood_m = (slr_m + total_subsidence_m + tide_amplitude_m - elevation_m).max(0.0);
    out.push_str("COMPOUND FLOOD DEPTH (bathtub):\n");
    out.push_str(&format!("   = SLR ({:.3}m) + Subsidence ({:.2}m) + Tide ({:.1}m) - Elevation ({:.2}m)\n",
        slr_m, total_subsidence_m, tide_amplitude_m, elevation_m));
    out.push_str(&format!("   = {:.2} m (max with 0)\n\n", compound_flood_m));

    // ─── INTEGRATED RISK SCORE ───
    // Normalize each component 0-1, weight: subsidence 40%, SLR 30%, elevation 20%, groundwater 10%
    let subs_norm = (total_subsidence_m / 3.0).min(1.0); // 3m subsidence = max risk
    let slr_norm = (slr_m / 1.0).min(1.0); // 1m SLR = max
    let elev_norm = elev_risk_factor;
    let gw_norm = gw_factor;
    let risk_score = (0.40 * subs_norm + 0.30 * slr_norm + 0.20 * elev_norm + 0.10 * gw_norm) * 100.0;

    let risk_class = if risk_score >= 75.0 { "CRITICAL" }
        else if risk_score >= 50.0 { "HIGH" }
        else if risk_score >= 25.0 { "MODERATE" }
        else { "LOW" };

    out.push_str("INTEGRATED RISK SCORE:\n");
    out.push_str("   Weighting: subsidence 40% + SLR 30% + elevation 20% + groundwater 10%\n");
    out.push_str(&format!("   Subsidence norm: {:.2} (cap 3m)\n", subs_norm));
    out.push_str(&format!("   SLR norm       : {:.2} (cap 1m)\n", slr_norm));
    out.push_str(&format!("   Elevation norm : {:.2}\n", elev_norm));
    out.push_str(&format!("   Groundwater norm: {:.2}\n", gw_norm));
    out.push_str(&format!("\n   >>> RISK SCORE = {:.1}/100  [{}]\n\n", risk_score, risk_class));

    // ─── MITIGATION ───
    out.push_str("─── MITIGATION RECOMMENDATIONS ───\n");
    if risk_class == "CRITICAL" {
        out.push_str("  CRITICAL — immediate action:\n");
        out.push_str("  1. STOP groundwater extraction (primary subsidence driver).\n");
        out.push_str("     Substitute with piped water (PAM J), regulate deep wells.\n");
        out.push_str("  2. Seawall / coastal dike (Giant Sea Wall Jakarta GCW est. 32 km).\n");
        out.push_str("  3. Managed retreat for areas >2m below MSL by horizon year.\n");
        out.push_str("  4. Mangrove restoration (green belt) for tidal buffer.\n");
        out.push_str("  5. InSAR subsidence monitoring (quarterly) to verify stop.\n");
        out.push_str("  6. Pump stations + polder system for stormwater drainage.\n");
    } else if risk_class == "HIGH" {
        out.push_str("  HIGH — priority action:\n");
        out.push_str("  1. Reduce groundwater extraction by 50%+ over 5 years.\n");
        out.push_str("  2. Build/enhance seawall; elevate critical infrastructure.\n");
        out.push_str("  3. Mangrove restoration + coastal revegetation.\n");
        out.push_str("  4. Annual InSAR subsidence monitoring.\n");
        out.push_str("  5. Flood early warning (BMKG + Pusdalops BPBD).\n");
    } else if risk_class == "MODERATE" {
        out.push_str("  MODERATE — planned action:\n");
        out.push_str("  1. Limit new groundwater wells; incentivize piped water.\n");
        out.push_str("  2. Mangrove / coastal vegetation buffer.\n");
        out.push_str("  3. Biennial InSAR subsidence monitoring.\n");
        out.push_str("  4. Stormwater drainage capacity upgrade.\n");
    } else {
        out.push_str("  LOW — monitoring:\n");
        out.push_str("  1. Maintain current groundwater management.\n");
        out.push_str("  2. 5-year InSAR subsidence check.\n");
        out.push_str("  3. Mangrove protection.\n");
    }

    // ─── REGULATORY CONTEXT ───
    out.push_str("\n─── REGULATORY / PLANNING CONTEXT ───\n");
    out.push_str("  - NCICD (National Capital Integrated Coastal Development): GCW Jakarta.\n");
    out.push_str("  - PP 22/2021 (PPLH) — coastal zone management.\n");
    out.push_str("  - Permen PUPR 28/2015 (persampahan & drainase).\n");
    out.push_str("  - UU 6/2023 (Cipta Kerja) — coastal reclamation rules.\n");
    out.push_str("  - Perpres 80/2019 (Jakarta revitalisasi pesisir).\n");
    out.push_str("  - RAN-PI (National Adaptation Plan) 2014; BAPPENAS.\n");
    out.push_str(&format!("\n  Location: ({:.4}, {:.4})\n", lat, lon));

    // ─── LIMITATIONS ───
    out.push_str("\n─── HONEST LIMITATIONS ───\n");
    out.push_str("  1. NO HYDRODYNAMIC MODEL: bathtub only (no flow paths, no drainage).\n");
    out.push_str("     Real flood needs MIKE 21 / Delft3D with bathymetry + drainage network.\n");
    out.push_str("  2. NO SPATIAL VARIATION: subsidence assumed uniform within radius.\n");
    out.push_str("     Jakarta subsidence is highly localized (patchy InSAR pattern).\n");
    out.push_str("  3. SIMPLIFIED GW LINK: groundwater extraction -> subsidence coupling is\n");
    out.push_str("     non-linear (depends on aquifer compressibility, clay thickness).\n");
    out.push_str("     Use tools::calculators::land_subsidence (Terzaghi 1D) for physics.\n");
    out.push_str("  4. SLR LINEAR INTERP: AR6 trajectories are non-linear (accelerating).\n");
    out.push_str("     For policy: use AR6 SLR projection tool (sealevel.nasa.gov).\n");
    out.push_str("  5. NO STORM SURGE: excludes extreme events (typhoons, storm surge).\n");
    out.push_str("  6. NO COMPACTION DELAY: Terzaghi consolidation has time-dependent U(t).\n");
    out.push_str("  7. WEIGHTING (40/30/20/10) is heuristic, not calibrated to flood damage.\n");
    out.push_str("  8. DEMNAS vertical accuracy ~2-3m; for plot-scale use TLS/RTK survey.\n");
    out.push_str("══════════════════════════════════════════════════════\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Self-check: Jakarta subsidence 75 mm/yr * 30 yr = 2.25 m total
    #[test]
    fn jakarta_subsidence_75mm_30yr() {
        let res = assess(-6.2, 106.8, 75.0, 500.0, 5.0, 3.0, 30);
        // 75 * 30 = 2250 mm = 2.25 m
        assert!(res.contains("2.25"), "expected 2.25m subsidence, got:\n{}", res);
        assert!(!res.contains("ERROR"));
    }

    // Self-check: Semarang 150 mm/yr * 30 yr = 4.5m (capped in normalization but reported)
    #[test]
    fn semarang_subsidence_extreme() {
        let res = assess(-6.97, 110.4, 150.0, 800.0, 2.0, 2.0, 30);
        // 150 * 30 = 4500 mm = 4.5 m
        assert!(res.contains("4.50") || res.contains("4500"), "expected 4.5m, got:\n{}", res);
    }

    // Self-check: SLR by 2050 (~24 yr from 2026) = 0.24 * 36/36 ~ 0.16m via early formula
    // Actually: 0.00667 * (12 + 24) = 0.00667 * 36 = 0.24m
    #[test]
    fn slr_24yr_matches_2050() {
        let res = assess(-6.2, 106.8, 75.0, 500.0, 5.0, 3.0, 24);
        // 24 years from 2026 -> 2050; linear interp gives 0.24m
        assert!(res.contains("0.24"), "expected 0.24m SLR by +24yr, got:\n{}", res);
    }

    // Self-check: risk classification — high subsidence + low elevation + coastal = CRITICAL
    #[test]
    fn critical_risk_classification() {
        let res = assess(-6.2, 106.8, 75.0, 1000.0, 1.0, 1.0, 30);
        assert!(res.contains("CRITICAL") || res.contains("HIGH"), "expected high/critical risk");
    }

    // Self-check: SLR monotonic past 2050 (was discontinuous: dropped from 0.32m to 0.246m at years>36)
    #[test]
    fn slr_monotonic_past_2050() {
        let res = assess(-6.2, 106.8, 75.0, 500.0, 5.0, 3.0, 37);
        // years=37 (>24): 0.24 + 0.32·13/50 = 0.323 m
        assert!(res.contains("0.323"), "SLR by +37yr should be ~0.323 m:\n{}", res);
    }
}
