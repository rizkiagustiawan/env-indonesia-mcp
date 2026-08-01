//! 2D Shallow Water Equations (SWE) Flood Solver
//!
//! Physics-based flood simulation using Finite Volume Method
//! with HLL (Harten-Lax-van Leer) approximate Riemann solver.
//!
//! Equations (conservative form):
//!   ∂h/∂t + ∂(hu)/∂x + ∂(hv)/∂y = R(t)
//!   ∂(hu)/∂t + ∂(hu² + gh²/2)/∂x + ∂(huv)/∂y = -ghS₀ₓ - ghSfₓ
//!   ∂(hv)/∂t + ∂(huv)/∂x + ∂(hv² + gh²/2)/∂y = -ghS₀ᵧ - ghSfᵧ
//!
//! Where: h=depth, u,v=velocity, g=gravity, S₀=bed slope, Sf=friction (Manning)
//!
//! Ref: Toro 2001 (Shock-Capturing Methods), LeVeque 2002 (Finite Volume Methods)

use serde::{Deserialize, Serialize};

const G: f64 = 9.81; // gravitational acceleration (m/s²)
const H_MIN: f64 = 1.0e-6; // minimum depth threshold for wet/dry
const CFL: f64 = 0.45; // CFL number for stability

/// Cell state: conserved variables [h, hu, hv]
#[derive(Clone, Copy, Debug)]
struct CellState {
    h: f64,  // water depth (m)
    hu: f64, // x-momentum (m²/s)
    hv: f64, // y-momentum (m²/s)
}

impl CellState {
    fn new() -> Self {
        CellState {
            h: 0.0,
            hu: 0.0,
            hv: 0.0,
        }
    }

    fn velocity_u(&self) -> f64 {
        if self.h > H_MIN {
            self.hu / self.h
        } else {
            0.0
        }
    }

    fn velocity_v(&self) -> f64 {
        if self.h > H_MIN {
            self.hv / self.h
        } else {
            0.0
        }
    }

    fn wave_speed(&self) -> f64 {
        if self.h > H_MIN {
            (G * self.h).sqrt()
        } else {
            0.0
        }
    }

    fn is_wet(&self) -> bool {
        self.h > H_MIN
    }
}

/// HLL flux in x-direction
fn hll_flux_x(left: &CellState, right: &CellState) -> (f64, f64, f64) {
    let ul = left.velocity_u();
    let ur = right.velocity_u();
    let cl = left.wave_speed();
    let cr = right.wave_speed();

    // Wave speed estimates (Einfeldt)
    let sl = (ul - cl).min(ur - cr);
    let sr = (ul + cl).max(ur + cr);

    // Fluxes
    let fl = (
        left.hu,
        left.hu * ul + 0.5 * G * left.h * left.h,
        left.hu * left.velocity_v(),
    );
    let fr = (
        right.hu,
        right.hu * ur + 0.5 * G * right.h * right.h,
        right.hu * right.velocity_v(),
    );

    if sl >= 0.0 {
        fl
    } else if sr <= 0.0 {
        fr
    } else {
        let denom = sr - sl;
        if denom.abs() < 1e-12 {
            return (0.0, 0.0, 0.0);
        }
        (
            (sr * fl.0 - sl * fr.0 + sr * sl * (right.h - left.h)) / denom,
            (sr * fl.1 - sl * fr.1 + sr * sl * (right.hu - left.hu)) / denom,
            (sr * fl.2 - sl * fr.2 + sr * sl * (right.hv - left.hv)) / denom,
        )
    }
}

/// HLL flux in y-direction
fn hll_flux_y(bottom: &CellState, top: &CellState) -> (f64, f64, f64) {
    let vb = bottom.velocity_v();
    let vt = top.velocity_v();
    let cb = bottom.wave_speed();
    let ct = top.wave_speed();

    let sl = (vb - cb).min(vt - ct);
    let sr = (vb + cb).max(vt + ct);

    let fb = (
        bottom.hv,
        bottom.hv * bottom.velocity_u(),
        bottom.hv * vb + 0.5 * G * bottom.h * bottom.h,
    );
    let ft = (
        top.hv,
        top.hv * top.velocity_u(),
        top.hv * vt + 0.5 * G * top.h * top.h,
    );

    if sl >= 0.0 {
        fb
    } else if sr <= 0.0 {
        ft
    } else {
        let denom = sr - sl;
        if denom.abs() < 1e-12 {
            return (0.0, 0.0, 0.0);
        }
        (
            (sr * fb.0 - sl * ft.0 + sr * sl * (top.h - bottom.h)) / denom,
            (sr * fb.1 - sl * ft.1 + sr * sl * (top.hu - bottom.hu)) / denom,
            (sr * fb.2 - sl * ft.2 + sr * sl * (top.hv - bottom.hv)) / denom,
        )
    }
}

/// Manning friction source term (implicit treatment for stability)
fn manning_friction(state: &CellState, n: f64, dt: f64) -> CellState {
    if !state.is_wet() || n <= 0.0 {
        return *state;
    }

    let u = state.velocity_u();
    let v = state.velocity_v();
    let speed = (u * u + v * v).sqrt();

    if speed < 1e-12 {
        return *state;
    }

    // Implicit friction: Sf = n² * |v| * v / h^(4/3)
    let h43 = state.h.powf(4.0 / 3.0);
    let cf = G * n * n * speed / h43;
    let factor = 1.0 / (1.0 + cf * dt);

    CellState {
        h: state.h,
        hu: state.hu * factor,
        hv: state.hv * factor,
    }
}

/// Compute adaptive timestep from CFL condition
fn compute_dt(grid: &[Vec<CellState>], dx: f64) -> f64 {
    let mut max_speed = 1e-10;
    for row in grid {
        for cell in row {
            if cell.is_wet() {
                let u = cell.velocity_u().abs();
                let v = cell.velocity_v().abs();
                let c = cell.wave_speed();
                let speed = (u + c).max(v + c);
                if speed > max_speed {
                    max_speed = speed;
                }
            }
        }
    }
    CFL * dx / max_speed
}

/// SWE simulation parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct SweParams {
    pub nx: usize,              // grid cells in x
    pub ny: usize,              // grid cells in y
    pub dx: f64,                // cell size (m)
    pub manning_n: f64,         // Manning's roughness coefficient
    pub duration_s: f64,        // simulation duration (seconds)
    pub output_interval_s: f64, // output snapshot interval
}

/// Inflow boundary condition
#[derive(Debug, Deserialize, Serialize)]
pub struct InflowBC {
    pub cell_x: usize,      // inflow cell x-index
    pub cell_y: usize,      // inflow cell y-index
    pub width_cells: usize, // inflow width in cells
    pub discharge_m3s: f64, // discharge Q (m³/s)
    pub start_s: f64,       // start time (s)
    pub end_s: f64,         // end time (s)
}

/// Run 2D SWE simulation
pub fn simulate_flood(
    dem: &[Vec<f64>], // DEM elevations (m), row-major [ny][nx]
    params: &SweParams,
    inflow: &InflowBC,
) -> String {
    let ny = params.ny.min(dem.len());
    let nx = params.nx.min(if dem.is_empty() { 0 } else { dem[0].len() });

    if nx < 3 || ny < 3 {
        return "ERROR: Grid terlalu kecil. Minimal 3x3.".to_string();
    }

    let dx = params.dx;
    let manning = params.manning_n;

    // Initialize grid — dry everywhere
    let mut grid: Vec<Vec<CellState>> = vec![vec![CellState::new(); nx]; ny];

    let mut time = 0.0;
    let mut step = 0u64;
    let mut max_depth: f64 = 0.0;
    let mut max_velocity: f64 = 0.0;
    let mut total_volume: f64 = 0.0;
    let mut snapshots: Vec<String> = Vec::new();
    let mut next_output = params.output_interval_s;

    // Time stepping loop
    while time < params.duration_s {
        // Adaptive timestep
        let dt = compute_dt(&grid, dx).min(params.duration_s - time).min(1.0);

        if dt < 1e-12 {
            break;
        }

        // Apply inflow BC
        if time >= inflow.start_s && time <= inflow.end_s {
            let q_per_cell = inflow.discharge_m3s / (inflow.width_cells.max(1) as f64 * dx);
            for di in 0..inflow.width_cells {
                let ix = (inflow.cell_x + di).min(nx - 1);
                let iy = inflow.cell_y.min(ny - 1);
                grid[iy][ix].h += q_per_cell * dt / dx;
                grid[iy][ix].hu += q_per_cell * dt / dx * 0.1; // small initial velocity
            }
        }

        // Compute fluxes and update (Godunov splitting)
        let mut new_grid = grid.clone();

        // X-direction fluxes
        for j in 0..ny {
            for i in 0..nx - 1 {
                let flux = hll_flux_x(&grid[j][i], &grid[j][i + 1]);
                let ratio = dt / dx;
                new_grid[j][i].h -= ratio * flux.0;
                new_grid[j][i].hu -= ratio * flux.1;
                new_grid[j][i].hv -= ratio * flux.2;
                new_grid[j][i + 1].h += ratio * flux.0;
                new_grid[j][i + 1].hu += ratio * flux.1;
                new_grid[j][i + 1].hv += ratio * flux.2;
            }
        }

        // Y-direction fluxes
        for j in 0..ny - 1 {
            for i in 0..nx {
                let flux = hll_flux_y(&new_grid[j][i], &new_grid[j + 1][i]);
                let ratio = dt / dx;
                new_grid[j][i].h -= ratio * flux.0;
                new_grid[j][i].hu -= ratio * flux.1;
                new_grid[j][i].hv -= ratio * flux.2;
                new_grid[j + 1][i].h += ratio * flux.0;
                new_grid[j + 1][i].hu += ratio * flux.1;
                new_grid[j + 1][i].hv += ratio * flux.2;
            }
        }

        // Bed slope source terms — improved wet/dry boundary handling
        // Ref: Audusse et al. (2004), LeVeque (2002)
        // NOTE: Full well-balancing (exact lake-at-rest on sloped bed)
        // requires hydrostatic reconstruction in the HLL flux. This
        // simplified treatment guards against spurious flow at wet/dry
        // interfaces by clamping dry-neighbor bed elevations.
        for j in 1..ny - 1 {
            for i in 1..nx - 1 {
                if new_grid[j][i].is_wet() {
                    let z_c = dem[j][i];
                    let z_e = if new_grid[j][i + 1].is_wet() {
                        dem[j][i + 1]
                    } else {
                        z_c + new_grid[j][i].h
                    };
                    let z_w = if new_grid[j][i - 1].is_wet() {
                        dem[j][i - 1]
                    } else {
                        z_c + new_grid[j][i].h
                    };
                    let z_n = if new_grid[j + 1][i].is_wet() {
                        dem[j + 1][i]
                    } else {
                        z_c + new_grid[j][i].h
                    };
                    let z_s = if new_grid[j - 1][i].is_wet() {
                        dem[j - 1][i]
                    } else {
                        z_c + new_grid[j][i].h
                    };

                    let s0x = -(z_e - z_w) / (2.0 * dx);
                    let s0y = -(z_n - z_s) / (2.0 * dx);
                    new_grid[j][i].hu += dt * G * new_grid[j][i].h * s0x;
                    new_grid[j][i].hv += dt * G * new_grid[j][i].h * s0y;
                }
            }
        }

        // Manning friction (implicit)
        for j in 0..ny {
            for i in 0..nx {
                new_grid[j][i] = manning_friction(&new_grid[j][i], manning, dt);
                // Clamp negative depths
                if new_grid[j][i].h < H_MIN {
                    new_grid[j][i] = CellState::new();
                }
            }
        }

        grid = new_grid;
        time += dt;
        step += 1;

        // Track statistics
        for j in 0..ny {
            for i in 0..nx {
                if grid[j][i].h > max_depth {
                    max_depth = grid[j][i].h;
                }
                let speed =
                    (grid[j][i].velocity_u().powi(2) + grid[j][i].velocity_v().powi(2)).sqrt();
                if speed > max_velocity {
                    max_velocity = speed;
                }
            }
        }

        // Output snapshot
        if time >= next_output {
            let mut wet_cells = 0u64;
            let mut vol = 0.0f64;
            let mut cur_max_h = 0.0f64;
            for j in 0..ny {
                for i in 0..nx {
                    if grid[j][i].is_wet() {
                        wet_cells += 1;
                        vol += grid[j][i].h * dx * dx;
                        if grid[j][i].h > cur_max_h {
                            cur_max_h = grid[j][i].h;
                        }
                    }
                }
            }
            total_volume = vol;
            snapshots.push(format!(
                "  t={:.0}s: wet_cells={}, max_h={:.3}m, volume={:.1}m³",
                time, wet_cells, cur_max_h, vol
            ));
            next_output += params.output_interval_s;
        }

        // Safety: max 10M steps
        if step > 10_000_000 {
            break;
        }
    }

    // Final summary
    let mut wet_area = 0.0f64;
    for j in 0..ny {
        for i in 0..nx {
            if grid[j][i].is_wet() {
                wet_area += dx * dx;
            }
        }
    }

    let mut result = format!(
        "SUCCESS: Simulasi 2D SWE Selesai\n\
         Grid: {}x{} ({:.0}m × {:.0}m)\n\
         Resolusi sel: {:.1}m\n\
         Durasi: {:.0}s ({:.1} jam)\n\
         Timesteps: {}\n\
         Manning's n: {:.4}\n\
         \n\
         === HASIL ===\n\
         Kedalaman maks: {:.3} m\n\
         Kecepatan maks: {:.3} m/s\n\
         Area tergenang: {:.1} m² ({:.4} Ha)\n\
         Volume air: {:.1} m³\n\
         \n\
         Inflow: Q={:.1} m³/s, t={:.0}-{:.0}s\n\
         \n\
         === PROGRES ===\n",
        nx,
        ny,
        nx as f64 * dx,
        ny as f64 * dx,
        dx,
        params.duration_s,
        params.duration_s / 3600.0,
        step,
        manning,
        max_depth,
        max_velocity,
        wet_area,
        wet_area / 10000.0,
        total_volume,
        inflow.discharge_m3s,
        inflow.start_s,
        inflow.end_s,
    );

    for snap in &snapshots {
        result.push_str(snap);
        result.push('\n');
    }

    result.push_str(&format!(
        "\nMetode: Finite Volume, HLL Riemann solver, CFL={}\n\
         Friction: Manning (implicit treatment)\n\
         Ref: Toro 2001, LeVeque 2002\n\
         CATATAN: Ini simplified 2D SWE. Untuk regulasi, gunakan HEC-RAS 2D.",
        CFL
    ));

    result
}

/// Simple test: flat DEM with point inflow
pub fn test_swe_flat() -> String {
    let nx = 50;
    let ny = 50;
    let dem: Vec<Vec<f64>> = vec![vec![10.0; nx]; ny]; // flat at 10m elevation

    let params = SweParams {
        nx,
        ny,
        dx: 30.0,
        manning_n: 0.035,
        duration_s: 600.0,       // 10 minutes
        output_interval_s: 60.0, // every minute
    };

    let inflow = InflowBC {
        cell_x: 25,
        cell_y: 25,
        width_cells: 3,
        discharge_m3s: 50.0,
        start_s: 0.0,
        end_s: 300.0,
    };

    simulate_flood(&dem, &params, &inflow)
}

/// Simple test: tilted DEM (valley)
pub fn test_swe_valley() -> String {
    let nx = 80;
    let ny = 40;
    let mut dem = vec![vec![0.0f64; nx]; ny];

    // Create V-shaped valley
    for j in 0..ny {
        for i in 0..nx {
            let cross_valley = ((j as f64 - ny as f64 / 2.0).abs() * 0.5).max(0.0);
            let along_valley = (nx as f64 - i as f64) * 0.002; // gentle downstream slope
            dem[j][i] = cross_valley + along_valley;
        }
    }

    let params = SweParams {
        nx,
        ny,
        dx: 30.0,
        manning_n: 0.04,
        duration_s: 1800.0,       // 30 minutes
        output_interval_s: 180.0, // every 3 minutes
    };

    let inflow = InflowBC {
        cell_x: 5,
        cell_y: 18, // upstream center of valley
        width_cells: 5,
        discharge_m3s: 100.0,
        start_s: 0.0,
        end_s: 900.0, // 15 min flood pulse
    };

    simulate_flood(&dem, &params, &inflow)
}
