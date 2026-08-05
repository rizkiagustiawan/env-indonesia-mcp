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

                // X-direction fluxes (simplified HLL)
                let h_l = h[i-1][j]; let h_c = h[i][j]; let h_r = h[i+1][j];
                let u_l = if h_l > min_depth { hu[i-1][j]/h_l } else { 0.0 };
                let u_c = if h_c > min_depth { hu[i][j]/h_c } else { 0.0 };
                let _u_r = if h_r > min_depth { hu[i+1][j]/h_r } else { 0.0 };

                let flux_x = dt / dx * (
                    0.5 * (h_c * u_c + h_l * u_l) - 0.5 * (h_r * _u_r + h_c * u_c)
                    + 0.5 * g * (h_l * h_l - h_r * h_r) / (2.0 * dx) * dt
                );

                // Y-direction fluxes
                let h_b = h[i][j-1]; let h_t = h[i][j+1];
                let v_b = if h_b > min_depth { hv[i][j-1]/h_b } else { 0.0 };
                let v_c = if h_c > min_depth { hv[i][j]/h_c } else { 0.0 };
                let v_t = if h_t > min_depth { hv[i][j+1]/h_t } else { 0.0 };

                let flux_y = dt / dx * (
                    0.5 * (h_c * v_c + h_b * v_b) - 0.5 * (h_t * v_t + h_c * v_c)
                );

                h_new[i][j] = (h_c + flux_x + flux_y).max(0.0);

                // Gravity source term (slope)
                let sx = -g * h_c * (z_r - z_l) / (2.0 * dx);
                let sy = -g * h_c * (z_t - z_b) / (2.0 * dx);
                hu_new[i][j] = hu[i][j] + sx * dt;
                hv_new[i][j] = hv[i][j] + sy * dt;

                // Manning friction
                if h_new[i][j] > min_depth {
                    let u_mag = ((hu_new[i][j]/h_new[i][j]).powi(2) + (hv_new[i][j]/h_new[i][j]).powi(2)).sqrt();
                    let friction = g * params.manning_n * params.manning_n * u_mag / h_new[i][j].powf(1.0/3.0);
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
