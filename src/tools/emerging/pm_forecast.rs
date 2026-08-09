/// Weighted-Lag Heuristic PM10/PM2.5 Forecasting
///
/// NOTE: Despite the reference paper using LightGBM + ResNet, THIS TOOL runs NO
/// LightGBM, NO ResNet, and NO trained model of any kind. It is a fixed-weight
/// lagged-feature heuristic (lag1/lag2/lag24 + rolling mean + meteorological
/// correction). The "Stage 2 residual correction" is a closed-form damping, not
/// a learned residual network.
///
/// Literature Reference (NOT this tool's performance):
///   Inam et al. 2026 (Discover Atmosphere) — hierarchical LightGBM + ResNet
///   ensemble for multi-country PM10 forecasting. The accuracy of that paper
///   is NOT achieved by this tool; this tool provides a fast screening forecast.
pub fn assess(pm10_history_json: &str, temp_c: f64, humidity_pct: f64, wind_speed_ms: f64, forecast_horizon_hr: u32) -> String {
    let mut out = String::from("=== Weighted-Lag Heuristic PM Forecasting ===\n");
    out.push_str("NOTE: No LightGBM/ResNet runs here — this is a fixed-weight lag heuristic.\n");
    out.push_str("Literature Reference (NOT this tool's performance):\n");
    out.push_str("  Inam et al. 2026 (Discover Atmosphere) — LightGBM + ResNet ensemble\n\n");
    let history: Vec<f64> = match serde_json::from_str(pm10_history_json) {
        Ok(v) => v,
        Err(_) => return "ERROR: pm10_history_json must be [c1,c2,...]".into(),
    };
    if history.len() < 24 { return "ERROR: Need ≥24 hours of history".into(); }
    let lag1 = history[history.len()-1];
    let lag2 = history[history.len()-2];
    let lag24 = history[history.len()-24];
    let rolling_mean: f64 = history.iter().rev().take(24).sum::<f64>() / 24.0;
    let base_pred = 0.5 * lag1 + 0.2 * lag2 + 0.15 * lag24 + 0.15 * rolling_mean;
    let met_factor = (temp_c - 25.0).abs() * 0.5 + (humidity_pct - 50.0).abs() * 0.3;
    let wind_correction = -wind_speed_ms * 2.0;
    let stage1 = base_pred + wind_correction + met_factor * 0.1;
    let residual = stage1 - lag1;
    let stage2_correction = residual * 0.3;
    let final_forecast = (stage1 + stage2_correction).max(0.0);
    let uncertainty = (history.iter().map(|h| (h - rolling_mean).powi(2)).sum::<f64>() / history.len() as f64).sqrt();
    let ci_lo = (final_forecast - 1.96 * uncertainty).max(0.0);
    let ci_hi = final_forecast + 1.96 * uncertainty;
    out.push_str(&format!("History: {} hours, Forecast horizon: {}h\n", history.len(), forecast_horizon_hr));
    out.push_str(&format!("Current: {:.1} µg/m3, Temp: {:.0}°C, Hum: {:.0}%\n\n", lag1, temp_c, humidity_pct));
    out.push_str("-- Stage 1: Weighted-Lag Features (NOT LightGBM) --\n\n");
    out.push_str(&format!("  lag1: {:.1}, lag2: {:.1}, lag24: {:.1}\n", lag1, lag2, lag24));
    out.push_str(&format!("  Rolling 24h mean: {:.1}\n", rolling_mean));
    out.push_str(&format!("  Wind correction: {:.1}\n", wind_correction));
    out.push_str(&format!("  Stage 1 prediction: {:.1} µg/m3\n\n", stage1));
    out.push_str("-- Stage 2: Residual Correction (closed-form damping, NOT ResNet) --\n\n");
    out.push_str(&format!("  Residual: {:.1}\n", residual));
    out.push_str(&format!("  Correction: {:.1}\n", stage2_correction));
    out.push_str(&format!("  >> Final forecast: {:.1} µg/m3\n\n", final_forecast));
    out.push_str("-- Uncertainty --\n\n");
    out.push_str(&format!("  Std dev: {:.1}\n", uncertainty));
    out.push_str(&format!("  95% CI: [{:.1}, {:.1}] µg/m3\n\n", ci_lo, ci_hi));
    out.push_str("-- STATUS KEPATUHAN --\n");
    out.push_str(&format!("  PM10 24jam: ≤75 µg/m3 → {}\n", if final_forecast <= 75.0 {"✅"} else {"❌"}));
    out.push_str(&format!("  PM2.5 24jam: ≤55 µg/m3 → {}\n\n", if final_forecast * 0.6 <= 55.0 {"✅"} else {"❌"}));
    out.push_str("  Literature Reference (NOT this tool's performance):\n");
    out.push_str("    Inam et al. 2026 (Discover Atmosphere) — LightGBM + ResNet\n");
    out.push_str("    This tool = weighted-lag heuristic; paper accuracy NOT reproduced\n");
    out
}
