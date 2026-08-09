/// Chlorophyll-a Prediction — Vollenweider/OECD Model
/// Ref: Vollenweider 1968; Chapra & Tarapchak 1976; OECD 1982
pub fn assess(
    phosphorus_load_kg_yr: f64,
    lake_area_km2: f64,
    lake_volume_m3: f64,
    outflow_m3_s: f64,
    lake_type: &str, // "deep" or "shallow"
) -> String {
    let mut out = String::from("=== Chlorophyll-a Prediction (Vollenweider/OECD) ===\n");
    out.push_str("Ref: Vollenweider 1968; OECD 1982; Chapra & Tarapchak 1976\n\n");

    let A = lake_area_km2 * 1e6; // m2
    let V = lake_volume_m3;
    let Q = outflow_m3_s * 86400.0 * 365.0; // m3/year

    // Phosphorus loading (g P/m2/year)
    let L = phosphorus_load_kg_yr * 1000.0 / A; // g/m2/year

    // Hydraulic residence time (years)
    let tau = V / Q.max(1.0);

    // Hydraulic load (m/year)
    let qs = Q / A;

    // Vollenweider P concentration: P = L / (qs * (1 + sqrt(tau)))
    let p_conc_ug_l = L * 1000.0 / (qs * (1.0 + tau.sqrt()).max(1e-6)); // g/m2/year -> ug/L

    // Chl-a prediction: log(Chl-a) = a + b * log(P)
    let (a, b, _label) = match lake_type.to_lowercase().as_str() {
        "deep" => (-0.438, 1.058, "deep lakes (OECD)"),
        "shallow" => (-0.345, 0.911, "shallow lakes (OECD)"),
        _ => (-0.432, 0.794, "all lakes (OECD combined)"),
    };

    let log_p = (p_conc_ug_l.max(0.1)).log10();
    let log_chla = a + b * log_p;
    let chla_ug_l = 10.0_f64.powf(log_chla);

    out.push_str(&format!("Lake: area={:.1} km2, volume={:.0e} m3\n", lake_area_km2, V as f64));
    out.push_str(&format!("Outflow: {:.1} m3/s ({:.0e} m3/year)\n", outflow_m3_s, Q));
    out.push_str(&format!("P loading: {:.0} kg/year ({:.2} g/m2/year)\n\n", phosphorus_load_kg_yr, L));

    out.push_str("-- Vollenweider P Model --\n\n");
    out.push_str(&format!("  Hydraulic residence time: {:.2} years\n", tau));
    out.push_str(&format!("  Hydraulic load: {:.1} m/year\n", qs));
    out.push_str(&format!("  >> P concentration: {:.1} ug/L\n\n", p_conc_ug_l));

    out.push_str("-- Chlorophyll-a Prediction --\n\n");
    out.push_str(&format!("  OECD model ({}): log(Chl) = {:.3} + {:.3}*log(P)\n", lake_type, a, b));
    out.push_str(&format!("  >> Chl-a: {:.1} ug/L\n\n", chla_ug_l));

    // Trophic state classification
    out.push_str("-- Trophic State --\n");
    let (state, p_status, chl_status) = if p_conc_ug_l < 10.0 {
        ("Oligotrophic", "P < 10 ug/L (low nutrient)", "Chl-a < 3.5 ug/L (clear water)")
    } else if p_conc_ug_l < 35.0 {
        ("Mesotrophic", "P 10-35 ug/L (moderate)", "Chl-a 3.5-9 ug/L")
    } else if p_conc_ug_l < 100.0 {
        ("Eutrophic", "P 35-100 ug/L (high)", "Chl-a 9-25 ug/L (algae blooms)")
    } else {
        ("Hypertrophic", "P > 100 ug/L (excessive)", "Chl-a > 25 ug/L (severe)")
    };
    out.push_str(&format!("  >> {} (P={:.1} ug/L, Chl-a={:.1} ug/L)\n", state, p_conc_ug_l, chla_ug_l));
    out.push_str(&format!("  {}\n", p_status));
    out.push_str(&format!("  {}\n", chl_status));

    // Vollenweider permissible loading
    let _l_permissible = 2.7 * qs * (1.0 + tau.sqrt()) / (1.0 + tau.sqrt()).max(1e-6);

    out.push_str("\n  Ref: Vollenweider 1968; OECD 1982; Chapra & Tarapchak 1976\n");
    out
}
