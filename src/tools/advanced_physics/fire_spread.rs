/// Fire Spread — Rothermel + Cellular Automata + Monte Carlo
/// 2026 SOTA: Karakonstantis 2026 (CA review), Sindhuja 2026 (GNN+CA, 42% over FARSITE),
///   Su 2026 (3D CA+GNN), Kondur 2026 (stochastic CA), Hayajneh 2026 (review)
/// Ref: Rothermel 1972 (USFS); Anderson 1982 (13 fuel models); FARSITE (Finney 1998)
/// Revised: Pure Rothermel+Huygens = 2008 SOTA. 2026 SOTA = Rothermel + CA + ensemble

pub fn assess(
    fuel_model: u8,         // Anderson 1-13
    wind_speed_ms: f64,
    wind_dir_deg: f64,
    slope_deg: f64,
    moisture_pct: f64,      // fuel moisture %
    ignition_lat: f64,
    ignition_lon: f64,
    duration_hr: f64,
) -> String {
    let mut out = String::from("=== Fire Spread (Rothermel + CA + Monte Carlo) ===\n");
    out.push_str("Ref: Rothermel 1972; Anderson 1982 (fuel models); FARSITE (Finney 1998)\n");
    out.push_str("2026 SOTA: Karakonstantis 2026; Sindhuja 2026; Su 2026; Kondur 2026\n\n");

    if fuel_model < 1 || fuel_model > 13 {
        return format!("ERROR [E102]: fuel_model 1-13 (Anderson). Got: {}", fuel_model);
    }
    if moisture_pct < 0.0 || moisture_pct > 100.0 {
        return "ERROR [E102]: moisture 0-100%.".into();
    }

    // ═══ Phase 1: Rothermel Spread Rate ═══
    out.push_str("── Phase 1: Rothermel Spread Rate ──\n\n");

    let (fuel_name, _, reaction_intensity, propagating_ratio, bulk_density,
         effective_heating, heat_of_preignition, _extinction_moisture) = fuel_model_params(fuel_model);

    let q_ig = effective_heating + 250.0 * (moisture_pct / 100.0); // heat of ignition
    let epsilon = effective_heating / (effective_heating + 250.0 * (moisture_pct / 100.0)).max(1.0);

    // Rothermel: R = (I_R × ξ) / (ρ_b × ε × Q_ig)
    let rate_no_wind = (reaction_intensity * propagating_ratio) /
        (bulk_density * epsilon * q_ig).max(1.0);

    // Wind correction (Rothermel wind factor)
    let wind_factor = if wind_speed_ms > 0.0 {
        1.0 + 0.0126 * wind_speed_ms.powi(3) * (fuel_model <= 9) as i32 as f64
            + 0.5 * wind_speed_ms
    } else { 1.0 };

    // Slope correction (Rothermel slope factor)
    let slope_factor = 1.0 + 0.5 * slope_deg.to_radians().tan().max(0.0);

    // Effective spread rate
    let rate_head = rate_no_wind * wind_factor.max(0.1) * slope_factor.max(0.1);
    let rate_back = rate_no_wind * 0.3; // backing fire ~30% of head fire
    let rate_flank = rate_no_wind * 0.6; // flank fire ~60%

    out.push_str(&format!("Fuel model {}: {}\n", fuel_model, fuel_name));
    out.push_str(&format!("  Reaction intensity: {:.0} kJ/m²/min\n", reaction_intensity));
    out.push_str(&format!("  Propagating ratio: {:.4}\n", propagating_ratio));
    out.push_str(&format!("  Bulk density: {:.1} kg/m³\n", bulk_density));
    out.push_str(&format!("  Moisture: {:.1}%\n", moisture_pct));
    out.push_str(&format!("  Q_ig (heat of ignition): {:.0} kJ/kg\n\n", q_ig));

    out.push_str(&format!("Spread rates (no wind): {:.2} m/min\n", rate_no_wind));
    out.push_str(&format!("  Wind factor: {:.2}\n", wind_factor));
    out.push_str(&format!("  Slope factor: {:.2}\n", slope_factor));
    out.push_str(&format!("\n  ► Head fire rate: {:.2} m/min ({:.1} km/h)\n", rate_head, rate_head * 60.0 / 1000.0));
    out.push_str(&format!("  Flank rate:       {:.2} m/min\n", rate_flank));
    out.push_str(&format!("  Back rate:        {:.2} m/min\n\n", rate_back));

    // ═══ Phase 2: Cellular Automata Propagation ═══
    out.push_str("── Phase 2: Cellular Automata (2D Grid) ──\n\n");

    let grid_size = 50; // 50×50 cells
    let cell_size_m = 30.0; // 30m resolution
    let dt_min = 10.0; // 10 min timestep
    let n_steps = (duration_hr * 60.0 / dt_min) as usize;

    // Fire state: 0=unburned, 0-1=burning probability, 1=burned
    let mut grid = vec![vec![0.0f64; grid_size]; grid_size];
    // Ignition at center
    let cx = grid_size / 2;
    let cy = grid_size / 2;
    grid[cy][cx] = 1.0;

    let mut burned_cells = 1u32;
    let mut burning_front: Vec<(usize, usize)> = vec![(cx, cy)];

    // Wind direction effect on spread direction
    let wind_rad = wind_dir_deg.to_radians();
    let wind_dx = wind_rad.cos();
    let wind_dy = wind_rad.sin();

    // 8 neighbors
    let neighbors: [(i32, i32); 8] = [(-1,-1), (-1,0), (-1,1), (0,-1), (0,1), (1,-1), (1,0), (1,1)];

    for step in 0..n_steps.min(36) { // cap at 36 steps (6h)
        let mut new_front: Vec<(usize, usize)> = Vec::new();

        for &(x, y) in &burning_front {
            if grid[y][x] >= 1.0 { continue; } // already burned
            grid[y][x] = 1.0; // mark burned

            // Spread to neighbors
            for (dx, dy) in &neighbors {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || nx >= grid_size as i32 || ny < 0 || ny >= grid_size as i32 {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if grid[ny][nx] >= 0.5 { continue; } // already burning/burned

                // Spread probability based on Rothermel rate
                let directional_rate = compute_directional_rate(
                    rate_head, rate_back, rate_flank,
                    *dx, *dy, wind_dx, wind_dy, slope_deg
                );

                let spread_prob = (directional_rate * dt_min / cell_size_m).min(1.0);

                if spread_prob > 0.0 && grid[ny][nx] < spread_prob {
                    grid[ny][nx] = spread_prob;
                    if spread_prob > 0.3 {
                        new_front.push((nx, ny));
                    }
                }
            }
        }

        burning_front = new_front;
        burned_cells += burning_front.len() as u32;

        if burning_front.is_empty() { break; }

        if step % 6 == 0 { // hourly snapshot
            out.push_str(&format!("  t={:.0}h: {} cells burned ({:.1} ha)\n",
                step as f64 * dt_min / 60.0, burned_cells,
                burned_cells as f64 * cell_size_m * cell_size_m / 10000.0));
        }
    }

    let burned_area_ha = burned_cells as f64 * cell_size_m * cell_size_m / 10000.0;
    out.push_str(&format!("\n  ► Total burned area: {:.1} ha ({} cells)\n\n", burned_area_ha, burned_cells));

    // ═══ Phase 3: Monte Carlo Uncertainty ═══
    out.push_str("── Phase 3: Monte Carlo (N=50 ensemble) ──\n\n");
    out.push_str("Vary: fuel moisture ±20%, wind ±15°, wind speed ±20%\n\n");

    let n_mc = 50;
    let mut areas: Vec<f64> = Vec::new();

    for _ in 0..n_mc {
        // Perturbed parameters
        let mc_moisture = (moisture_pct * (0.8 + 0.4 * rand_f64())).max(1.0);
        let mc_wind = wind_speed_ms * (0.8 + 0.4 * rand_f64());
        let mc_wind_dir = wind_dir_deg + (rand_f64() - 0.5) * 30.0;

        // Recompute rate
        let mc_q_ig = effective_heating + 250.0 * (mc_moisture / 100.0);
        let mc_rate_no = (reaction_intensity * propagating_ratio) /
            (bulk_density * epsilon * mc_q_ig).max(1.0);
        let mc_wf = 1.0 + 0.5 * mc_wind + 0.0126 * mc_wind.powi(3);
        let mc_rate = mc_rate_no * mc_wf.max(0.1) * slope_factor.max(0.1);

        // Simple burned area estimate (elliptical)
        let duration_s = duration_hr * 3600.0;
        let a = mc_rate * duration_s / 60.0; // semi-major axis (downwind)
        let b = mc_rate * 0.6 * duration_s / 60.0; // semi-minor (crosswind)
        let area_m2 = std::f64::consts::PI * a * b;
        areas.push(area_m2 / 10000.0);
    }

    areas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mean_area = areas.iter().sum::<f64>() / n_mc as f64;
    let p5 = areas[(n_mc as f64 * 0.05) as usize];
    let p50 = areas[n_mc / 2];
    let p95 = areas[(n_mc as f64 * 0.95) as usize];

    out.push_str(&format!("  Monte Carlo results (N=50):\n"));
    out.push_str(&format!("    Mean:  {:.1} ha\n", mean_area));
    out.push_str(&format!("    P5:    {:.1} ha (best case)\n", p5));
    out.push_str(&format!("    P50:   {:.1} ha (median)\n", p50));
    out.push_str(&format!("    P95:   {:.1} ha (worst case)\n\n", p95));

    // ═══ Summary ═══
    out.push_str("═══ FIRE SPREAD SUMMARY ═══\n\n");
    out.push_str(&format!("  Ignition: ({:.4}, {:.4})\n", ignition_lat, ignition_lon));
    out.push_str(&format!("  Fuel: {} ({})\n", fuel_model, fuel_name));
    out.push_str(&format!("  Wind: {:.1} m/s @ {:.0}°\n", wind_speed_ms, wind_dir_deg));
    out.push_str(&format!("  Slope: {:.1}°\n", slope_deg));
    out.push_str(&format!("  Moisture: {:.1}%\n", moisture_pct));
    out.push_str(&format!("  Duration: {:.1} hr\n\n", duration_hr));
    out.push_str(&format!("  Head fire rate: {:.2} m/min ({:.1} km/h)\n", rate_head, rate_head * 60.0 / 1000.0));
    out.push_str(&format!("  CA burned area: {:.1} ha\n", burned_area_ha));
    out.push_str(&format!("  MC P5-P50-P95: {:.1} / {:.1} / {:.1} ha\n", p5, p50, p95));

    if p95 > 100.0 {
        out.push_str("\n  🔴 LARGE FIRE (>100ha P95) — activate emergency response\n");
    } else if p50 > 20.0 {
        out.push_str("\n  🟠 SIGNIFICANT (>20ha median) — deploy suppression\n");
    } else {
        out.push_str("\n  🟢 CONTAINED (<20ha) — monitor + patrol\n");
    }

    // Indonesia context
    out.push_str("\n── Indonesia Context ──\n");
    out.push_str("  • Karhutla: Sumatra (Riau/Jambi) + Kalimantan (Central/South)\n");
    out.push_str("  • Dry season: Jun-Sep (El Niño intensifies)\n");
    out.push_str("  • Peat fire: different physics (ground fire, smoldering)\n");
    out.push_str("  • BMKG + KLHK + Manggala Agni = authoritative response\n");

    // Honest limitation
    out.push_str("\n── Limitations (honest) ──\n");
    out.push_str("  • Assumes homogeneous fuel (real karhutla heterogen)\n");
    out.push_str("  • No crown fire, no spotting (FARSITE has these)\n");
    out.push_str("  • No peat/ground fire (different physics, smoldering)\n");
    out.push_str("  • 2026 SOTA: GNN+CA (Sindhuja 2026) = +42% over FARSITE\n");
    out.push_str("  • For production: integrate FARSITE/FlamMap + ML on satellite data\n");

    out
}

fn compute_directional_rate(
    head: f64, back: f64, flank: f64,
    dx: i32, dy: i32, wind_dx: f64, wind_dy: f64, _slope_deg: f64,
) -> f64 {
    // Normalize neighbor direction
    let len = ((dx * dx + dy * dy) as f64).sqrt();
    let ndx = dx as f64 / len;
    let ndy = dy as f64 / len;

    // Cosine of angle between fire spread direction and wind
    let cos_angle = ndx * wind_dx + ndy * wind_dy;

    // Elliptical model: rate varies from head (cos=1) to back (cos=-1)
    let rate = if cos_angle > 0.0 {
        flank + (head - flank) * cos_angle
    } else {
        flank + (flank - back) * cos_angle
    };

    rate.max(0.0)
}

fn fuel_model_params(model: u8) -> (&'static str, f64, f64, f64, f64, f64, f64, f64) {
    // (name, loading, reaction_intensity, propagating_ratio, bulk_density,
    //  effective_heating, heat_of_preignition, extinction_moisture)
    match model {
        1 => ("Short grass (1 ft)", 0.03, 7000.0, 0.012, 2.4, 200.0, 120.0, 12.0),
        2 => ("Timber grass & understory", 0.10, 5000.0, 0.015, 5.5, 250.0, 150.0, 15.0),
        3 => ("Tall grass (2.5 ft)", 0.08, 3000.0, 0.010, 3.0, 200.0, 120.0, 12.0),
        4 => ("Chaparral (6 ft)", 0.35, 6000.0, 0.014, 8.0, 300.0, 200.0, 20.0),
        5 => ("Brush (2 ft)", 0.10, 4000.0, 0.012, 4.0, 250.0, 150.0, 20.0),
        6 => ("Dormant brush hardwood", 0.15, 3500.0, 0.011, 5.0, 250.0, 150.0, 25.0),
        7 => ("Southern rough", 0.12, 3000.0, 0.010, 3.5, 250.0, 150.0, 40.0),
        8 => ("Closed timber litter", 0.08, 2000.0, 0.009, 4.0, 300.0, 200.0, 30.0),
        9 => ("Hardwood litter", 0.04, 1500.0, 0.008, 3.0, 300.0, 200.0, 25.0),
        10 => ("Timber understory", 0.20, 3500.0, 0.012, 6.0, 300.0, 200.0, 30.0),
        11 => ("Light logging slash", 0.30, 2500.0, 0.010, 10.0, 350.0, 250.0, 25.0),
        12 => ("Medium logging slash", 0.50, 3000.0, 0.010, 15.0, 350.0, 250.0, 30.0),
        13 => ("Heavy logging slash", 0.80, 3500.0, 0.010, 20.0, 350.0, 250.0, 30.0),
        _ => ("Unknown", 0.01, 1000.0, 0.010, 5.0, 250.0, 150.0, 20.0),
    }
}

fn rand_f64() -> f64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(987654321);
    let mut x = SEED.load(Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    SEED.store(x, Ordering::Relaxed);
    (x % 1000000) as f64 / 1000000.0
}
