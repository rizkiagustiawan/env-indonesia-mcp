/// Problem-Solution-Impact Orchestrator (End-to-End Workflow)
/// Problem: Tools are separate calculators. User wants automated
///          problem -> solution -> impact chain in one structured report.
/// Method: SYNTHESIS meta-orchestrator. Does NOT re-implement sub-tools;
///         instead provides a structured framework with inline simplified
///         diagnostic models and references to the real sub-tools by name.
///
/// Phases:
///   1. DIAGNOSIS  - identify key parameters per problem type, run a simplified
///                   inline diagnostic (flood depth, PM conc, BOD/DO, recession,
///                   AMD risk, burned area). Reference the real sub-tool.
///   2. SOLUTION   - recommend interventions based on diagnosis + severity.
///   3. IMPACT     - quantify environmental + health + economic impact,
///                   including a simplified HIA (PM2.5 -> DALYs) and
///                   externality + restoration-cost estimate.
///
/// NOTE (honest): This is a FRAMEWORK, not a high-fidelity model.
///   - Some parameters use defaults when not supplied.
///   - Inline diagnostic models are simplified; for accuracy, call the named sub-tool.
///   - Health impact uses the same CRF as health_impact_assessment.rs (RR=1.0615/10ug).
///
/// Problem types: flood, fire, pollution_river, pollution_air, coastal_erosion, mining_impact.

const IDR_PER_USD: f64 = 16_500.0;
const INDONESIA_BASELINE_MORTALITY_PER_100K: f64 = 753.0;

/// Thousands-grouped integer string (e.g. 1234567 -> "1,234,567").
fn grp(x: f64) -> String {
    let n = x.round() as i64;
    let s = n.to_string();
    let bytes = s.as_bytes();
    let neg = bytes.first() == Some(&b'-');
    let digits = if neg { &bytes[1..] } else { bytes };
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    let len = digits.len();
    for (i, b) in digits.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

fn severity_mult(severity: &str) -> f64 {
    match severity.to_lowercase().as_str() {
        "low" => 0.5,
        "moderate" => 1.0,
        "high" => 2.0,
        _ => 1.0,
    }
}

/// Simplified PM2.5 HIA (mirrors health_impact_assessment.rs CRF).
fn hia_pm25(conc: f64, pop: f64, years: f64) -> (f64, f64, f64) {
    let background = 5.0; // WHO annual guideline
    let delta = (conc - background).max(0.0);
    let rr = 1.0615_f64.powf(delta / 10.0);
    let af = (rr - 1.0) / rr.max(1e-12);
    let deaths = pop * (INDONESIA_BASELINE_MORTALITY_PER_100K / 100_000.0) * af * years;
    let dalys = deaths * 12.0;
    let cost_usd = dalys * 75_000.0; // mid WHO valuation
    (deaths, dalys, cost_usd)
}

pub fn orchestrate(
    problem_type: &str,
    location_name: &str,
    lat: f64,
    lon: f64,
    area_ha: f64,
    severity: &str,
) -> String {
    if area_ha <= 0.0 {
        return "ERROR [E102]: area_ha must be > 0.".into();
    }
    let sev = severity_mult(severity);

    let mut out = String::new();
    out.push_str("===============================================================\n");
    out.push_str("  PROBLEM -> SOLUTION -> IMPACT  (End-to-End Orchestrator)\n");
    out.push_str("===============================================================\n");
    out.push_str("NOTE: This is a SYNTHESIS framework. Inline models are simplified.\n");
    out.push_str("      For high accuracy, call the named sub-tool directly.\n\n");

    out.push_str("PROBLEM DEFINITION:\n");
    out.push_str(&format!("  Type       : {}\n", problem_type.to_uppercase()));
    out.push_str(&format!(
        "  Location   : {} ({:.4}, {:.4})\n",
        location_name, lat, lon
    ));
    out.push_str(&format!("  Area       : {:.1} ha\n", area_ha));
    out.push_str(&format!("  Severity   : {} ({:.1}x)\n\n", severity, sev));

    match problem_type.to_lowercase().as_str() {
        "flood" => flood_workflow(&mut out, area_ha, sev),
        "fire" => fire_workflow(&mut out, area_ha, sev),
        "pollution_river" => river_workflow(&mut out, area_ha, sev),
        "pollution_air" => air_workflow(&mut out, area_ha, sev),
        "coastal_erosion" => erosion_workflow(&mut out, area_ha, sev),
        "mining_impact" => mining_workflow(&mut out, area_ha, sev),
        other => {
            return format!(
                "ERROR: Unknown problem_type '{}'. Supported: flood, fire, pollution_river, \
                 pollution_air, coastal_erosion, mining_impact.",
                other
            )
        }
    }

    out.push_str("\n===============================================================\n");
    out.push_str("ORCHESTRATOR LIMITATIONS (honest assessment):\n");
    out.push_str("  1. SYNTHESIS tool - inline diagnostic models are simplified; for accuracy\n");
    out.push_str("     call the named sub-tool (gaussian_plume, river_quality, bruun_rule, etc).\n");
    out.push_str("  2. Not all relevant sub-tools are invoked (some are conceptual references).\n");
    out.push_str("  3. Population exposure is estimated from area x density default; replace with\n");
    out.push_str("     real population data (BPS/satu_data) for accurate health impact.\n");
    out.push_str("  4. Economic estimates combine damage (externality) + restoration (cost) -\n");
    out.push_str("     avoid double-counting (restoration may already avert part of damage).\n");
    out.push_str("  5. Severity multiplier is heuristic, not calibrated per problem type.\n");
    out.push_str("  6. No temporal dynamics (single-snapshot); real events evolve over time.\n");
    out.push_str("  7. Recommendations are generic - site-specific design requires AMDAL/EIA.\n");
    out.push_str("===============================================================\n");
    out
}

// =========================== FLOOD ===========================
fn flood_workflow(out: &mut String, area_ha: f64, sev: f64) {
    out.push_str("===============================================================\n");
    out.push_str("PHASE 1 - DIAGNOSIS (FLOOD)\n");
    out.push_str("===============================================================\n");
    let flood_depth_m = 0.3 * sev; // 0.15 (low) -> 0.6 (high) m
    let affected_area_ha = area_ha * (0.6 + 0.2 * sev).min(1.0);
    let pop_density = 5_000.0; // default urban-coastal Indonesia
    let pop_affected = affected_area_ha * 10_000.0 / 100.0 * pop_density;
    out.push_str("Inline model: flood depth = 0.3 * severity_mult (heuristic)\n");
    out.push_str(&format!("  Estimated flood depth : {:.2} m\n", flood_depth_m));
    out.push_str(&format!(
        "  Affected area         : {:.1} ha ({:.0}% of input)\n",
        affected_area_ha,
        affected_area_ha / area_ha * 100.0
    ));
    out.push_str(&format!(
        "  Pop. affected (est.)  : {} (density {}/km2)\n",
        grp(pop_affected),
        grp(pop_density)
    ));
    out.push_str("  Real tool : tidal_flood_compound (SLR+subsidence+tide), swe_flood (2D hydro)\n\n");

    out.push_str("PHASE 2 - SOLUTION (recommended interventions)\n");
    out.push_str("  1. Seawall/dike (if coastal) - height = flood_depth + 0.5m freeboard\n");
    out.push_str("  2. Polder + pump system (urban, e.g. Jakarta, Pekalongan)\n");
    out.push_str("  3. Managed retreat / polder kampung (Demak-style)\n");
    out.push_str("  4. STOP groundwater extraction (largest subsidence driver in Jakarta)\n");
    out.push_str("  5. Mangrove belt restoration (coastal, green belt 200m)\n");
    out.push_str("  6. Early warning system (IBF/Waze flood alert)\n");
    out.push_str("  7. SUDS / bioretention for pluvial component\n\n");

    out.push_str("PHASE 3 - IMPACT\n");
    out.push_str("  Environmental:\n");
    out.push_str(&format!("    - Area inundated      : {:.1} ha\n", affected_area_ha));
    out.push_str(&format!(
        "    - Flood depth         : {:.2} m ({})\n",
        flood_depth_m,
        if flood_depth_m > 0.5 {
            "HIGH - structural damage likely"
        } else {
            "MODERATE"
        }
    ));
    out.push_str("  Health: flooding primarily affects morbidity (drowning, leptospirosis,\n");
    out.push_str("    diarrhea). No direct PM/DALY model here - use sanitation_impact tool.\n");
    let damage_usd = affected_area_ha * 15_000.0 * sev;
    out.push_str("  Economic (damage estimate):\n");
    out.push_str(&format!(
        "    - Direct damage (est.): ${} (Rp{})\n",
        grp(damage_usd),
        grp(damage_usd * IDR_PER_USD)
    ));
    out.push_str("    - Restoration (polder/mangrove): use restoration_cost tool.\n");
    out.push_str("    - See also: land_subsidence (Jakarta -10 to -25cm/yr).\n");
}

// =========================== FIRE ===========================
fn fire_workflow(out: &mut String, area_ha: f64, sev: f64) {
    out.push_str("===============================================================\n");
    out.push_str("PHASE 1 - DIAGNOSIS (FIRE)\n");
    out.push_str("===============================================================\n");
    let burned_area_ha = area_ha * (0.4 + 0.2 * sev).min(1.0);
    let pm25_emission_t = burned_area_ha * 1_400.0 / 1_000.0;
    let pop_density = 200.0; // rural peatland
    let pop_affected = burned_area_ha * 10_000.0 / 100.0 * pop_density;
    out.push_str("Inline model: burned_area = input * (0.4 + 0.2*sev); EF peat PM2.5 = 1,400 kg/ha\n");
    out.push_str(&format!("  Estimated burned area : {:.1} ha\n", burned_area_ha));
    out.push_str(&format!(
        "  PM2.5 emitted         : {:.1} t (peat fire EF 1,400 kg/ha)\n",
        pm25_emission_t
    ));
    out.push_str(&format!(
        "  Pop. exposed (est.)   : {} (rural peat, density {}/km2)\n",
        grp(pop_affected),
        grp(pop_density)
    ));
    out.push_str("  Real tools: fire_spread (advanced_physics), firms (hotspot), burned_area (sat)\n\n");

    out.push_str("PHASE 2 - SOLUTION\n");
    out.push_str("  1. Fire breaks (15-20m wide cleared strips)\n");
    out.push_str("  2. Canal blocking + AWD (air-static water level 40cm) for peat\n");
    out.push_str("  3. Early detection (FIRMS VIIRS/SNP, community patrols)\n");
    out.push_str("  4. Community firefighting (Masyarakat Peduli Api)\n");
    out.push_str("  5. Rewetting (BRG standard) + revegetation\n");
    out.push_str("  6. Law enforcement (Perpres 3/2016 moratorium, UU 32/2009)\n\n");

    out.push_str("PHASE 3 - IMPACT\n");
    out.push_str("  Environmental:\n");
    out.push_str(&format!("    - Burned area        : {:.1} ha\n", burned_area_ha));
    out.push_str(&format!(
        "    - CO2 emitted (peat) : {} t (EF ~400 tCO2/ha peat)\n",
        grp(burned_area_ha * 400.0)
    ));
    out.push_str(&format!(
        "    - PM2.5 emitted      : {:.1} t\n",
        pm25_emission_t
    ));
    out.push_str("  Health (PM2.5 downwind - simplified, assume 30% reaches populated zone):\n");
    let pop_health = pop_affected;
    let assumed_pm25_conc = 150.0 * sev;
    let (deaths, dalys, cost_usd) = hia_pm25(assumed_pm25_conc, pop_health, 1.0);
    out.push_str(&format!(
        "    - Assumed PM2.5 conc : {:.0} ug/m3 (haze event)\n",
        assumed_pm25_conc
    ));
    out.push_str(&format!("    - Attributable deaths: {}\n", grp(deaths)));
    out.push_str(&format!("    - DALYs              : {}\n", grp(dalys)));
    out.push_str(&format!("    - Health cost        : ${}\n", grp(cost_usd)));
    let restoration_usd = burned_area_ha * 80_000_000.0 / IDR_PER_USD * 1.5;
    out.push_str("  Economic:\n");
    out.push_str(&format!(
        "    - Restoration (peat, moderate): ${}\n",
        grp(restoration_usd)
    ));
    out.push_str(&format!(
        "    - Carbon loss value  : ${} (@ Rp465k/tCO2e)\n",
        grp(burned_area_ha * 400.0 * 465_000.0 / IDR_PER_USD)
    ));
}

// ===================== RIVER POLLUTION =====================
fn river_workflow(out: &mut String, area_ha: f64, sev: f64) {
    out.push_str("===============================================================\n");
    out.push_str("PHASE 1 - DIAGNOSIS (RIVER POLLUTION)\n");
    out.push_str("===============================================================\n");
    let bod_mg_l = 5.0 + 5.0 * sev;
    let do_mg_l = (8.0 - 2.0 * sev).max(1.0);
    let pp_bod_limit = 6.0; // PP 22/2021 Class III (fishery)
    let pp_do_min = 4.0;
    out.push_str("Inline model: BOD = 5 + 5*sev; DO = max(8 - 2*sev, 1)\n");
    out.push_str(&format!(
        "  Estimated BOD         : {:.1} mg/L (PP 22/2021 limit: {:.0})\n",
        bod_mg_l, pp_bod_limit
    ));
    out.push_str(&format!(
        "  Estimated DO          : {:.1} mg/L (PP 22/2021 min: {:.0})\n",
        do_mg_l, pp_do_min
    ));
    out.push_str(&format!(
        "  BAKU MUTU             : BOD {} | DO {}\n",
        if bod_mg_l > pp_bod_limit {
            "EXCEEDED"
        } else {
            "OK"
        },
        if do_mg_l < pp_do_min {
            "EXCEEDED"
        } else {
            "OK"
        }
    ));
    out.push_str("  Real tools: river_quality (Streeter-Phelps), streeter_phelps, do_saturation\n\n");

    out.push_str("PHASE 2 - SOLUTION\n");
    out.push_str("  1. WWTP upgrade (IPAL komunal / communal WWTP) - BOD removal 85-95%\n");
    out.push_str("  2. Source apportionment (industry vs domestic vs agriculture)\n");
    out.push_str("  3. Riparian buffer zone (10-30m, PP 22/2021 sempadan sungai)\n");
    out.push_str("  4. Industry compliance audit (PP 22/2021 baku mutu air limbah)\n");
    out.push_str("  5. Bioremediation (constructed wetland, biofilter)\n");
    out.push_str("  6. Real-time monitoring (IoT sonde BOD/DO/pH)\n\n");

    out.push_str("PHASE 3 - IMPACT\n");
    out.push_str("  Environmental:\n");
    out.push_str(&format!(
        "    - BOD exceedance     : {:.1}x PP 22/2021 limit\n",
        bod_mg_l / pp_bod_limit
    ));
    out.push_str(&format!(
        "    - DO deficit         : {:.1} mg/L below min\n",
        (pp_do_min - do_mg_l).max(0.0)
    ));
    let reach_km = area_ha * 10_000.0 / 50.0 / 1_000.0;
    out.push_str(&format!(
        "    - Affected reach     : ~{:.0} km (est. from area, 50m wide channel)\n",
        reach_km
    ));
    out.push_str("  Health: river pollution -> waterborne disease (diarrhea, skin).\n");
    out.push_str("    Use sanitation_impact tool for quantitative health burden.\n");
    let restoration_usd = reach_km * 1_250_000_000.0 / IDR_PER_USD * sev;
    out.push_str("  Economic:\n");
    out.push_str(&format!(
        "    - Restoration (river, per km): ${}\n",
        grp(restoration_usd)
    ));
    out.push_str(&format!(
        "    - Fisheries loss (est.): ${}\n",
        grp(area_ha * 500.0 * sev)
    ));
    out.push_str("    - Use restoration_cost(river) for detailed estimate.\n");
}

// ===================== AIR POLLUTION =====================
fn air_workflow(out: &mut String, area_ha: f64, sev: f64) {
    out.push_str("===============================================================\n");
    out.push_str("PHASE 1 - DIAGNOSIS (AIR POLLUTION)\n");
    out.push_str("===============================================================\n");
    let pm25_conc = 20.0 + 20.0 * sev; // 30 (low) -> 60 (high) ug/m3 annual
    let pp22_limit = 15.0; // PP 22/2021 annual PM2.5
    let who_limit = 5.0;
    out.push_str("Inline model: PM2.5 = 20 + 20*sev (ug/m3 annual)\n");
    out.push_str(&format!(
        "  Estimated PM2.5       : {:.1} ug/m3 annual\n",
        pm25_conc
    ));
    out.push_str(&format!(
        "  vs WHO 2021 ({:.0})     : {:.1}x exceeded\n",
        who_limit,
        pm25_conc / who_limit
    ));
    out.push_str(&format!(
        "  vs PP 22/2021 ({:.0})   : {}\n",
        pp22_limit,
        if pm25_conc > pp22_limit {
            "EXCEEDED"
        } else {
            "OK"
        }
    ));
    out.push_str("  Real tools: gaussian_plume (point source), waqi (ambient monitor),\n");
    out.push_str("              health_impact_assessment (full HIA)\n\n");

    out.push_str("PHASE 2 - SOLUTION\n");
    out.push_str("  1. Scrubber (FGD/wet) for SO2 - removal 90-95%\n");
    out.push_str("  2. Baghouse / ESP for PM - removal 99%\n");
    out.push_str("  3. Stack height increase (Gaussian: conc ~ 1/H^2)\n");
    out.push_str("  4. Fuel switch (HFO->LNG/biomass) - SO2 ~80% reduction\n");
    out.push_str("  5. Vehicle emission inspection & Euro IV fuel\n");
    out.push_str("  6. Green buffer / urban forest (PM deposition)\n\n");

    out.push_str("PHASE 3 - IMPACT\n");
    out.push_str("  Environmental:\n");
    out.push_str(&format!(
        "    - PM2.5 level       : {:.1} ug/m3 ({:.1}x WHO)\n",
        pm25_conc,
        pm25_conc / who_limit
    ));
    out.push_str(&format!(
        "    - Exceedance status : {}\n",
        if pm25_conc > pp22_limit {
            "PP 22/2021 EXCEEDED"
        } else {
            "OK"
        }
    ));
    let pop_density = 8_000.0;
    let pop_exposed = area_ha * 10_000.0 / 100.0 * pop_density;
    out.push_str("  Health (full HIA inline, CRF RR=1.0615/10ug):\n");
    let (deaths, dalys, cost_usd) = hia_pm25(pm25_conc, pop_exposed, 1.0);
    out.push_str(&format!(
        "    - Pop. exposed      : {} (density {}/km2)\n",
        grp(pop_exposed),
        grp(pop_density)
    ));
    out.push_str(&format!("    - Attributable deaths: {}/yr\n", grp(deaths)));
    out.push_str(&format!("    - DALYs             : {}\n", grp(dalys)));
    out.push_str(&format!(
        "    - Economic cost     : ${} (Rp{})\n",
        grp(cost_usd),
        grp(cost_usd * IDR_PER_USD)
    ));
    out.push_str("  Economic:\n");
    out.push_str("    - Externality (health) dominates; restoration not applicable for air.\n");
    out.push_str("    - Use health_impact_assessment for sensitivity (valuation, baseline).\n");
}

// ===================== COASTAL EROSION =====================
fn erosion_workflow(out: &mut String, _area_ha: f64, sev: f64) {
    out.push_str("===============================================================\n");
    out.push_str("PHASE 1 - DIAGNOSIS (COASTAL EROSION)\n");
    out.push_str("===============================================================\n");
    let slr_m = 0.3 * sev;
    let recession_m = 50.0 * sev;
    let risk_class = if recession_m > 80.0 {
        "CRITICAL"
    } else if recession_m > 40.0 {
        "HIGH"
    } else if recession_m > 15.0 {
        "MODERATE"
    } else {
        "LOW"
    };
    out.push_str("Inline model: Bruun R = SLR * L / (B + h*); simplified recession = 50*sev m\n");
    out.push_str(&format!("  SLR scenario         : {:.1} m (SSP2-4.5-like)\n", slr_m));
    out.push_str(&format!("  Shoreline recession  : {:.0} m by 2050\n", recession_m));
    out.push_str(&format!("  Risk class           : {}\n", risk_class));
    out.push_str("  Real tools: bruun_rule (ocean_modeling), coastal_erosion (Pantura combined),\n");
    out.push_str("              sea_level_rise, land_subsidence\n\n");

    out.push_str("PHASE 2 - SOLUTION\n");
    out.push_str("  1. Mangrove restoration (green belt, 200m wide) - preferred (nature-based)\n");
    out.push_str("  2. Groin field (perpendicular structures, trap sediment)\n");
    out.push_str("  3. Beach nourishment (sand replenishment, temporary)\n");
    out.push_str("  4. Revetment/seawall (hard, last resort - causes downdrift erosion)\n");
    out.push_str("  5. Managed retreat (relocate, Demak/Pekalongan)\n");
    out.push_str("  6. Stop sand mining (largest anthropogenic driver on Pantura)\n\n");

    out.push_str("PHASE 3 - IMPACT\n");
    out.push_str("  Environmental:\n");
    out.push_str(&format!("    - Shoreline loss    : {:.0} m\n", recession_m));
    let land_lost_ha = recession_m * 1_000.0 / 10_000.0;
    out.push_str(&format!(
        "    - Land lost (est.)  : {:.1} ha (assume 1km coastline)\n",
        land_lost_ha
    ));
    out.push_str(&format!("    - Risk class        : {}\n", risk_class));
    out.push_str("  Health: indirect (relocation stress, waterborne if flooding co-occurs).\n");
    let mangrove_ha = land_lost_ha.max(1.0);
    let restoration_usd = mangrove_ha * 50_000_000.0 * 1.5 / IDR_PER_USD;
    out.push_str("  Economic:\n");
    out.push_str(&format!(
        "    - Mangrove restoration ({} ha): ${}\n",
        grp(mangrove_ha),
        grp(restoration_usd)
    ));
    out.push_str(&format!(
        "    - Property loss (est.): ${}\n",
        grp(recession_m * 1_000.0 * 100.0 * sev)
    ));
    out.push_str("    - Use restoration_cost(mangrove) and coastal_erosion for detail.\n");
}

// ===================== MINING IMPACT =====================
fn mining_workflow(out: &mut String, area_ha: f64, sev: f64) {
    out.push_str("===============================================================\n");
    out.push_str("PHASE 1 - DIAGNOSIS (MINING IMPACT)\n");
    out.push_str("===============================================================\n");
    let amd_ph = (6.0 - 2.0 * sev).max(2.5);
    let deforestation_ha = area_ha * (0.3 + 0.3 * sev).min(0.95);
    let heavy_metal_risk = if sev >= 1.5 {
        "CRITICAL (Ni/Cr/Co/Hg)"
    } else if sev >= 1.0 {
        "HIGH"
    } else {
        "MODERATE"
    };
    out.push_str("Inline model: AMD pH = max(6 - 2*sev, 2.5); deforestation = area*(0.3+0.3*sev)\n");
    out.push_str(&format!(
        "  AMD pH (est.)         : {:.1} ({})\n",
        amd_ph,
        if amd_ph < 4.5 {
            "PAF - acid forming"
        } else {
            "NAF or low risk"
        }
    ));
    out.push_str(&format!(
        "  Deforestation         : {:.1} ha ({:.0}% of mine)\n",
        deforestation_ha,
        deforestation_ha / area_ha * 100.0
    ));
    out.push_str(&format!("  Heavy metal risk      : {}\n", heavy_metal_risk));
    out.push_str("  Real tools: mine_impact, acid_mine_drainage, phreeqc_leaching (Tier-3 thermodynamic), dstp_plume_dispersion,\n");
    out.push_str("              mine_reclamation, tailings_management, asgm_mercury\n\n");

    out.push_str("PHASE 2 - SOLUTION\n");
    out.push_str("  1. AMD treatment (active: limestone neutralization; passive: wetland/anoxic)\n");
    out.push_str("  2. Recontouring (Permen ESDM 26/2018 - post-mining landform)\n");
    out.push_str("  3. Topsoil replacement + revegetation (endemic species)\n");
    out.push_str("  4. Wetland passive treatment (compost/limestone bioreactor)\n");
    out.push_str("  5. Tailings management (dry stacking, NOT riverine/sea dumping)\n");
    out.push_str("  6. Heavy metal immobilization (biochar, phosphate amendment)\n");
    out.push_str("  7. Community resettlement + livelihood (Perpres 56/2022)\n\n");

    out.push_str("PHASE 3 - IMPACT\n");
    out.push_str("  Environmental:\n");
    out.push_str(&format!("    - Deforested area   : {:.1} ha\n", deforestation_ha));
    out.push_str(&format!(
        "    - AMD risk (pH)     : {:.1} {}\n",
        amd_ph,
        if amd_ph < 4.5 { "(PAF)" } else { "(NAF)" }
    ));
    out.push_str(&format!("    - Heavy metals      : {}\n", heavy_metal_risk));
    out.push_str("  Health: heavy metal exposure (Hg, Ni, As) - use heavy_metal_risk,\n");
    out.push_str("    asgm_mercury (gold), or HIA if airborne particulate (dust/PM10).\n");
    let reclamation_usd = area_ha * 300_000_000.0 * sev / IDR_PER_USD;
    out.push_str("  Economic:\n");
    out.push_str(&format!(
        "    - Reclamation (Permen ESDM 26/2018): ${}\n",
        grp(reclamation_usd)
    ));
    out.push_str(&format!(
        "    - AMD treatment (est.): ${}/yr\n",
        grp(area_ha * 5_000_000.0 * sev / IDR_PER_USD)
    ));
    out.push_str(&format!(
        "    - Carbon loss (deforest): {} tCO2 (~500 tC/ha forest)\n",
        grp(deforestation_ha * 500.0 * 3.67)
    ));
    out.push_str("    - Use mine_reclamation and restoration_cost(mine) for detail.\n");
}

// ========================= SELF-CHECK TESTS =========================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_problem_type_errors() {
        let out = orchestrate("earthquake", "Test", 0.0, 0.0, 100.0, "moderate");
        assert!(out.contains("ERROR"));
        assert!(out.contains("Unknown problem_type"));
    }

    #[test]
    fn test_negative_area_errors() {
        let out = orchestrate("flood", "Test", 0.0, 0.0, -1.0, "moderate");
        assert!(out.contains("ERROR"));
    }

    #[test]
    fn test_all_problem_types_run() {
        for pt in [
            "flood",
            "fire",
            "pollution_river",
            "pollution_air",
            "coastal_erosion",
            "mining_impact",
        ] {
            let out = orchestrate(pt, "TestLoc", -6.2, 106.8, 500.0, "high");
            assert!(!out.contains("ERROR"), "problem_type {} errored: {}", pt, out);
            assert!(out.contains("PHASE 1"));
            assert!(out.contains("PHASE 2"));
            assert!(out.contains("PHASE 3"));
            assert!(out.contains("PROBLEM DEFINITION"));
            assert!(out.contains("ORCHESTRATOR LIMITATIONS"));
        }
    }

    #[test]
    fn test_air_pollution_reports_health_impact() {
        // Spec self-check: pollution_air -> should report health impact section
        let out = orchestrate("pollution_air", "Jakarta", -6.2, 106.8, 1250.0, "moderate");
        assert!(out.contains("Health"), "missing Health section: {}", out);
        assert!(out.contains("Attributable deaths"));
        assert!(out.contains("DALYs"));
        assert!(out.contains("PM2.5"));
    }

    #[test]
    fn test_phases_present_in_order() {
        let out = orchestrate("flood", "Pekalongan", -7.0, 109.7, 1000.0, "high");
        let p1 = out.find("PHASE 1").unwrap();
        let p2 = out.find("PHASE 2").unwrap();
        let p3 = out.find("PHASE 3").unwrap();
        assert!(p1 < p2 && p2 < p3, "phases not in order");
    }

    #[test]
    fn test_solution_recommendations_present() {
        let out = orchestrate("fire", "Riau", 0.5, 101.5, 2000.0, "high");
        assert!(out.contains("AWD") || out.contains("Fire break") || out.contains("Rewetting"));
        assert!(out.contains("FIRMS"));
    }

    #[test]
    fn test_diagnosis_references_real_subtool() {
        let out = orchestrate("pollution_river", "Citarum", -6.9, 107.5, 500.0, "high");
        assert!(out.contains("river_quality") || out.contains("streeter_phelps"));
        assert!(out.contains("PP 22/2021"));
    }

    #[test]
    fn test_mining_includes_amd() {
        let out = orchestrate("mining_impact", "Sulawesi", -1.0, 121.0, 500.0, "high");
        assert!(out.contains("AMD"));
        assert!(out.contains("PAF") || out.contains("NAF"));
        assert!(out.contains("Permen ESDM 26/2018"));
    }

    #[test]
    fn test_coastal_erosion_risk_class() {
        let out = orchestrate("coastal_erosion", "Demak", -6.9, 110.6, 300.0, "high");
        assert!(out.contains("Risk class"));
        assert!(out.contains("Bruun"));
        assert!(out.contains("Mangrove") || out.contains("mangrove"));
    }

    #[test]
    fn test_hia_pm25_formula() {
        // Verify inline HIA matches health_impact_assessment CRF
        let (deaths, dalys, _) = hia_pm25(40.0, 10_000_000.0, 1.0);
        let rr = 1.0615_f64.powf((40.0 - 5.0) / 10.0);
        let af = (rr - 1.0) / rr;
        let expected_deaths = 10_000_000.0 * (753.0 / 100_000.0) * af * 1.0;
        assert!((deaths - expected_deaths).abs() < 1e-6);
        assert!((dalys - deaths * 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_severity_multiplier() {
        assert_eq!(severity_mult("low"), 0.5);
        assert_eq!(severity_mult("moderate"), 1.0);
        assert_eq!(severity_mult("high"), 2.0);
        assert_eq!(severity_mult("unknown"), 1.0);
    }

    #[test]
    fn test_limitations_section() {
        let out = orchestrate("flood", "Test", 0.0, 0.0, 100.0, "moderate");
        assert!(out.contains("SYNTHESIS"));
        assert!(out.contains("Not all relevant sub-tools"));
    }

    #[test]
    fn test_economic_impact_section() {
        let out = orchestrate("fire", "Test", 0.0, 0.0, 1000.0, "high");
        assert!(out.contains("Economic"));
        assert!(out.contains("Restoration") || out.contains("Carbon loss"));
    }

    #[test]
    fn test_location_and_coords_displayed() {
        let out = orchestrate("flood", "Jakarta Utara", -6.13, 106.78, 500.0, "high");
        assert!(out.contains("Jakarta Utara"));
        assert!(out.contains("-6.1300"));
        assert!(out.contains("106.7800"));
    }

    #[test]
    fn test_grp_thousands_separator() {
        assert_eq!(grp(1_234_567.0), "1,234,567");
        assert_eq!(grp(100.0), "100");
        assert_eq!(grp(0.0), "0");
        assert_eq!(grp(-1_234.0), "-1,234");
    }
}
