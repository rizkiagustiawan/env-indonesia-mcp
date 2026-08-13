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

        // Flux computation (HLL) - X direction
        let mut h_new = h.clone();
        let mut hu_new = hu.clone();
        let mut hv_new = hv.clone();

        for i in 1..nx-1 {
            for j in 1..ny-1 {
                if h[i][j] < min_depth && h[i-1][j] < min_depth && h[i+1][j] < min_depth { continue; }

                let _z_c = dem[i][j];
                let z_l = dem[i-1][j];
                let z_r = dem[i+1][j];
                let z_b = dem[i][j-1];
                let z_t = dem[i][j+1];

                // X-direction HLL Riemann solver (Toro 2001)
                // BUG FIX: previous code was central-differencing with dimensionally-broken
                // gravity term and non-conservative momentum update. Replaced with proper HLL.
                let h_l = h[i-1][j]; let h_c = h[i][j]; let h_r = h[i+1][j];
                let u_l = if h_l > min_depth { hu[i-1][j]/h_l } else { 0.0 };
                let u_c = if h_c > min_depth { hu[i][j]/h_c } else { 0.0 };
                let u_r = if h_r > min_depth { hu[i+1][j]/h_r } else { 0.0 };

                // Left state at interface (i-1/2)
                let (fhl, fhul) = hll_flux(h_l, u_l, h_c, u_c, g);
                // Right state at interface (i+1/2)
                let (fhr, fhur) = hll_flux(h_c, u_c, h_r, u_r, g);

                let flux_h_x = (fhr - fhl) / dx;
                let flux_hu_x = (fhur - fhul) / dx;

                // Y-direction HLL
                let h_b = h[i][j-1]; let h_t = h[i][j+1];
                let v_b = if h_b > min_depth { hv[i][j-1]/h_b } else { 0.0 };
                let v_c = if h_c > min_depth { hv[i][j]/h_c } else { 0.0 };
                let v_t = if h_t > min_depth { hv[i][j+1]/h_t } else { 0.0 };

                let (fhb, fhvb) = hll_flux(h_b, v_b, h_c, v_c, g);
                let (fht, fhvt) = hll_flux(h_c, v_c, h_t, v_t, g);

                let flux_h_y = (fht - fhb) / dx;
                let flux_hv_y = (fhvt - fhvb) / dx;

                // Conservative update: dU/dt + dF/dx + dG/dy = S
                h_new[i][j] = (h_c - dt * (flux_h_x + flux_h_y)).max(0.0);

                // Bed slope source term
                let sx = -g * h_c * (dem[i+1][j] - dem[i-1][j]) / (2.0 * dx);
                let sy = -g * h_c * (dem[i][j+1] - dem[i][j-1]) / (2.0 * dx);

                hu_new[i][j] = hu[i][j] - dt * flux_hu_x + sx * dt;
                hv_new[i][j] = hv[i][j] - dt * flux_hv_y + sy * dt;

                // Manning friction
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
        "=== 2D SWE Solver Result ===\nRef: Toro (2001), Saint-Venant Equations\nSolver: HLL Approximate Riemann\n\nGrid: {}x{} | dx: {:.0}m\nManning's n: {:.3}\nDuration: {:.0}s ({:.1} jam)\nTimesteps: {}\n\nMax Depth: {:.2} m\nFlooded Cells: {} / {} ({:.1}%)\nFlooded Area: {:.0} m² ({:.2} ha)\n",
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

#[cfg(test)]
mod tests {
    use super::hll_flux;
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
        let cl = (g * h).sqrt(); // 3.13
        // SL = min(uL-cL, uR-cR) = min(10-3.13, 10-3.13) = 6.87 > 0 -> left flux
        let (fh, fhu) = hll_flux(h, u, h, u, g);
        assert!((fh - h * u).abs() < 1e-6, "supercritical left flux fh={fh} expected {}", h*u);
    }
}

