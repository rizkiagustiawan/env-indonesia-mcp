/// MBR (Membrane Bioreactor) Design
/// Ref: Judd & Judd 2011 "Membrane Bioreactors"; Lee 2002; Yoon 2004
pub fn assess(
    flow_m3_day: f64,
    influent_bod_mg_l: f64,
    target_effluent_bod_mg_l: f64,
    hrt_hours: f64,
    srt_days: f64,
    mlss_mg_l: f64,
    membrane_flux_lmh: f64,
    temp_c: f64,
) -> String {
    let mut out = String::from("=== MBR (Membrane Bioreactor) Design ===\n");
    out.push_str("Ref: Judd & Judd 2011; Lee 2002; Yoon 2004\n\n");

    let Q = flow_m3_day;
    let V = Q * hrt_hours / 24.0; // reactor volume m3
    let F_M = Q * influent_bod_mg_l / (V * mlss_mg_l); // F/M ratio
    let srt = srt_days;
    let waste_sludge = V * mlss_mg_l / (srt * 1000.0); // kg/day wasted

    // Membrane area
    let perm_flow = Q * 1000.0 / 24.0; // L/hr
    let membrane_area = perm_flow / membrane_flux_lmh.max(1e-6); // m2
    let n_modules = (membrane_area / 25.0).ceil() as u32; // 25 m2 per module

    // O2 demand (carbonaceous + endogenous)
    let bod_removed = (influent_bod_mg_l - target_effluent_bod_mg_l) * Q / 1000.0; // kg/day
    let o2_demand = bod_removed * 1.1 + waste_sludge * 1.42 * 0.7; // ~1.1 kg O2/kg BOD + endogenous

    // Temperature correction for nitrification
    let k_nitrification = 0.5 * 1.034_f64.powf(temp_c - 20.0); // day-1
    let srt_min_nitr = 1.0 / k_nitrification.max(1e-6);

    out.push_str(&format!("Flow: {:.1} m3/day\n", Q));
    out.push_str(&format!("BOD: {:.0} -> {:.0} mg/L\n", influent_bod_mg_l, target_effluent_bod_mg_l));
    out.push_str(&format!("HRT: {:.1} hr, SRT: {:.0} days\n", hrt_hours, srt));
    out.push_str(&format!("MLSS: {} mg/L, Flux: {:.1} LMH\n\n", mlss_mg_l, membrane_flux_lmh));

    out.push_str("-- Design Parameters --\n\n");
    out.push_str(&format!("  Reactor volume: {:.1} m3\n", V));
    out.push_str(&format!("  F/M ratio: {:.3} kg BOD/kg MLSS/day\n", F_M));
    out.push_str(&format!("  Waste sludge: {:.1} kg/day\n", waste_sludge));
    out.push_str(&format!("  O2 demand: {:.1} kg/day\n", o2_demand));
    out.push_str(&format!("  Membrane area: {:.1} m2 ({} modules)\n", membrane_area, n_modules));
    out.push_str(&format!("  Nitrification k({:.0}C): {:.3}/day, SRT_min: {:.1}d\n\n", temp_c, k_nitrification, srt_min_nitr));

    if F_M > 0.2 { out.push_str("  [WARN] F/M > 0.2 — high loading, fouling risk\n"); }
    else if F_M < 0.05 { out.push_str("  [WARN] F/M < 0.05 — low loading, EPS increase\n"); }
    else { out.push_str("  [OK] F/M in range 0.05-0.2\n"); }

    if srt < srt_min_nitr { out.push_str(&format!("  [WARN] SRT < SRT_min for nitrification ({:.1}d). No N removal.\n", srt_min_nitr)); }
    else { out.push_str("  [OK] SRT sufficient for nitrification\n"); }

    out.push_str("\n  Ref: Judd & Judd 2011; Metcalf & Eddy 2004\n");
    out
}
