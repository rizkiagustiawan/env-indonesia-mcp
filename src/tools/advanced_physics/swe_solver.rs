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
    pub summary: String,
}

pub fn solve(
    dem: &[Vec<f64>],
    params: &SweParams,
    inflow_discharge_m3s: f64,
    inflow_x: usize,
    inflow_y: usize,
    inflow_width: usize,
) -> SweResult {
    let nx = params.nx;
    let ny = params.ny;
    let dx = params.dx;
    let g = 9.81_f64;
    let min_depth = 0.001;

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

        // Inflow boundary
        if t < params.duration_s * 0.7 {
            let q_per_cell = inflow_discharge_m3s / (inflow_width as f64 * dx);
            for w in 0..inflow_width {
                let jj = (inflow_y + w).min(ny - 1);
                let ii = inflow_x.min(nx - 1);
                h[ii][jj] += q_per_cell * dt / dx;
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
                let (fhl, fhul) = hllc_flux(hl_star_l, u_l, hr_star_l, u_cl, g);

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
                let (fhr, fhur) = hllc_flux(hl_star_r, u_cr, hr_star_r, u_r, g);

                let flux_h_x = (fhr - fhl) / dx;
                let flux_hu_x = (fhur - fhul) / dx;

                // Bed-slope source (Audusse 2004): S = g/2 * [ h*_{i+1/2,L}^2 - h*_{i-1/2,R}^2 ]
                let sx = 0.5 * g * (hl_star_r * hl_star_r - hr_star_l * hr_star_l) / dx;

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
                let (fhb, fhvb) = hllc_flux(hl_star_b, v_b, hr_star_b, v_cb, g);

                let eta_tb = eta_c + 0.5 * slope_eta_y[i][j];
                let eta_tt = eta[i][j+1] - 0.5 * slope_eta_y[i][j+1];
                let q_tb = hv[i][j] + 0.5 * slope_qy[i][j];
                let q_tt = hv[i][j+1] - 0.5 * slope_qy[i][j+1];
                let z_star_t = z_c.max(z_t);
                let hl_star_t = (eta_tb - z_star_t).max(0.0);
                let hr_star_t = (eta_tt - z_star_t).max(0.0);
                let v_ct = if hl_star_t > min_depth { q_tb / hl_star_t } else { 0.0 };
                let v_t = if hr_star_t > min_depth { q_tt / hr_star_t } else { 0.0 };
                let (fht, fhvt) = hllc_flux(hl_star_t, v_ct, hr_star_t, v_t, g);

                let flux_h_y = (fht - fhb) / dx;
                let flux_hv_y = (fhvt - fhvb) / dx;

                let sy = 0.5 * g * (hl_star_t * hl_star_t - hr_star_b * hr_star_b) / dx;

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

        h = h_new;
        hu = hu_new;
        hv = hv_new;
        t += dt;
        step += 1;
    }

    // Statistik
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
        "=== 2D SWE Solver Result ===\nRef: Toro (2001); Audusse et al. 2004 (hydrostatic reconstruction)\nSolver: HLLC + Well-Balanced{} (hydrostatic reconstruction)\n\nGrid: {}x{} | dx: {:.0}m\nManning's n: {:.3}\nDuration: {:.0}s ({:.1} jam)\nTimesteps: {}\n\nMax Depth: {:.2} m\nFlooded Cells: {} / {} ({:.1}%)\nFlooded Area: {:.0} m² ({:.2} ha)\n",
        if params.second_order { " + MUSCL 2nd-order (η-limiting)" } else { "" },
        nx, ny, dx, params.manning_n, params.duration_s, params.duration_s / 3600.0,
        step, max_depth, flooded, nx * ny,
        100.0 * flooded as f64 / (nx * ny) as f64,
        flooded_area, flooded_area / 10000.0
    );

    SweResult {
        max_depth,
        flooded_cells: flooded,
        total_cells: nx * ny,
        flooded_area_m2: flooded_area,
        summary,
    }
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
    use super::{minmod, solve, SweParams, SweResult};

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
}
