/// SBR (Sequencing Batch Reactor) Design
/// Ref: Metcalf & Eddy 2004; Grady et al. 2011
pub fn assess(
    flow_m3_day: f64,
    influent_bod_mg_l: f64,
    target_bod_mg_l: f64,
    n_cycles_day: u32,
    mlss_mg_l: f64,
    fill_fraction: f64,
    react_time_hr: f64,
    settle_time_hr: f64,
    draw_time_hr: f64,
) -> String {
    let mut out = String::from("=== SBR (Sequencing Batch Reactor) Design ===\n");
    out.push_str("Ref: Metcalf & Eddy 2004; Grady et al. 2011\n\n");

    let n = n_cycles_day;
    let q_per_cycle = flow_m3_day / n as f64;
    let total_cycle_time = 24.0 / n as f64;
    let idle_time = total_cycle_time - fill_fraction * total_cycle_time - react_time_hr - settle_time_hr - draw_time_hr;

    // Reactor volume = fill volume + settle volume
    let v_fill = q_per_cycle;
    let v_settle = q_per_cycle * 1.3; // 30% excess for settling
    let v_reactor = v_fill + v_settle;

    let fm = q_per_cycle * influent_bod_mg_l / (v_reactor * mlss_mg_l);

    out.push_str(&format!("Flow: {:.1} m3/day, Cycles: {}/day\n", flow_m3_day, n));
    out.push_str(&format!("Volume/cycle: {:.1} m3\n", q_per_cycle));
    out.push_str(&format!("Cycle time: {:.1} hr (fill+react+settle+draw+idle)\n\n", total_cycle_time));

    out.push_str("-- Cycle Phases --\n\n");
    out.push_str(&format!("  Fill: {:.1} hr ({:.0}%)\n", fill_fraction * total_cycle_time, fill_fraction * 100.0));
    out.push_str(&format!("  React: {:.1} hr\n", react_time_hr));
    out.push_str(&format!("  Settle: {:.1} hr\n", settle_time_hr));
    out.push_str(&format!("  Draw: {:.1} hr\n", draw_time_hr));
    out.push_str(&format!("  Idle: {:.1} hr\n\n", idle_time.max(0.0)));

    out.push_str("-- Design --\n\n");
    out.push_str(&format!("  Reactor volume: {:.1} m3 (per basin)\n", v_reactor));
    out.push_str(&format!("  F/M: {:.3} kg BOD/kg MLSS/day\n", fm));
    out.push_str(&format!("  BOD removal: {:.0} -> {:.0} mg/L ({:.0}%)\n\n", influent_bod_mg_l, target_bod_mg_l, (1.0-target_bod_mg_l/influent_bod_mg_l)*100.0));

    let n_basins = 2; // minimum
    out.push_str(&format!("  >> Basins: {} (parallel operation)\n", n_basins));
    out.push_str(&format!("  >> Total volume: {:.1} m3\n", v_reactor * n_basins as f64));

    if fm > 0.4 { out.push_str("  [WARN] F/M > 0.4 — high loading\n"); }
    else if fm < 0.05 { out.push_str("  [WARN] F/M < 0.05 — low loading\n"); }
    else { out.push_str("  [OK] F/M in range\n"); }

    // ─── EFFLUENT COMPLIANCE ───
    out.push_str("\n─── STATUS KEPATUHAN EFLUEN (Permen LH 11/2025) ───\n\n");
    let bod_ok = target_bod_mg_l <= 30.0;
    out.push_str(&format!("  BOD: {:.0} mg/L → ≤30 mg/L (domestik): {}\n", target_bod_mg_l, if bod_ok {"✅"} else {"❌"}));
    let cod_eff = target_bod_mg_l * 2.0;
    out.push_str(&format!("  COD: {:.0} mg/L → ≤100 mg/L: {}\n\n", cod_eff, if cod_eff <= 100.0 {"✅"} else {"❌"}));

    out.push_str("─── PEMANTAUAN (RPL) ───\n");
    out.push_str("  Parameter: BOD, COD, TSS, NH3-N, pH, coliform\n");
    out.push_str("  Frekuensi: Bulanan (effluent)\n");

    out.push_str("\n─── PELAPORAN & IZIN ───\n");
    out.push_str("  Permen LH 11/2025: Baku mutu air limbah domestik\n");
    out.push_str("  PP 22/2021 Pasal 124-131; Amdalnet + OSS\n");
    out.push_str("  Permen LH 6/2026: Sanksi berbasis risiko (denda max Rp3M)\n");

    out.push_str("\n  Ref: Metcalf & Eddy 2004; Permen LH 11/2025; PP 22/2021\n");
    out
}
