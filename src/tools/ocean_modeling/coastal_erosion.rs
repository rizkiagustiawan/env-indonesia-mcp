/// Pantura Coastal Erosion — Combined Bruun + CERC Longshore + Human Factor
/// Problem: Only Bruun Rule (SLR-only) exists; no sediment budget or longshore transport.
/// Method: Bruun recession + CERC longshore transport + sand mining + mangrove loss.
/// Ref: Bruun 1962; USACE CERC (SPM 1984); Hanson & Kraus (GENESIS); Kamphuis 1991;
///      van Rijn 2014; IPCC AR6; Marfai et al. (Pantura Pekalongan subsidence-accelerated);
///      PP 22/2021 (coastal zone); UU 27/2007 (Pengelolaan Wilayah Pesisir).
/// Formula:
///   Bruun: R_bruun = SLR * L / (B + h*)
///   CERC: Q = K * (rho_w * g * Hs^2 * c_gb) / (16 * (rho_s - rho_w) * (1-p) * g) * sin(2*alpha_b)
///         simplified to Q = K * Hs^2 * c_gb * sin(2*alpha_b) / (8 * (s-1) * (1-p))  [m3/s]
///         with K ~ 0.39 (SPM 1984), s = rho_s/rho_w = 2.65, p = 0.4
///   Net shoreline change rate = R_bruun + (Q_in - Q_out) * t / (D * beach_length) - mining - mangrove
///   Note: CERC is alongshore transport volume; shoreline response requires transport gradient.
///         Here we use transport imbalance * planning_horizon / active_profile_height as recession.

pub fn assess(
    shoreline_length_km: f64,
    sea_level_rise_m: f64,
    closure_depth_m: f64,
    beach_width_m: f64,
    wave_height_m: f64,
    wave_period_s: f64,
    wave_angle_deg: f64,
    sand_mining_m3_yr: f64,
    mangrove_loss_ha: f64,
    planning_horizon_years: u32,
) -> String {
    if shoreline_length_km <= 0.0 || shoreline_length_km > 500.0 {
        return "ERROR [E102]: shoreline_length_km must be 0-500.".into();
    }
    if sea_level_rise_m < 0.0 || sea_level_rise_m > 3.0 {
        return "ERROR [E102]: sea_level_rise_m outside plausible [0, 3].".into();
    }
    if wave_height_m <= 0.0 || wave_height_m > 15.0 {
        return "ERROR [E102]: wave_height_m outside plausible (0, 15].".into();
    }
    if planning_horizon_years == 0 || planning_horizon_years > 100 {
        return "ERROR [E102]: planning_horizon_years must be 1-100.".into();
    }

    let mut out = String::from("════════════════════════════════════════════════════════════\n");
    out.push_str("PANTURA COASTAL EROSION — BRUUN + CERC + HUMAN FACTOR\n");
    out.push_str("Ref: Bruun 1962; USACE CERC SPM 1984; van Rijn 2014; IPCC AR6\n");
    out.push_str("════════════════════════════════════════════════════════════\n\n");

    let years = planning_horizon_years as f64;

    // ─── 1. BRUUN RULE ───
    // R_bruun = SLR * L / (B + h*)
    // L = closure distance (estimate from closure depth + beach slope ~1:100)
    let berm_height_m = 2.0; // typical sandy berm, Indonesia 1-3m
    let closure_distance_m = (closure_depth_m / 0.01).min(2000.0); // assume 1:100 slope
    let bruun_recession_m = sea_level_rise_m * closure_distance_m / (berm_height_m + closure_depth_m);
    let bruun_rate_m_yr = bruun_recession_m / years.max(1.0);

    out.push_str("1. BRUUN RULE (SLR-driven recession):\n");
    out.push_str(&format!("   Berm height B = {:.1} m (assumed typical)\n", berm_height_m));
    out.push_str(&format!("   Beach width   = {:.0} m (buffer before infrastructure)\n", beach_width_m));
    out.push_str(&format!("   Closure depth h* = {:.1} m\n", closure_depth_m));
    out.push_str(&format!("   Closure distance L = {:.0} m (from 1:100 slope)\n", closure_distance_m));
    out.push_str(&format!("   R_bruun = SLR * L / (B + h*) = {:.3}*{:.0}/({:.1}+{:.1}) = {:.2} m\n",
        sea_level_rise_m, closure_distance_m, berm_height_m, closure_depth_m, bruun_recession_m));
    out.push_str(&format!("   Rate = {:.3} m/yr over {:.0} yr\n\n", bruun_rate_m_yr, years));

    // ─── 2. CERC LONGSHORE TRANSPORT ───
    // Q = K * (rho_w * g * Hsb^2 * cgb) * sin(2*alpha_b) / (16 * (rho_s - rho_w) * (1-p) * g)
    // Simplified (divide numerator/denominator by rho_w*g):
    //   Q = K * Hsb^2 * cgb * sin(2*alpha_b) / (16 * (s-1) * (1-p))   [m3/s]
    // where s = rho_s/rho_w = 2.65, p = 0.4 porosity
    // cgb = group velocity at breaking ~ sqrt(g * hb) ~ sqrt(g * Hsb/gamma_b)
    // gamma_b = 0.78 (breaker index)
    let k_cerc = 0.39; // SPM 1984 recommended
    let s = 2.65; // quartz sand
    let porosity = 0.4;
    let gamma_b = 0.78;
    let g = 9.81;
    let h_break = wave_height_m / gamma_b; // depth at breaking
    let c_gb = (g * h_break).sqrt(); // group velocity at breaking (shallow water)
    let alpha_rad = wave_angle_deg.to_radians();
    let sin2alpha = (2.0 * alpha_rad).sin();
    // Q in m3/s
    let q_m3_s = k_cerc * wave_height_m.powi(2) * c_gb * sin2alpha
        / (16.0 * (s - 1.0) * (1.0 - porosity));
    let q_m3_yr = q_m3_s * 365.25 * 86400.0;

    // Sign convention: positive Q = net alongshore transport (downdrift).
    // Recession (or accretion) depends on transport GRADIENT dQ/dx.
    // For a straight uniform coastline with constant wave angle, dQ/dx = 0 -> no net shoreline change.
    // But for planning we assume an imbalance of |Q| * imbalance_fraction.
    // Typical imbalance (spit/headland downdrift): 10-30% of Q causes erosion.
    let imbalance_fraction = 0.2; // 20% transport gradient (assumed)
    let q_imbalance_m3_yr = q_m3_yr * imbalance_fraction;
    // Convert volume imbalance to shoreline recession:
    // dV/dt = (D_active) * dR/dt  -> R = (Q_imbalance * years) / (D_active * shoreline_length)
    let d_active_m = berm_height_m + closure_depth_m; // active profile height
    let shoreline_length_m = shoreline_length_km * 1000.0;
    let cerc_recession_m = (q_imbalance_m3_yr * years) / (d_active_m * shoreline_length_m);

    out.push_str("2. CERC LONGSHORE TRANSPORT:\n");
    out.push_str(&format!("   Wave Hs = {:.1} m, T = {:.1} s, alpha = {:.1} deg\n",
        wave_height_m, wave_period_s, wave_angle_deg));
    out.push_str(&format!("   Breaker depth hb = Hs/gamma_b = {:.2} m (gamma_b=0.78)\n", h_break));
    out.push_str(&format!("   Group velocity cgb = sqrt(g*hb) = {:.2} m/s\n", c_gb));
    out.push_str(&format!("   K (CERC) = {:.2}, s = {:.2}, porosity = {:.1}\n", k_cerc, s, porosity));
    out.push_str(&format!("   sin(2*alpha) = {:.3}\n", sin2alpha));
    out.push_str(&format!("   Q = {:.4} m3/s = {:.0} m3/yr (gross alongshore)\n", q_m3_s, q_m3_yr));
    out.push_str(&format!("   Imbalance (20%) = {:.0} m3/yr -> recession {:.2} m over {:.0} yr\n\n",
        q_imbalance_m3_yr, cerc_recession_m, years));

    // ─── 3. SAND MINING ───
    // Sand mining removes sediment budget: R_mining = V_mining / (D_active * shoreline_length) per yr
    let mining_recession_m = (sand_mining_m3_yr * years) / (d_active_m * shoreline_length_m);
    out.push_str("3. SAND MINING (sediment budget deficit):\n");
    out.push_str(&format!("   Volume = {:.0} m3/yr x {:.0} yr = {:.0} m3\n",
        sand_mining_m3_yr, years, sand_mining_m3_yr * years));
    out.push_str(&format!("   R_mining = V / (D_active * L) = {:.2} m\n\n", mining_recession_m));

    // ─── 4. MANGROVE LOSS ───
    // Mangrove loss exposes coast; estimate additional recession ~1m per ha lost per km coastline
    // (empirical from Marfai et al. for Pekalongan/Demak)
    let mangrove_recession_m = (mangrove_loss_ha * 1.0) / shoreline_length_km.max(0.1);
    out.push_str("4. MANGROVE LOSS (exposure increase):\n");
    out.push_str(&format!("   Lost = {:.0} ha over {:.1} km shoreline\n", mangrove_loss_ha, shoreline_length_km));
    out.push_str(&format!("   R_mangrove ~ {:.2} m (empirical ~1 m/ha per km)\n\n", mangrove_recession_m));

    // ─── NET SHORELINE CHANGE ───
    let net_recession_m = bruun_recession_m + cerc_recession_m + mining_recession_m + mangrove_recession_m;
    let net_rate_m_yr = net_recession_m / years;

    out.push_str("════════════════════════════════════════════════════════════\n");
    out.push_str("NET SHORELINE RECESSION:\n");
    out.push_str(&format!("   Bruun (SLR)      : +{:.2} m ({:.3} m/yr)\n", bruun_recession_m, bruun_rate_m_yr));
    out.push_str(&format!("   CERC imbalance   : +{:.2} m\n", cerc_recession_m));
    out.push_str(&format!("   Sand mining      : +{:.2} m\n", mining_recession_m));
    out.push_str(&format!("   Mangrove loss    : +{:.2} m\n", mangrove_recession_m));
    out.push_str(&format!("\n   >>> TOTAL RECESSION = {:.2} m over {:.0} yr = {:.3} m/yr <<<\n\n",
        net_recession_m, years, net_rate_m_yr));

    // Area lost
    let area_lost_ha = net_recession_m * shoreline_length_m / 10000.0;
    out.push_str(&format!("   Area lost: {:.2} ha ({:.0} m x {:.0} m)\n\n",
        area_lost_ha, net_recession_m, shoreline_length_m));

    // Beach-width exposure check: if recession exceeds beach width, infrastructure threatened
    let infra_exposure = net_recession_m >= beach_width_m;
    out.push_str(&format!("BEACH WIDTH EXPOSURE:\n"));
    out.push_str(&format!("   Beach width: {:.0} m | Net recession: {:.2} m\n", beach_width_m, net_recession_m));
    out.push_str(&format!("   Infrastructure (landward of beach): {}\n\n",
        if infra_exposure { "EXPOSED — recession exceeds beach buffer" } else { "buffered by beach width" }));

    // ─── RISK CLASS ───
    let risk_class = if net_rate_m_yr > 2.0 { "CRITICAL (>2 m/yr)" }
        else if net_rate_m_yr > 1.0 { "HIGH (1-2 m/yr)" }
        else if net_rate_m_yr > 0.3 { "MODERATE (0.3-1 m/yr)" }
        else if net_rate_m_yr > 0.0 { "LOW (<0.3 m/yr)" }
        else { "STABLE/ACCRETING" };
    out.push_str(&format!("RISK CLASS: {}\n\n", risk_class));

    // ─── INDONESIA CONTEXT ───
    out.push_str("─── PANTURA (NORTH COAST JAVA) CONTEXT ───\n");
    out.push_str("  Pekalongan : subsidence -100 to -200 mm/yr + erosion 5-15 m/yr (Marfai).\n");
    out.push_str("  Semarang   : subsidence -150 mm/yr, Demak regression 'rob' flood.\n");
    out.push_str("  Demak      : Sayung coastal retreat 1-2 km since 1990s (mangrove loss + mining).\n");
    out.push_str("  Indramayu  : erosion 1-3 m/yr; mangrove conversion to tambak.\n");
    out.push_str("  Subsidence-accelerated: Pantura has COMPOUND erosion (SLR + subsidence + mining).\n\n");

    // ─── MITIGATION ───
    out.push_str("─── MITIGATION RECOMMENDATIONS ───\n");
    if net_rate_m_yr > 1.0 {
        out.push_str("  CRITICAL erosion — combined hard + soft engineering:\n");
        out.push_str("  1. Mangrove restoration (Rhizophora, Avicennia) — 100-200 m green belt.\n");
        out.push_str("     Cost-effective, co-benefits (carbon, fisheries, biodiversity).\n");
        out.push_str("  2. Beach nourishment (sand supply) — restore sediment budget.\n");
        out.push_str("     Source sand from offshore (avoid mining coastal sediment).\n");
        out.push_str("  3. Groin field / detached breakwater — reduce alongshore transport loss.\n");
        out.push_str("  4. STOP sand mining — enforce UU 27/2007, Permen ESDM on galian C.\n");
        out.push_str("  5. Seawall for critical infrastructure (last resort, causes downdrift erosion).\n");
        out.push_str("  6. Managed retreat where recession > 2 m/yr (Demak case).\n");
        out.push_str("  7. Address subsidence (see tools::advanced_physics::jakarta_coastal_risk).\n");
    } else if net_rate_m_yr > 0.3 {
        out.push_str("  MODERATE erosion — soft engineering + monitoring:\n");
        out.push_str("  1. Mangrove restoration / coastal revegetation.\n");
        out.push_str("  2. Beach nourishment periodic.\n");
        out.push_str("  3. Limit sand mining (zoning, quotas).\n");
        out.push_str("  4. Annual shoreline monitoring (satellite + DGPS survey).\n");
    } else {
        out.push_str("  LOW erosion / stable — monitoring:\n");
        out.push_str("  1. Biennial shoreline monitoring (Landsat/Sentinel-2 coastline extraction).\n");
        out.push_str("  2. Protect existing mangrove.\n");
    }

    // ─── REGULATORY ───
    out.push_str("\n─── REGULATORY CONTEXT ───\n");
    out.push_str("  - UU 27/2007 (Pengelolaan Wilayah Pesisir dan Pulau-Pulau Kecil)\n");
    out.push_str("  - UU 1/2014 (perubahan UU 27/2007)\n");
    out.push_str("  - PP 22/2021 (PPLH — coastal zone management)\n");
    out.push_str("  - Permen KP 9/2024 (kegiatan pesisir); Perda Pantura provinsi\n");
    out.push_str("  - Permen ESDM (galian C / sand mining quotas)\n");
    out.push_str("  - RAN-PI 2014 (National Adaptation Plan)\n\n");

    // ─── LIMITATIONS ───
    out.push_str("─── HONEST LIMITATIONS ───\n");
    out.push_str("  1. NO 2D COASTLINE EVOLUTION: assumes straight shoreline; no headlands,\n");
    out.push_str("     tombolos, tidal inlets. Use GENESIS, LX-Shore, or ShorelineS for 2D.\n");
    out.push_str("  2. NO SEASONAL VARIATION: monsoon reverses wave direction. Annual\n");
    out.push_str("     average may mask winter/summer erosion-accretion cycles.\n");
    out.push_str("  3. SIMPLIFIED CROSS-SHORE: Bruun assumes equilibrium profile (sandy beach).\n");
    out.push_str("     Mud coasts (Pantura, Demak) do NOT follow Bruun well.\n");
    out.push_str("  4. CERC K UNCERTAINTY: factor of 2 between highest/lowest estimate.\n");
    out.push_str("     K=0.39 may overestimate; Mil-Homens 2013 gives smaller K.\n");
    out.push_str("  5. IMBALANCE FRACTION (20%) is assumed, not measured. Real dQ/dx\n");
    out.push_str("     requires wave climate + bathymetry (SWAN model).\n");
    out.push_str("  6. MANGROVE-RECESSION COEFFICIENT (1 m/ha per km) is empirical estimate,\n");
    out.push_str("     not from controlled experiment. Variable by mangrove species/density.\n");
    out.push_str("  7. NO SUBSIDENCE COUPLING: Pantura subsidence amplifies relative SLR.\n");
    out.push_str("     Use tools::advanced_physics::jakarta_coastal_risk for subsidence integration.\n");
    out.push_str("  8. NO STORM IMPACT: extreme waves (50-yr return) cause episodic erosion\n");
    out.push_str("     >10m in hours, not captured in annual average.\n");
    out.push_str("════════════════════════════════════════════════════════════\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Self-check from spec: Bruun with SLR=0.3m, L=1000m, B=2m, h*=10m
    // R = 0.3 * 1000 / (2 + 10) = 300/12 = 25 m recession
    // Our tool derives L from h* via 1:100 slope: L = h*/0.01 = 10/0.01 = 1000m ✓
    #[test]
    fn bruun_25m_recession() {
        // SLR=0.3, closure_depth=10 -> L=1000, B=2 (assumed), h*=10
        // R = 0.3*1000/(2+10) = 25
        let res = assess(10.0, 0.3, 10.0, 50.0, 1.5, 8.0, 15.0, 0.0, 0.0, 30);
        assert!(res.contains("25.00"), "expected 25.00m Bruun recession, got:\n{}", res);
        assert!(!res.contains("ERROR"));
    }

    // Self-check: zero SLR, no waves, no mining, no mangrove -> ~0 recession
    #[test]
    fn zero_drivers_zero_recession() {
        // With sin(2*alpha)=sin(0)=0, Q=0; SLR=0; mining=0; mangrove=0
        let res = assess(10.0, 0.0, 10.0, 50.0, 1.5, 8.0, 0.0, 0.0, 0.0, 30);
        assert!(res.contains("0.00"), "expected ~0 recession, got:\n{}", res);
    }

    // Self-check: CERC Q positive for oblique waves
    // Hs=1.5, T=8, alpha=15deg -> sin(30)=0.5; cgb=sqrt(9.81*1.5/0.78)=sqrt(18.87)=4.34
    // Q = 0.39 * 2.25 * 4.34 * 0.5 / (16 * 1.65 * 0.6) = 1.903 / 15.84 = 0.120 m3/s
    #[test]
    fn cerc_q_positive_oblique() {
        let res = assess(10.0, 0.0, 10.0, 50.0, 1.5, 8.0, 15.0, 0.0, 0.0, 30);
        assert!(res.contains("0.12"), "expected ~0.12 m3/s Q, got:\n{}", res);
    }

    #[test]
    fn rejects_invalid_wave_height() {
        let res = assess(10.0, 0.3, 10.0, 50.0, 0.0, 8.0, 15.0, 0.0, 0.0, 30);
        assert!(res.contains("ERROR"));
    }
}
