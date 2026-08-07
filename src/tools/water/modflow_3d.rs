/// MODFLOW 6 3D Groundwater Model — Python/FloPy Bridge
/// 2026 SOTA: Dharma et al. 2026 (Seulawah Agam geothermal, Aceh — MODFLOW 6 + FloPy)
/// Ref: USGS MODFLOW 6 (Langevin et al. 2017); Bakker et al. 2016
/// Python package: flopy (3D grid, steady/transient, head + drawdown)

use std::process::Command;

pub fn assess(
    grid_nlay: u32, grid_nrow: u32, grid_ncol: u32,
    cell_size_m: f64,
    hk_m_s: f64,        // horizontal hydraulic conductivity
    vk_m_s: f64,        // vertical hydraulic conductivity
    sy: f64,            // specific yield
    ss_per_m: f64,      // specific storage (1/m)
    pumping_m3_day: f64,
    pumping_x: u32, pumping_y: u32, pumping_layer: u32,
    recharge_mm_yr: f64,
    chb_head_m: f64,
    sim_type: &str,     // "steady" or "transient"
    duration_days: u32,
) -> String {
    let mut out = String::from("=== MODFLOW 6 3D Groundwater Model (FloPy) ===\n");
    out.push_str("Ref: USGS MODFLOW 6 (Langevin et al. 2017); FloPy v3\n");
    out.push_str("2026 SOTA: Dharma et al. 2026 (Seulawah Agam, Aceh)\n\n");

    let script = "
import sys, json
try:
    import numpy as np
    import flopy as fp
except ImportError:
    print(json.dumps({'error': 'flopy/numpy not installed. Run: pip install flopy numpy'}))
    sys.exit(1)

args = sys.argv[1:]
nlay, nrow, ncol = int(args[0]), int(args[1]), int(args[2])
delr = float(args[3])
hk = float(args[4])
vk = float(args[5])
sy_val = float(args[6])
ss = float(args[7])
pumping = float(args[8])
px, py, pl = int(args[9]), int(args[10]), int(args[11])
recharge = float(args[12]) / 1000.0 / 365.0  # mm/yr to m/s
chb_head = float(args[13])
sim_type = args[14]
duration = int(args[15])

ws = '/tmp/modflow_run'
name = 'env_mcp_model'
try:
    sim = fp.mf6.MFSimulation.load(name=name, sim_ws=ws, exe_name='mf6')
    if sim is None:
        raise Exception()
except:
    sim = fp.mf6.MFSimulation(sim_name=name, sim_ws=ws, exe_name='mf6')
    fp.mf6.ModflowTdis(sim, nper=1, perioddata=[(duration if sim_type=='transient' else 1, 1, 1.0)])
    fp.mf6.ModflowIms(sim)
    gwf = fp.mf6.ModflowGwf(sim, modelname=name, model_npgs=10)
    fp.mf6.ModflowGwfdis(gwf, nlay=nlay, nrow=nrow, ncol=ncol, delr=delr, delc=delr, top=chb_head, botm=[chb_head - 10*(i+1) for i in range(nlay)])
    fp.mf6.ModflowGwfnpf(gwf, k=hk, k33=vk, save_flows=True)
    fp.mf6.ModflowGwfic(gwf, strt=chb_head)
    fp.mf6.ModflowGwfsto(gwf, sy=sy_val, ss=ss, iconvert=1, steady_state={0: True} if sim_type=='steady' else {0: False})

    # CHB
    chb_spd = {}
    for i in range(nrow):
        chb_spd[(0, i, 0)] = chb_head
        chb_spd[(0, i, ncol-1)] = chb_head
    fp.mf6.ModflowGwfchd(gwf, stress_period_data=chb_spd)

    # Recharge
    fp.mf6.ModflowGwfrch(gwf, recharge=recharge)

    # Wells
    well_spd = {0: [(pl-1, py-1, px-1, -pumping)]}
    fp.mf6.ModflowGwfwel(gwf, stress_period_data=well_spd)

    # OC
    fp.mf6.ModflowGwfoc(gwf, head_filerecord=f'{name}.hds', saverecord=[('HEAD', 'ALL')], printrecord=[('HEAD', 'ALL')])

sim.write_simulation()
success, buff = sim.run_simulation()

if not success:
    print(json.dumps({'error': 'MODFLOW failed to converge', 'log': str(buff)[:500]}))
    sys.exit(1)

import flopy.utils.binaryfile as bf
try:
    head = bf.HeadFile(f'{ws}/{name}.hds').get_data()
    head_arr = np.array(head)
    result = {
        'max_head': float(np.max(head_arr)),
        'min_head': float(np.min(head_arr)),
        'mean_head': float(np.mean(head_arr)),
        'drawdown_at_well': float(chb_head - head_arr[pl-1, py-1, px-1]),
        'head_profile': head_arr[:, nrow//2, :].flatten().tolist()[:20],
        'grid_shape': list(head_arr.shape),
    }
    print(json.dumps(result))
except Exception as e:
    print(json.dumps({'error': f'Reading heads failed: {e}'}))
";

    let result = Command::new("python3")
        .arg("-c")
        .arg(script)
        .args(&[
            grid_nlay.to_string(), grid_nrow.to_string(), grid_ncol.to_string(),
            cell_size_m.to_string(), hk_m_s.to_string(), vk_m_s.to_string(),
            sy.to_string(), ss_per_m.to_string(), pumping_m3_day.to_string(),
            pumping_x.to_string(), pumping_y.to_string(), pumping_layer.to_string(),
            recharge_mm_yr.to_string(), chb_head_m.to_string(),
            sim_type.to_string(), duration_days.to_string(),
        ])
        .output();

    out.push_str(&format!("Grid: {}×{}×{}, cell={:.0}m\n", grid_nlay, grid_nrow, grid_ncol, cell_size_m));
    out.push_str(&format!("Aquifer: K_h={:.2e} m/s, K_v={:.2e} m/s\n", hk_m_s, vk_m_s));
    out.push_str(&format!("Sy={:.3}, Ss={:.2e}/m\n", sy, ss_per_m));
    out.push_str(&format!("Pumping: {:.1} m³/day at ({},{},{})\n", pumping_m3_day, pumping_x, pumping_y, pumping_layer));
    out.push_str(&format!("Recharge: {:.1} mm/yr\n", recharge_mm_yr));
    out.push_str(&format!("CHB head: {:.1} m\n", chb_head_m));
    out.push_str(&format!("Sim: {} ({} days)\n\n", sim_type, duration_days));

    match result {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);

            if !o.status.success() || stdout.trim().is_empty() {
                if stdout.contains("not installed") || stderr.contains("No module") {
                    out.push_str("⚠️ flopy/numpy tidak terinstall.\n");
                    out.push_str("Install: pip install flopy numpy\n\n");
                    out.push_str("── Fallback: Analytical Theis Solution ──\n\n");
                    return fallback_theis(&mut out, pumping_m3_day, hk_m_s, sy, duration_days, cell_size_m);
                }
                out.push_str(&format!("ERROR [E502]: MODFLOW failed.\nStderr: {}\n", &stderr[..stderr.len().min(500)]));
                return fallback_theis(&mut out, pumping_m3_day, hk_m_s, sy, duration_days, cell_size_m);
            }

            // Parse JSON output
            let json_str = stdout.trim();
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(err) = v.get("error") {
                    out.push_str(&format!("⚠️ MODFLOW Error: {}\n", err));
                    if err.as_str().unwrap_or("").contains("not installed") {
                        out.push_str("Install: pip install flopy numpy\n\n");
                    }
                    return fallback_theis(&mut out, pumping_m3_day, hk_m_s, sy, duration_days, cell_size_m);
                }

                let max_h = v.get("max_head").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let min_h = v.get("min_head").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let mean_h = v.get("mean_head").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let drawdown = v.get("drawdown_at_well").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let shape = v.get("grid_shape").and_then(|x| x.as_array());

                out.push_str("═══ MODFLOW 6 RESULTS ═══\n\n");
                out.push_str(&format!("  Max head:    {:.2} m\n", max_h));
                out.push_str(&format!("  Min head:    {:.2} m\n", min_h));
                out.push_str(&format!("  Mean head:   {:.2} m\n", mean_h));
                out.push_str(&format!("  Drawdown at well: {:.2} m\n", drawdown));
                if let Some(s) = shape {
                    out.push_str(&format!("  Grid shape:  {:?}\n", s));
                }

                // Head profile
                if let Some(profile) = v.get("head_profile").and_then(|x| x.as_array()) {
                    out.push_str("\n  Head profile (center row, layer 1):\n  ");
                    for (i, h) in profile.iter().enumerate() {
                        if let Some(hv) = h.as_f64() {
                            out.push_str(&format!("{:.1} ", hv));
                            if (i + 1) % 5 == 0 { out.push_str("\n  "); }
                        }
                    }
                    out.push('\n');
                }

                out.push_str("\n── Interpretation ──\n");
                if drawdown > 10.0 {
                    out.push_str("  🔴 Drawdown >10m — Overextraction. Reduce pumping or add recharge.\n");
                } else if drawdown > 3.0 {
                    out.push_str("  🟠 Drawdown 3-10m — Monitor aquifer. Sustainable limit approaching.\n");
                } else {
                    out.push_str("  🟢 Drawdown <3m — Sustainable extraction.\n");
                }

                out.push_str("\n  Ref: Dharma et al. 2026 (Seulawah Agam geothermal, Aceh)\n");
            } else {
                out.push_str(&format!("Raw output: {}", &stdout[..stdout.len().min(500)]));
                return fallback_theis(&mut out, pumping_m3_day, hk_m_s, sy, duration_days, cell_size_m);
            }
        }
        Err(e) => {
            out.push_str(&format!("ERROR: Python execution failed: {}\n", e));
            return fallback_theis(&mut out, pumping_m3_day, hk_m_s, sy, duration_days, cell_size_m);
        }
    }

    out
}

fn fallback_theis(out: &mut String, pumping: f64, hk: f64, sy: f64, duration: u32, cell_size: f64) -> String {
    // Theis (1935) analytical solution for drawdown
    // s = (Q / (4πT)) × W(u), where u = (r²S) / (4Tt)
    let T = hk * 10.0 * cell_size; // transmissivity = K × aquifer thickness (approx 10m × cell)
    let S = sy * cell_size * cell_size; // storativity = Sy × area
    let t = duration as f64 * 86400.0; // seconds
    let r = cell_size; // distance = 1 cell

    let u = (r * r * S) / (4.0 * T * t).max(1e-10);
    // Well function W(u) ≈ -0.5772 - ln(u) for small u (Cooper-Jacob)
    let w_u = -0.5772 - u.ln();
    let s = (pumping / 86400.0 / (4.0 * std::f64::consts::PI * T).max(1e-10)) * w_u;

    out.push_str(&format!("  Pumping: {:.1} m³/day\n", pumping));
    out.push_str(&format!("  Transmissivity T: {:.2e} m²/s\n", T));
    out.push_str(&format!("  Storativity S: {:.2e}\n", S));
    out.push_str(&format!("  Duration: {} days\n", duration));
    out.push_str(&format!("\n  ► Drawdown (Theis): {:.2} m\n", s));

    if s > 10.0 {
        out.push_str("  🔴 Overextraction (>10m drawdown)\n");
    } else if s > 3.0 {
        out.push_str("  🟠 Monitor (>3m drawdown)\n");
    } else {
        out.push_str("  🟢 Sustainable (<3m drawdown)\n");
    }

    out.push_str("\n  ⚠️ Analytical fallback (Theis 1935). Install flopy for full 3D model.\n");
    out.push_str("  Ref: Theis 1935; Dharma et al. 2026 (MODFLOW 6 + FloPy, Indonesia)\n");
    out.clone()
}
