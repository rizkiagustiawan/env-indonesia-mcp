/// 2D Shallow Water Equation Solver (SWE)
/// HLL Approximate Riemann Solver pada grid terstruktur
/// Ref: Toro (2001) "Shock-Capturing Methods for Free-Surface Shallow Flows"
/// Saint-Venant Equations: dh/dt + d(hu)/dx + d(hv)/dy = 0

pub struct SweParams {
    pub nx: usize,
    pub ny: usize,
    pub dx: f64,
    pub manning_n: f64,
    pub duration_s: f64,
    pub dt_max: f64,
    pub second_order: bool,
}

pub struct SweResult {
    pub max_depth: f64,
    pub flooded_cells: usize,
    pub total_cells: usize,
    pub flooded_area_m2: f64,
    pub total_volume_m3: f64,
    pub summary: String,
}

/// A single point inflow source (e.g. a manhole / culvert outlet) expressed in
/// grid coordinates with its own discharge.
pub struct InflowSource {
    pub x: usize,
    pub y: usize,
    pub discharge_m3s: f64,
}

pub fn solve(
    dem: &[Vec<f64>],
    params: &SweParams,
    inflow_discharge_m3s: f64,
    inflow_x: usize,
    inflow_y: usize,
    inflow_width: usize,
) -> SweResult {
    let mut sources = Vec::new();
    if inflow_discharge_m3s > 0.0 && inflow_width > 0 {
        let per_cell = inflow_discharge_m3s / inflow_width as f64;
        for w in 0..inflow_width {
            sources.push(InflowSource {
                x: inflow_x,
                y: inflow_y + w,
                discharge_m3s: per_cell,
            });
        }
    }
    solve_multi_source(dem, params, &sources, 0.7)
}

/// Multi-source variant: every `InflowSource` injects independently while
/// `t < duration_s * duty_fraction`. `solve` is a thin wrapper over this.
pub fn solve_multi_source(
    dem: &[Vec<f64>],
    params: &SweParams,
    sources: &[InflowSource],
    duty_fraction: f64,
) -> SweResult {
    let nx = params.nx;
    let ny = params.ny;
    let dx = params.dx;
    let g = 9.81_f64;
    let min_depth = 0.001;

    // Aggregate single-inlet approximation, used ONLY to build the static
    // boundary condition for the AI/FNO surrogate branch below. For the
    // evenly-split `solve` wrapper this reproduces the original values exactly
    // (total discharge, first-source position, cell count as width).
    let inflow_discharge_m3s: f64 = sources.iter().map(|s| s.discharge_m3s).sum();
    let inflow_x = sources.first().map_or(0, |s| s.x);
    let inflow_y = sources.first().map_or(0, |s| s.y);
    let inflow_width = sources.len();

    // === NEW: Deep Tech AI Accelerated FNO Inference with Mass Conservation Gate ===
    if nx > 50 && ny > 50 {
        let mut initial_h_flat = vec![0.0; nx * ny];
        let mut dem_flat = vec![0.0; nx * ny];
        let mut total_inflow_vol = 0.0;
        
        // Initial setup for AI Request
        for i in 0..nx {
            for j in 0..ny {
                dem_flat[j * nx + i] = dem[i][j];
                // Approximate inflow volume representation for AI static boundary
                if i < inflow_x + inflow_width && j == inflow_y {
                    initial_h_flat[j * nx + i] = inflow_discharge_m3s; 
                    total_inflow_vol += inflow_discharge_m3s * params.duration_s;
                }
            }
        }
        
        let initial_volume: f64 = initial_h_flat.iter().sum::<f64>() * dx * dx;

        let req = super::ai_bridge::InferenceRequest {
            site_id: "sumbawa_grid".to_string(),
            bbox: vec![117.0, -8.5, 118.0, -9.0],
            initial_h: initial_h_flat,
            dem: dem_flat,
            width: nx,
            height: ny,
            t_end: params.duration_s,
        };
        
        match super::ai_bridge::call_ai_node(req) {
            Ok(resp) if resp.status == "success" && resp.predicted_h.len() == nx * ny => {
                let predicted_volume: f64 = resp.predicted_h.iter().map(|&v| v.max(0.0)).sum::<f64>() * dx * dx;
                let mass_error_pct = if initial_volume + total_inflow_vol > 0.0 {
                    (predicted_volume - (initial_volume + total_inflow_vol)).abs() / (initial_volume + total_inflow_vol) * 100.0
                } else { 0.0 };

                if mass_error_pct <= 1.0 { // STRICT 1% CONSERVATION GATE
                    let mut max_depth = 0.0;
                    let mut flooded_cells = 0;
                    for &h_val in &resp.predicted_h {
                        if h_val > 0.05 {
                            flooded_cells += 1;
                            if h_val > max_depth { max_depth = h_val; }
                        }
                    }
                    return SweResult {
                        max_depth,
                        flooded_cells,
                        total_cells: nx * ny,
                        flooded_area_m2: flooded_cells as f64 * dx * dx,
                        total_volume_m3: predicted_volume,
                        summary: format!("AI Accelerated (PINO) in {} ms.\nStatus: {}\nMass Balance Error: {:.2}% (Passed Gate)", resp.inference_ms, resp.status, mass_error_pct),
                    };
                } else {
                    println!("Physics Gate Failed: AI Mass Error {:.2}% > 1%. Triggering Numerical Fallback...", mass_error_pct);
                }
            }
            Ok(resp) => println!("AI Gateway error status: {}. Falling back...", resp.status),
            Err(e) => println!("AI Gateway failed ({}). Falling back to CPU numerical solver...", e),
        }
    }
    // === END AI ===

    let mut h = vec![vec![0.0_f64; ny]; nx];
    let mut hu = vec![vec![0.0_f64; ny]; nx];
    let mut hv = vec![vec![0.0_f64; ny]; nx];

    let mut t = 0.0;
    let mut step = 0u64;

    while t < params.duration_s {
        // CFL condition
        let mut max_speed = 0.1_f64;
        for i in 0..nx {
            for j in 0..ny {
                if h[i][j] > min_depth {
                    let u_vel = hu[i][j] / h[i][j];
                    let v_vel = hv[i][j] / h[i][j];
                    let wave_speed = (g * h[i][j]).sqrt();
                    let local_max = (u_vel.abs() + wave_speed).max(v_vel.abs() + wave_speed);
                    if local_max > max_speed { max_speed = local_max; }
                }
            }
        }
        let dt = (0.4 * dx / max_speed).min(params.dt_max).min(params.duration_s - t);
        if dt <= 0.0 { break; }

        // Inflow source term. The injected depth is volume-conserving:
        // depth increment = Q * dt / (dx*dx) for each point source (m^3 of water
        // spread over one cell footprint). Injecting into the interior ensures
        // the finite-volume update below actually advects the source; a boundary
        // cell is never updated (the flux loop runs 1..nx-1 / 1..ny-1).
        if t < params.duration_s * duty_fraction {
            for src in sources {
                if src.discharge_m3s <= 0.0 {
                    continue;
                }
                let jj = src.y.min(ny - 1);
                let ii = src.x.min(nx - 1);
                let ii = ii.clamp(1, nx.saturating_sub(2).max(1));
                let jj = jj.clamp(1, ny.saturating_sub(2).max(1));
                let dh = src.discharge_m3s * dt / (dx * dx);
                h[ii][jj] += dh;
                // Inject momentum so the source has a physical velocity rather than
                // piling up as a static column. Direction follows the local bed slope
                // (water flows downhill); flat beds default to +x.
                let (gx, gy) = downslope_direction(&dem, ii, jj);
                let mag = (gx * gx + gy * gy).sqrt();
                let (ux, uy) = if mag > 1e-9 {
                    (gx / mag, gy / mag)
                } else {
                    (1.0, 0.0)
                };
                let u_inflow = 1.0;
                hu[ii][jj] += dh * u_inflow * ux;
                hv[ii][jj] += dh * u_inflow * uy;
            }
        }

        // Flux computation with HLLC + hydrostatic reconstruction (Audusse et al. 2004).
        // This is a WELL-BALANCED scheme: it preserves the lake-at-rest state
        // (h+z=const, u=v=0) to machine precision, unlike a naive bed-slope source.
        // With second_order=true, MUSCL slope reconstruction (minmod) is applied to the
        // WATER SURFACE elevation η = h+z (and discharge q), NOT the depth h, so the
        // well-balanced property is preserved under 2nd-order reconstruction.
        let mut h_new = h.clone();
        let mut hu_new = hu.clone();
        let mut hv_new = hv.clone();

        // Precompute water surface elevation η = h + z and (optionally) MUSCL slopes.
        let mut eta = vec![vec![0.0_f64; ny]; nx];
        for i in 0..nx {
            for j in 0..ny {
                eta[i][j] = h[i][j] + dem[i][j];
            }
        }
        let mut slope_eta_x = vec![vec![0.0_f64; ny]; nx];
        let mut slope_eta_y = vec![vec![0.0_f64; ny]; nx];
        let mut slope_qx = vec![vec![0.0_f64; ny]; nx];
        let mut slope_qy = vec![vec![0.0_f64; ny]; nx];
        if params.second_order {
            for i in 1..nx - 1 {
                for j in 1..ny - 1 {
                    slope_eta_x[i][j] = minmod(eta[i][j] - eta[i - 1][j], eta[i + 1][j] - eta[i][j]);
                    slope_eta_y[i][j] = minmod(eta[i][j] - eta[i][j - 1], eta[i][j + 1] - eta[i][j]);
                    slope_qx[i][j] = minmod(hu[i][j] - hu[i - 1][j], hu[i + 1][j] - hu[i][j]);
                    slope_qy[i][j] = minmod(hv[i][j] - hv[i][j - 1], hv[i][j + 1] - hv[i][j]);
                }
            }
        }

        for i in 1..nx-1 {
            for j in 1..ny-1 {
                if h[i][j] < min_depth && h[i-1][j] < min_depth && h[i+1][j] < min_depth { continue; }

                let z_c = dem[i][j];
                let eta_c = eta[i][j];

                // ---- X-direction (interfaces i-1/2 and i+1/2) ----
                let z_l = dem[i-1][j];
                let z_r = dem[i+1][j];

                // Left interface (i-1/2): reconstructed η and q.
                let eta_ll = eta[i-1][j] + 0.5 * slope_eta_x[i-1][j]; // cell i-1 right face
                let eta_lr = eta_c - 0.5 * slope_eta_x[i][j];          // cell i left face
                let q_ll = hu[i-1][j] + 0.5 * slope_qx[i-1][j];
                let q_lr = hu[i][j] - 0.5 * slope_qx[i][j];
                let z_star_l = z_l.max(z_c);
                let hl_star_l = (eta_ll - z_star_l).max(0.0);
                let hr_star_l = (eta_lr - z_star_l).max(0.0);
                let u_l = if hl_star_l > min_depth { q_ll / hl_star_l } else { 0.0 };
                let u_cl = if hr_star_l > min_depth { q_lr / hr_star_l } else { 0.0 };
                let (mut fhl, mut fhul) = hllc_flux(hl_star_l, u_l, hr_star_l, u_cl, g);

                // Right interface (i+1/2).
                let eta_rl = eta_c + 0.5 * slope_eta_x[i][j];          // cell i right face
                let eta_rr = eta[i+1][j] - 0.5 * slope_eta_x[i+1][j];  // cell i+1 left face
                let q_rl = hu[i][j] + 0.5 * slope_qx[i][j];
                let q_rr = hu[i+1][j] - 0.5 * slope_qx[i+1][j];
                let z_star_r = z_c.max(z_r);
                let hl_star_r = (eta_rl - z_star_r).max(0.0);
                let hr_star_r = (eta_rr - z_star_r).max(0.0);
                let u_cr = if hl_star_r > min_depth { q_rl / hl_star_r } else { 0.0 };
                let u_r = if hr_star_r > min_depth { q_rr / hr_star_r } else { 0.0 };
                let (mut fhr, mut fhur) = hllc_flux(hl_star_r, u_cr, hr_star_r, u_r, g);

                // Reflective (closed-basin) walls at the domain boundary so the
                // frozen boundary cells act as a wall, not as a mass sink.
                if i == 1 {
                    fhl = 0.0;
                    fhul = 0.0;
                }
                if i == nx - 2 {
                    fhr = 0.0;
                    fhur = 0.0;
                }

                let flux_h_x = (fhr - fhl) / dx;
                let flux_hu_x = (fhur - fhul) / dx;

                // Bed-slope source (Audusse 2004): S = g/2 * [ h*_{i+1/2,L}^2 - h*_{i-1/2,R}^2 ]
                let sx = if i == 1 || i == nx - 2 {
                    0.0
                } else {
                    0.5 * g * (hl_star_r * hl_star_r - hr_star_l * hr_star_l) / dx
                };

                // ---- Y-direction (interfaces j-1/2 and j+1/2) ----
                let z_b = dem[i][j-1];
                let z_t = dem[i][j+1];

                let eta_bb = eta[i][j-1] + 0.5 * slope_eta_y[i][j-1];
                let eta_bt = eta_c - 0.5 * slope_eta_y[i][j];
                let q_bb = hv[i][j-1] + 0.5 * slope_qy[i][j-1];
                let q_bt = hv[i][j] - 0.5 * slope_qy[i][j];
                let z_star_b = z_b.max(z_c);
                let hl_star_b = (eta_bb - z_star_b).max(0.0);
                let hr_star_b = (eta_bt - z_star_b).max(0.0);
                let v_b = if hl_star_b > min_depth { q_bb / hl_star_b } else { 0.0 };
                let v_cb = if hr_star_b > min_depth { q_bt / hr_star_b } else { 0.0 };
                let (mut fhb, mut fhvb) = hllc_flux(hl_star_b, v_b, hr_star_b, v_cb, g);

                let eta_tb = eta_c + 0.5 * slope_eta_y[i][j];
                let eta_tt = eta[i][j+1] - 0.5 * slope_eta_y[i][j+1];
                let q_tb = hv[i][j] + 0.5 * slope_qy[i][j];
                let q_tt = hv[i][j+1] - 0.5 * slope_qy[i][j+1];
                let z_star_t = z_c.max(z_t);
                let hl_star_t = (eta_tb - z_star_t).max(0.0);
                let hr_star_t = (eta_tt - z_star_t).max(0.0);
                let v_ct = if hl_star_t > min_depth { q_tb / hl_star_t } else { 0.0 };
                let v_t = if hr_star_t > min_depth { q_tt / hr_star_t } else { 0.0 };
                let (mut fht, mut fhvt) = hllc_flux(hl_star_t, v_ct, hr_star_t, v_t, g);

                if j == 1 {
                    fhb = 0.0;
                    fhvb = 0.0;
                }
                if j == ny - 2 {
                    fht = 0.0;
                    fhvt = 0.0;
                }

                let flux_h_y = (fht - fhb) / dx;
                let flux_hv_y = (fhvt - fhvb) / dx;

                let sy = if j == 1 || j == ny - 2 {
                    0.0
                } else {
                    0.5 * g * (hl_star_t * hl_star_t - hr_star_b * hr_star_b) / dx
                };

                // Conservative update
                h_new[i][j] = (eta_c - dem[i][j] - dt * (flux_h_x + flux_h_y)).max(0.0);
                hu_new[i][j] = hu[i][j] - dt * flux_hu_x + dt * sx;
                hv_new[i][j] = hv[i][j] - dt * flux_hv_y + dt * sy;

                // Manning friction (semi-implicit)
                if h_new[i][j] > min_depth {
                    let u_mag = ((hu_new[i][j]/h_new[i][j]).powi(2) + (hv_new[i][j]/h_new[i][j]).powi(2)).sqrt();
                    let friction = g * params.manning_n * params.manning_n * u_mag / h_new[i][j].powf(4.0/3.0);
                    let factor = 1.0 / (1.0 + dt * friction);
                    hu_new[i][j] *= factor;
                    hv_new[i][j] *= factor;
                }
            }
        }

        // Mass-conservation correction. The explicit update clips h to >= 0,
        // which silently destroys water whenever the flux over-drains a cell.
        // Interior fluxes telescope and the domain walls are reflective, so any
        // deficit is numerical clipping; rescaling restores the injected volume.
        let vol_before: f64 = h.iter().flatten().sum::<f64>();
        let vol_after: f64 = h_new.iter().flatten().sum::<f64>();
        if vol_after > 0.0 && vol_after < vol_before {
            let scale = vol_before / vol_after;
            for i in 0..nx {
                for j in 0..ny {
                    h_new[i][j] *= scale;
                }
            }
        }

        h = h_new;
        hu = hu_new;
        hv = hv_new;
        t += dt;
        step += 1;
    }

    // Statistik
    let total_volume_m3: f64 = h.iter().flatten().sum::<f64>() * dx * dx;
    let mut max_depth = 0.0_f64;
    let mut flooded = 0usize;
    for i in 0..nx {
        for j in 0..ny {
            if h[i][j] > 0.05 { // > 5cm dianggap tergenang
                flooded += 1;
                if h[i][j] > max_depth { max_depth = h[i][j]; }
            }
        }
    }

    let flooded_area = flooded as f64 * dx * dx;

    let summary = format!(
        "=== 2D SWE Solver Result ===\nRef: Toro (2001); Audusse et al. 2004 (hydrostatic reconstruction)\nSolver: HLLC + Well-Balanced{} (hydrostatic reconstruction)\n\nGrid: {}x{} | dx: {:.0}m\nManning's n: {:.3}\nDuration: {:.0}s ({:.1} jam)\nTimesteps: {}\n\nMax Depth: {:.2} m\nFlooded Cells: {} / {} ({:.1}%)\nFlooded Area: {:.0} m² ({:.2} ha)\nTotal Volume: {:.0} m³\n",
        if params.second_order { " + MUSCL 2nd-order (η-limiting)" } else { "" },
        nx, ny, dx, params.manning_n, params.duration_s, params.duration_s / 3600.0,
        step, max_depth, flooded, nx * ny,
        100.0 * flooded as f64 / (nx * ny) as f64,
        flooded_area, flooded_area / 10000.0,
        total_volume_m3
    );

    SweResult {
        max_depth,
        flooded_cells: flooded,
        total_cells: nx * ny,
        flooded_area_m2: flooded_area,
        total_volume_m3,
        summary,
    }
}

/// Downslope direction at cell (i,j): returns the negative DEM gradient
/// (water flows toward lower elevation), zero at domain edges.
fn downslope_direction(dem: &[Vec<f64>], i: usize, j: usize) -> (f64, f64) {
    let nx = dem.len();
    let ny = dem.first().map_or(0, Vec::len);
    if i == 0 || i + 1 >= nx || j == 0 || j + 1 >= ny {
        return (0.0, 0.0);
    }
    let gx = dem[i - 1][j] - dem[i + 1][j];
    let gy = dem[i][j - 1] - dem[i][j + 1];
    (gx, gy)
}

/// minmod slope limiter: returns 0 if a and b have opposite signs, else the
/// one with smaller magnitude (TVD, prevents spurious oscillations).
fn minmod(a: f64, b: f64) -> f64 {
    if a * b <= 0.0 {
        0.0
    } else if a.abs() < b.abs() {
        a
    } else {
        b
    }
}

/// HLL approximate Riemann solver flux (Toro 2001).
/// Returns (F_h, F_hu) = flux of mass and momentum for 1D SWE.
/// Einfeldt wave speeds: SL = min(uL-cL, uR-cR), SR = max(uL+cL, uR+cR).
fn hll_flux(hl: f64, ul: f64, hr: f64, ur: f64, g: f64) -> (f64, f64) {
    let cl = (g * hl).sqrt();
    let cr = (g * hr).sqrt();
    let sl = (ul - cl).min(ur - cr);
    let sr = (ul + cl).max(ur + cr);

    // Left/right flux vectors: F = (h*u, h*u^2 + 0.5*g*h^2)
    let fl_h = hl * ul;
    let fl_hu = hl * ul * ul + 0.5 * g * hl * hl;
    let fr_h = hr * ur;
    let fr_hu = hr * ur * ur + 0.5 * g * hr * hr;

    if sl >= 0.0 {
        // Supercritical left-to-right: use left flux
        (fl_h, fl_hu)
    } else if sr <= 0.0 {
        // Supercritical right-to-left: use right flux
        (fr_h, fr_hu)
    } else {
        // Subcritical: HLL middle-state flux
        let f_star_h = (sr * fl_h - sl * fr_h + sl * sr * (hr - hl)) / (sr - sl);
        let f_star_hu = (sr * fl_hu - sl * fr_hu + sl * sr * (hr * ur - hl * ul)) / (sr - sl);
        (f_star_h, f_star_hu)
    }
}

/// HLLC (contact-wave-resolving) approximate Riemann solver flux (Toro 2001).
/// Resolves the middle contact wave S_*, giving sharper profiles than HLL.
/// Falls back to HLL for dry-bed states (h ~ 0) to avoid division by zero.
fn hllc_flux(hl: f64, ul: f64, hr: f64, ur: f64, g: f64) -> (f64, f64) {
    let eps = 1e-9;
    if hl < eps && hr < eps {
        return (0.0, 0.0);
    }
    if hl < eps || hr < eps {
        return hll_flux(hl, ul, hr, ur, g); // dry bed: HLL (no contact wave)
    }
    let cl = (g * hl).sqrt();
    let cr = (g * hr).sqrt();
    let sl = (ul - cl).min(ur - cr);
    let sr = (ul + cl).max(ur + cr);

    let fl_h = hl * ul;
    let fl_hu = hl * ul * ul + 0.5 * g * hl * hl;
    let fr_h = hr * ur;
    let fr_hu = hr * ur * ur + 0.5 * g * hr * hr;

    // Middle (star) wave speed (Toro 2001): S_L weights the right state, S_R the left.
    let denom = hr * (ur - sr) - hl * (ul - sl);
    let s_star = if denom.abs() < 1e-12 {
        0.5 * (sl + sr)
    } else {
        (sl * hr * (ur - sr) - sr * hl * (ul - sl)) / denom
    };

    if sl >= 0.0 {
        (fl_h, fl_hu)
    } else if s_star >= 0.0 {
        let h_star = hl * (ul - sl) / (s_star - sl);
        (fl_h + sl * (h_star - hl), fl_hu + sl * (h_star * s_star - hl * ul))
    } else if sr >= 0.0 {
        let h_star = hr * (ur - sr) / (s_star - sr);
        (fr_h + sr * (h_star - hr), fr_hu + sr * (h_star * s_star - hr * ur))
    } else {
        (fr_h, fr_hu)
    }
}

#[cfg(test)]
mod tests {
    use super::{hll_flux, hllc_flux};
    // Self-check: HLL flux for still water (hL=hR=h, uL=uR=0) should give zero mass flux
    // and hydrostatic pressure flux 0.5*g*h^2 (balanced by source in well-balanced scheme).
    #[test]
    fn hll_still_water() {
        let g = 9.81_f64;
        let (fh, fhu) = hll_flux(1.0, 0.0, 1.0, 0.0, g);
        assert!(fh.abs() < 1e-10, "mass flux for still water must be 0, got {fh}");
        assert!((fhu - 0.5 * g * 1.0 * 1.0).abs() < 1e-6, "momentum flux = 0.5gh^2 = {:.4}, got {fhu}", 0.5*g);
    }

    // Self-check: left-to-right supercritical flow (SL >= 0) uses left flux
    #[test]
    fn hll_supercritical_left() {
        let g = 9.81_f64;
        let h = 1.0; let u = 10.0; // both states supercritical left-to-right
        let (fh, _fhu) = hll_flux(h, u, h, u, g);
        assert!((fh - h * u).abs() < 1e-6, "supercritical left flux fh={fh} expected {}", h*u);
    }

    #[test]
    fn hllc_still_water_flux() {
        let g = 9.81_f64;
        let (fh, fhu) = hllc_flux(1.0, 0.0, 1.0, 0.0, g);
        assert!(fh.abs() < 1e-10);
        assert!((fhu - 0.5 * g).abs() < 1e-9);
    }

    #[test]
    fn hllc_dam_break_positive_flux() {
        let g = 9.81_f64;
        // Dam break: high water left (1.0m) → low water right (0.1m), both at rest.
        // Flow must be rightward (positive mass flux) for both HLL and HLLC.
        let (fh_hllc, fhu_hllc) = hllc_flux(1.0, 0.0, 0.1, 0.0, g);
        let (fh_hll, fhu_hll) = hll_flux(1.0, 0.0, 0.1, 0.0, g);
        assert!(fh_hllc.is_finite() && fh_hllc > 0.0, "HLLC mass flux must be positive");
        assert!(fhu_hllc.is_finite());
        assert!(fh_hll.is_finite() && fh_hll > 0.0, "HLL mass flux must be positive");
        assert!(fhu_hll.is_finite());
    }

    // KEY well-balancing test: for lake-at-rest (h+z=const, u=0) the Audusse
    // hydrostatic-reconstruction source term must EXACTLY cancel the flux difference,
    // so no spurious momentum is generated over uneven topography.
    #[test]
    fn well_balanced_lake_at_rest() {
        let g = 9.81_f64;
        let eta = 3.0_f64; // flat water surface
        let (zl, zc, zr) = (0.0_f64, 1.0_f64, 2.0_f64);
        let (hl, hc, hr) = (eta - zl, eta - zc, eta - zr);

        // Left interface (i-1/2)
        let z_star_l = zl.max(zc);
        let hl_star_l = (hl + zl - z_star_l).max(0.0);
        let hr_star_l = (hc + zc - z_star_l).max(0.0);
        let (_fl_h, fl_hu) = hllc_flux(hl_star_l, 0.0, hr_star_l, 0.0, g);

        // Right interface (i+1/2)
        let z_star_r = zc.max(zr);
        let hl_star_r = (hc + zc - z_star_r).max(0.0);
        let hr_star_r = (hr + zr - z_star_r).max(0.0);
        let (_fr_h, fr_hu) = hllc_flux(hl_star_r, 0.0, hr_star_r, 0.0, g);

        let dx = 1.0;
        let flux_hu_x = (fr_hu - fl_hu) / dx;
        let sx = 0.5 * g * (hl_star_r * hl_star_r - hr_star_l * hr_star_l) / dx;

        // Net momentum update: -dt*flux_hu_x + dt*sx must be zero.
        assert!(
            (-flux_hu_x + sx).abs() < 1e-9,
            "NOT well-balanced: flux={} source={} net={}",
            flux_hu_x, sx, -flux_hu_x + sx
        );
    }
}


#[cfg(test)]
mod muscl_tests {
    use super::{minmod, solve, solve_multi_source, InflowSource, SweParams, SweResult};

    #[test]
    fn minmod_limiter_behavior() {
        // Same sign → smaller magnitude.
        assert_eq!(minmod(2.0, 3.0), 2.0);
        assert_eq!(minmod(-2.0, -3.0), -2.0);
        // Opposite sign → 0 (no oscillation).
        assert_eq!(minmod(2.0, -3.0), 0.0);
        // Zero → 0.
        assert_eq!(minmod(0.0, 5.0), 0.0);
    }

    #[test]
    fn second_order_solve_runs_stable() {
        // Uneven bed + inflow; second-order MUSCL must not produce NaN.
        let nx = 20; let ny = 20;
        let mut dem = vec![vec![0.0; ny]; nx];
        for i in 0..nx {
            for j in 0..ny {
                dem[i][j] = ((i as f64 - 10.0) * 0.05).abs() + ((j as f64 - 10.0) * 0.03).abs();
            }
        }
        let params = SweParams {
            nx, ny, dx: 10.0, manning_n: 0.03, duration_s: 60.0, dt_max: 1.0,
            second_order: true,
        };
        let res: SweResult = solve(&dem, &params, 5.0, 2, 10, 3);
        assert!(res.max_depth.is_finite());
        assert!(res.summary.contains("MUSCL 2nd-order"));
    }

    #[test]
    fn inflow_at_boundary_cell_spreads_with_bounded_depth() {
        // Inflow at a boundary cell must not pile up into a non-physical column:
        // the source must enter the interior so the finite-volume update advects it.
        let nx = 16;
        let ny = 16;
        let dem = vec![vec![1.0; ny]; nx];
        let params = SweParams {
            nx,
            ny,
            dx: 10.0,
            manning_n: 0.03,
            duration_s: 300.0,
            dt_max: 0.5,
            second_order: false,
        };
        let res: SweResult = solve(&dem, &params, 50.0, 0, 0, 1);
        assert!(res.max_depth.is_finite());
        // Mass conservation: total volume must match the injected volume
        // (Q * 0.7 * duration) regardless of how it redistributes.
        let expected_volume = 50.0 * 0.7 * 300.0;
        let ratio = res.total_volume_m3 / expected_volume;
        assert!(
            (ratio - 1.0).abs() < 0.05,
            "mass not conserved: volume={} expected={}",
            res.total_volume_m3,
            expected_volume
        );
        assert!(
            res.flooded_cells > 1,
            "water must spread beyond the single injection cell"
        );
        // Single-cell theoretical max = expected_volume / (dx*dx) = 105 m.
        // A boundary-cell pile-up bug previously produced ~224 m in one cell.
        assert!(
            res.max_depth <= expected_volume / (params.dx * params.dx) * 1.05,
            "max depth {} m exceeds the single-cell volume bound",
            res.max_depth
        );
    }

    #[test]
    fn multi_source_conserves_total_injected_volume() {
        let dem = vec![vec![10.0; 12]; 12];
        let params = SweParams { nx: 12, ny: 12, dx: 10.0, manning_n: 0.03, duration_s: 60.0, dt_max: 0.5, second_order: false };
        let sources = vec![
            InflowSource { x: 3, y: 3, discharge_m3s: 2.0 },
            InflowSource { x: 8, y: 8, discharge_m3s: 3.0 },
        ];
        let res = solve_multi_source(&dem, &params, &sources, 1.0);
        let expected = (2.0 + 3.0) * 60.0;
        let ratio = res.total_volume_m3 / expected;
        assert!(ratio > 0.95 && ratio < 1.05, "expected ~{expected}, got {}", res.total_volume_m3);
    }

    #[test]
    fn solve_delegates_to_multi_source_with_same_result() {
        let dem = vec![vec![10.0; 8]; 8];
        let params = SweParams { nx: 8, ny: 8, dx: 10.0, manning_n: 0.03, duration_s: 30.0, dt_max: 0.5, second_order: false };
        let via_solve = solve(&dem, &params, 4.0, 3, 3, 2);
        let sources = vec![
            InflowSource { x: 3, y: 3, discharge_m3s: 2.0 },
            InflowSource { x: 3, y: 4, discharge_m3s: 2.0 },
        ];
        let via_multi = solve_multi_source(&dem, &params, &sources, 0.7);
        assert!((via_solve.total_volume_m3 - via_multi.total_volume_m3).abs() < 1e-6);
    }

    #[test]
    fn zero_sources_produces_no_water() {
        let dem = vec![vec![10.0; 6]; 6];
        let params = SweParams { nx: 6, ny: 6, dx: 10.0, manning_n: 0.03, duration_s: 10.0, dt_max: 0.5, second_order: false };
        let res = solve_multi_source(&dem, &params, &[], 1.0);
        assert_eq!(res.flooded_cells, 0);
        assert!(res.total_volume_m3 < 1e-9);
    }
}
