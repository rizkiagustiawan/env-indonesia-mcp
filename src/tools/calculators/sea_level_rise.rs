/// Sea Level Rise Inundation Model
/// Ref: Bathtub model from DEM + IPCC AR6 projections

pub fn calculate(current_elevation_m: f64, slr_scenario_m: f64, storm_surge_m: f64) -> String {
    let total_water = slr_scenario_m + storm_surge_m;
    let freeboard = current_elevation_m - total_water;
    let inundated = freeboard < 0.0;

    let mut out = format!("=== Sea Level Rise Inundation ===\nRef: IPCC AR6 SLR scenarios\n\nElevasi lahan: {:.2} m (dari MSL)\nSLR skenario: +{:.2} m\nStorm surge: +{:.2} m\nTotal kenaikan air: {:.2} m\n\n", current_elevation_m, slr_scenario_m, storm_surge_m, total_water);
    out.push_str(&format!("Freeboard: {:.2} m\nStatus: {}\n\n", freeboard, if inundated { "❌ TERENDAM" } else { "✅ Aman" }));

    out.push_str("Skenario IPCC AR6 (2100):\n");
    out.push_str("  SSP1-2.6 (optimis): +0.28–0.55 m\n");
    out.push_str("  SSP2-4.5 (sedang): +0.44–0.76 m\n");
    out.push_str("  SSP5-8.5 (pesimis): +0.63–1.01 m\n");
    out.push_str("  SSP5-8.5 + ice sheet collapse: +1.0–2.0 m\n");
    if inundated { out.push_str("\n⚠️ Area ini akan tergenang. Pertimbangkan relokasi atau tanggul.\n"); }
    out
}
