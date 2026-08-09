/// Screening Mass-Balance Watershed PFAS Estimate (NOT a SWAT/MODFLOW/RT3D twin)
///
/// NOTE: Despite the references describing coupled SWAT + MODFLOW + RT3D with PEST
/// calibration, THIS TOOL runs NO SWAT, NO MODFLOW, NO RT3D, and NO PEST. It is a
/// single-cell lumped mass balance: partition rainfall into runoff/infiltration,
/// apply retardation, and route a fixed fraction to the river. The "NSE > 0.6"
/// figure is a calibration target from the literature, NOT a result computed here.
///
/// Literature Reference (NOT this tool's performance):
///   Zhang et al. 2025 (ASCE); SWAT-MODFLOW-RT3D coupled model; HYDRUS-PFAS.
///   Those models couple surface-subsurface flow and transport with calibration;
///   this tool is a screening-grade mass balance only.
pub fn assess(watershed_area_km2: f64, pfas_source_kg_yr: f64, rainfall_mm_yr: f64, soil_kd_l_kg: f64, foc_pct: f64, river_flow_m3_s: f64, n_subbasins: u32) -> String {
    let mut out = String::from("=== Screening Mass-Balance Watershed PFAS Estimate ===\n");
    out.push_str("NOTE: No SWAT/MODFLOW/RT3D/PEST runs here — this is a lumped mass balance.\n");
    out.push_str("Literature Reference (NOT this tool's performance):\n");
    out.push_str("  Zhang 2025 (ASCE); SWAT-MODFLOW-RT3D coupled; HYDRUS-PFAS\n\n");
    let foc = foc_pct / 100.0;
    let kd = foc * soil_kd_l_kg;
    let retardation = 1.0 + 2650.0 * kd / 0.3;
    let runoff_mm = rainfall_mm_yr * 0.3;
    let infiltration_mm = rainfall_mm_yr * 0.5;
    let pfas_leached_kg = pfas_source_kg_yr * (infiltration_mm / rainfall_mm_yr) / retardation.max(1.0);
    let pfas_runoff_kg = pfas_source_kg_yr * 0.05;
    let pfas_to_river_kg = pfas_leached_kg * 0.7 + pfas_runoff_kg;
    let river_volume_m3_yr = river_flow_m3_s * 86400.0 * 365.0;
    let pfas_conc_river_ug_l = if river_volume_m3_yr > 0.0 { pfas_to_river_kg * 1e6 / river_volume_m3_yr } else { 0.0 };
    let per_subbasin = pfas_to_river_kg / n_subbasins as f64;
    out.push_str(&format!("Watershed: {:.0} km2, {} subbasins\n", watershed_area_km2, n_subbasins));
    out.push_str(&format!("PFAS source: {:.1} kg/yr, Rainfall: {:.0} mm/yr\n", pfas_source_kg_yr, rainfall_mm_yr));
    out.push_str(&format!("Soil: foc={:.1}%, Koc={:.0} → Kd={:.4}, R={:.1}\n\n", foc_pct, soil_kd_l_kg, kd, retardation));
    out.push_str("-- Lumped Mass Balance (NOT coupled SWAT/MODFLOW/RT3D) --\n\n");
    out.push_str(&format!("  Runoff: {:.0} mm/yr → PFAS runoff: {:.2} kg/yr\n", runoff_mm, pfas_runoff_kg));
    out.push_str(&format!("  Infiltration: {:.0} mm/yr → PFAS leached: {:.2} kg/yr\n", infiltration_mm, pfas_leached_kg));
    out.push_str(&format!("  >> PFAS to river: {:.2} kg/yr\n", pfas_to_river_kg));
    out.push_str(&format!("  >> Per subbasin: {:.2} kg/yr\n\n", per_subbasin));
    out.push_str("-- River Concentration --\n\n");
    out.push_str(&format!("  River flow: {:.1} m3/s\n", river_flow_m3_s));
    out.push_str(&format!("  >> PFAS conc: {:.4} µg/L ({:.1} ng/L)\n\n", pfas_conc_river_ug_l, pfas_conc_river_ug_l * 1000.0));
    out.push_str("-- STATUS KEPATUHAN --\n");
    out.push_str(&format!("  EPA MCL: 4 ng/L → {}\n", if pfas_conc_river_ug_l * 1000.0 <= 4.0 {"✅"} else {"❌"}));
    out.push_str("  Indonesia: belum ada baku mutu PFAS\n\n");
    out.push_str("-- PEMANTAUAN --\n");
    out.push_str("  This tool: lumped screening mass balance (no calibration run)\n");
    out.push_str("  Literature Reference (NOT this tool's performance):\n");
    out.push_str("    Full model: SWAT + MODFLOW + RT3D coupled (Zhang 2025)\n");
    out.push_str("    Calibration target NSE > 0.6 via PEST — NOT computed by this tool\n");
    out.push_str("    Ref: Zhang 2025; SWAT-MODFLOW; HYDRUS-PFAS\n");
    out
}
