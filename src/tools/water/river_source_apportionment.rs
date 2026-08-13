/// River Source Apportionment — Multi-source BOD Mass Balance
/// Problem: river_quality.rs handles single BOD load decay, not multi-source attribution.
/// Method: 1D steady-state first-order decay per source; sum at river mouth; attribution %.
/// Ref: Chapra 2008 "Surface Water Quality Modeling"; QUAL2K (Chapra & Pelletier 2003);
///      Thomann & Mueller 1987; PP 22/2021 Lampiran VI (BOD Kelas II <= 3 mg/L).
/// Formula:
///   For source i at distance x_i (km from upstream), load L_i (kg/day):
///   BOD_i at river mouth = L_i * exp(-k * (L_river - x_i) / u)
///   where u = velocity (km/day), k = BOD deoxygenation rate (1/day, tropical 0.1-0.5)
///   Velocity estimate: u = Q / (W * D); W from Q (Leopold scaling), D ~ 1-3 m tropical river
///   Total at mouth = sum of all BOD_i; Concentration = Total / Q

pub fn apportion(
    river_length_km: f64,
    flow_m3_s: f64,
    sources_json: &str,
) -> String {
    if river_length_km <= 0.0 || river_length_km > 1000.0 {
        return "ERROR [E102]: river_length_km must be 0-1000.".into();
    }
    if flow_m3_s <= 0.0 {
        return "ERROR [E102]: flow_m3_s must be > 0.".into();
    }

    #[derive(serde::Deserialize)]
    struct Source {
        name: String,
        bod_kg_day: f64,
        distance_km: f64,
        #[serde(rename = "type")]
        source_type: String,
    }

    let sources: Vec<Source> = match serde_json::from_str(sources_json) {
        Ok(v) => v,
        Err(e) => return format!(
            "ERROR: Gagal parse sources_json: {}.\nFormat: [{{\"name\":\"IPAL-X\",\"bod_kg_day\":100,\"distance_km\":10,\"type\":\"point\"}}]",
            e
        ),
    };

    if sources.is_empty() {
        return "ERROR: Minimal 1 source.".into();
    }

    // Validate sources
    for s in &sources {
        if s.bod_kg_day < 0.0 {
            return format!("ERROR: bod_kg_day for '{}' must be >= 0.", s.name);
        }
        if s.distance_km < 0.0 || s.distance_km > river_length_km {
            return format!("ERROR: distance_km for '{}' must be 0-{}", s.name, river_length_km);
        }
        if s.source_type != "point" && s.source_type != "diffuse" {
            return format!("ERROR: type for '{}' must be 'point' or 'diffuse'.", s.name);
        }
    }

    let mut out = String::from("════════════════════════════════════════════════════════\n");
    out.push_str("RIVER SOURCE APPORTIONMENT — MULTI-SOURCE BOD DECAY\n");
    out.push_str("Ref: Chapra 2008; QUAL2K; PP 22/2021 Lampiran VI\n");
    out.push_str("════════════════════════════════════════════════════════\n\n");

    // ─── HYDRAULICS ───
    // Estimate river width from flow (Leopold-Maddock empirical): W = a * Q^b
    // For tropical rivers: a ~ 8.0, b ~ 0.5 (width in m, Q in m3/s)
    let width_m = 8.0 * flow_m3_s.powf(0.5);
    // Depth: assume 1.5m average (tropical lowland river) — user can refine
    let depth_m = 1.5;
    let velocity_m_s = flow_m3_s / (width_m * depth_m);
    let velocity_km_day = velocity_m_s * 86.4; // m/s * 86400 / 1000

    // BOD deoxygenation rate (tropical, 25-30C): k20 = 0.1-0.5 /day
    // Default 0.3 (mid-tropical); temperature correction theta=1.047
    let k_day = 0.3;

    out.push_str("RIVER HYDRAULICS (estimated):\n");
    out.push_str(&format!("   Length        : {:.1} km\n", river_length_km));
    out.push_str(&format!("   Flow Q        : {:.1} m3/s\n", flow_m3_s));
    out.push_str(&format!("   Width (est)   : {:.1} m (Leopold Q^0.5)\n", width_m));
    out.push_str(&format!("   Depth (assumed): {:.1} m (tropical lowland)\n", depth_m));
    out.push_str(&format!("   Velocity u    : {:.3} m/s = {:.1} km/day\n", velocity_m_s, velocity_km_day));
    out.push_str(&format!("   BOD k (25C)   : {:.2} /day (tropical range 0.1-0.5)\n\n", k_day));

    // ─── PER-SOURCE DECAY ───
    out.push_str("PER-SOURCE BOD AT RIVER MOUTH:\n");
    out.push_str(&format!("  {:<20} {:>10} {:>10} {:>10} {:>10} {:>12}\n",
        "Source", "Load(kg/d)", "x(km)", "type", "BOD_mouth", "Attribution"));
    out.push_str(&format!("  {}\n", "─".repeat(76)));

    let mut total_bod_mouth: f64 = 0.0;
    let mut results: Vec<(String, f64, f64, String, f64)> = Vec::new(); // (name, load, x, type, bod_mouth)

    for s in &sources {
        let travel_km = river_length_km - s.distance_km;
        let travel_days = travel_km / velocity_km_day.max(0.01);
        // For diffuse sources, BOD is distributed; approximate as point at centroid
        // (a true diffuse integration would split into N sub-loads)
        let decay_factor = (-k_day * travel_days).exp();
        let bod_mouth = s.bod_kg_day * decay_factor;
        total_bod_mouth += bod_mouth;
        results.push((s.name.clone(), s.bod_kg_day, s.distance_km, s.source_type.clone(), bod_mouth));
    }

    for (name, load, x, stype, bod_mouth) in &results {
        let pct = if total_bod_mouth > 0.0 { bod_mouth / total_bod_mouth * 100.0 } else { 0.0 };
        out.push_str(&format!("  {:<20} {:>10.1} {:>10.1} {:>10} {:>10.2} {:>11.1}%\n",
            name, load, x, stype, bod_mouth, pct));
    }
    out.push_str(&format!("  {}\n", "─".repeat(76)));
    out.push_str(&format!("  {:<20} {:>10.1} {:>10} {:>10} {:>10.2} {:>11.1}%\n\n",
        "TOTAL", sources.iter().map(|s| s.bod_kg_day).sum::<f64>(),
        "", "", total_bod_mouth, 100.0));

    // ─── RIVER MOUTH CONCENTRATION ───
    // C = total_load_kg_day / (Q * 86400) converted to mg/L
    // 1 kg/day = 1e6 mg/day; Q*86400 = m3/day; 1 m3 = 1000 L
    // C (mg/L) = (load_kg_day * 1e6) / (Q_m3_s * 86400 * 1000) = load_kg_day / (Q * 86.4)
    let conc_mg_l = total_bod_mouth / (flow_m3_s * 86.4);
    out.push_str(&format!("RIVER MOUTH BOD CONCENTRATION: {:.2} mg/L\n", conc_mg_l));
    out.push_str(&format!("  (C = total_load_kg_d / (Q * 86.4); 1 kg/d / m3/d conversion)\n\n"));

    // ─── PP 22/2021 COMPLIANCE ───
    out.push_str("─── PP 22/2021 Lampiran VI COMPLIANCE (BOD) ───\n");
    let class1 = 2.0; // mg/L
    let class2 = 3.0;
    let class3 = 6.0;
    out.push_str(&format!("  Kelas I  (<= 2 mg/L, raw drinking water): {}\n",
        if conc_mg_l <= class1 { "MEMENUHI" } else { "TIDAK MEMENUHI" }));
    out.push_str(&format!("  Kelas II (<= 3 mg/L, recreation):        {}\n",
        if conc_mg_l <= class2 { "MEMENUHI" } else { "TIDAK MEMENUHI" }));
    out.push_str(&format!("  Kelas III(<= 6 mg/L, livestock):         {}\n\n",
        if conc_mg_l <= class3 { "MEMENUHI" } else { "TIDAK MEMENUHI" }));

    // ─── DOMINANT SOURCE IDENTIFICATION ───
    if !results.is_empty() {
        let mut sorted = results.clone();
        sorted.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap_or(std::cmp::Ordering::Equal));
        let top = &sorted[0];
        let top_pct = if total_bod_mouth > 0.0 { top.4 / total_bod_mouth * 100.0 } else { 0.0 };
        out.push_str(&format!("DOMINANT SOURCE: {} ({:.1}% of mouth load)\n\n", top.0, top_pct));
    }

    // ─── INDONESIA CONTEXT ───
    out.push_str("─── INDONESIA RIVER CONTEXT ───\n");
    out.push_str("  Citarum (West Java): most polluted river in Indonesia.\n");
    out.push_str("    Sources: 2,000+ textile factories (point), domestic Bandung\n");
    out.push_str("    (point/diffuse), agriculture (diffuse). BOD upstream 20-50 mg/L.\n");
    out.push_str("    Citarum Harum program (2018-2025) target BOD < 3 mg/L.\n\n");
    out.push_str("  Brantas (East Java): industrial + domestic Surabaya + Kediri.\n");
    out.push_str("  Solo (Central/East Java): agriculture + domestic Solo City.\n");
    out.push_str("  Ciliwung (Jakarta): domestic (70%+), squatter settlements.\n\n");

    // ─── MITIGATION ───
    out.push_str("─── MITIGATION RECOMMENDATIONS ───\n");
    if conc_mg_l > class2 {
        out.push_str(&format!("  BOD {:.2} mg/L exceeds PP 22/2021 Kelas II (3 mg/L):\n", conc_mg_l));
        out.push_str("  1. Prioritize dominant source reduction (see above).\n");
        out.push_str("  2. For point sources: enforce IPAL (PermenLHK 11/2025 for domestik,\n");
        out.push_str("     PP 22/2021 for industri). Effluent BOD <= 30 mg/L (standar).\n");
        out.push_str("  3. For diffuse: riparian buffer, constructed wetland, agricultural BMP.\n");
        out.push_str("  4. Continuous water quality monitoring (SPKLH / SNI 6989 series).\n");
        out.push_str("  5. Pollution Load Capacity (Daya Tampung Beban Pencemaran) per PP 22/2021.\n");
    } else {
        out.push_str("  BOD within Kelas II — maintain monitoring.\n");
    }

    // ─── LIMITATIONS ───
    out.push_str("\n─── HONEST LIMITATIONS ───\n");
    out.push_str("  1. 1D ONLY: no lateral/vertical variation; no meandering effect.\n");
    out.push_str("  2. NO TRIBUTARY CONFLUENCE: assumes single mainstem. Real rivers\n");
    out.push_str("     have multiple branches (Citarum has 9 major tributaries).\n");
    out.push_str("  3. SINGLE PARAMETER (BOD): no DO coupling, no N/P, no coliform.\n");
    out.push_str("     Use tools::calculators::river_quality for full BOD-DO sag curve.\n");
    out.push_str("  4. DIFFUSE = POINT APPROX: true diffuse source requires integration\n");
    out.push_str("     over river segment (QUAL2K diffuse source rate per km).\n");
    out.push_str("  5. CONSTANT k: BOD deoxygenation rate varies with temperature,\n");
    out.push_str("     nutrient availability, microbial community. theta=1.047 for T-corr.\n");
    out.push_str("  6. VELOCITY ESTIMATE: width/depth from Q is rough. For policy,\n");
    out.push_str("     use measured hydraulic geometry (gauge data, BIG/PUSAIR).\n");
    out.push_str("  7. STEADY STATE: no transient events (storm flush, dam release).\n");
    out.push_str("  8. NO SEDIMENT OXYGEN DEMAND: SOD can dominate in muddy rivers.\n");
    out.push_str("════════════════════════════════════════════════════════\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Self-check from spec:
    // 2 sources: at 0km (100 kg/d), at 50km (100 kg/d), L=100km, k=0.3/day, u=0.5 m/s=43.2km/day
    // source1: exp(-0.3 * 100/43.2) = exp(-0.694) = 0.499 -> 49.9 kg/d (spec says ~50)
    // source2: exp(-0.3 * 50/43.2) = exp(-0.347) = 0.707 -> 70.7 kg/d (spec says ~70)
    // Total = 120.6 kg/d; attribution 41.4% / 58.6%
    // NOTE: our tool estimates u from Q. To match spec u=0.5 m/s with depth=1.5: Q = u*W*D = 0.5 * 8*sqrt(Q) * 1.5
    // -> Q = 6 * sqrt(Q) -> sqrt(Q)=6 -> Q=36 m3/s. So flow=36 gives u~0.5 m/s.
    #[test]
    fn two_source_apportionment_matches_spec() {
        let sources = r#"[
            {"name":"src_upstream","bod_kg_day":100,"distance_km":0,"type":"point"},
            {"name":"src_midstream","bod_kg_day":100,"distance_km":50,"type":"point"}
        ]"#;
        let res = apportion(100.0, 36.0, sources);
        assert!(!res.contains("ERROR"), "unexpected error:\n{}", res);
        // With Q=36, W=8*6=48, D=1.5, u = 36/(48*1.5) = 0.5 m/s = 43.2 km/day ✓
        // Source1: exp(-0.3*100/43.2) = 0.499 -> 49.9
        // Source2: exp(-0.3*50/43.2) = 0.707 -> 70.7
        assert!(res.contains("49.9") || res.contains("50.0"), "expected ~50 kg/d for source1, got:\n{}", res);
        assert!(res.contains("70.7") || res.contains("70.6"), "expected ~70.7 kg/d for source2, got:\n{}", res);
    }

    #[test]
    fn rejects_invalid_json() {
        let res = apportion(100.0, 10.0, "not valid json");
        assert!(res.contains("ERROR"));
    }

    #[test]
    fn rejects_negative_load() {
        let sources = r#"[{"name":"x","bod_kg_day":-5,"distance_km":0,"type":"point"}]"#;
        let res = apportion(100.0, 10.0, sources);
        assert!(res.contains("ERROR"));
    }

    // Self-check: single source at mouth (distance = river_length) has decay factor 1.0
    #[test]
    fn source_at_mouth_no_decay() {
        let sources = r#"[{"name":"mouth_src","bod_kg_day":50,"distance_km":100,"type":"point"}]"#;
        let res = apportion(100.0, 36.0, sources);
        assert!(res.contains("50.00"), "source at mouth should not decay, got:\n{}", res);
    }
}
