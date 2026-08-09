/// POME (Palm Oil Mill Effluent) Calculator
/// Ref: KLHK P.05/2014; Rana 2017; Setiawan 2025; Sabiani 2023
pub fn assess(ton_ffb_day: f64, has_pond_system: bool, target_bod_mg_l: f64) -> String {
    let mut out = String::from("=== POME (Palm Oil Mill Effluent) ===\n");
    out.push_str("Ref: KLHK P.05/2014; Rana 2017; Setiawan 2025\n\n");
    // POME generation: 0.5-1.0 m3/ton FFB (Fresh Fruit Bunches)
    let pome_volume = ton_ffb_day * 0.7; // m3/day (average 0.7)
    // Typical POME characteristics (untreated)
    let bod = 25000.0 + (ton_ffb_day * 10.0).min(25000.0); // 25,000-50,000 mg/L
    let cod = bod * 2.0; // COD/BOD ~2
    let tss = 18000.0;
    let fog = 4000.0; // fat oil grease
    let tn = 500.0;
    let ph = 4.5;

    out.push_str(&format!("FFB: {:.0} ton/day -> POME: {:.0} m3/day\n", ton_ffb_day, pome_volume));
    out.push_str("-- Untreated POME Characteristics --\n\n");
    out.push_str(&format!("  BOD: {:.0} mg/L (typical 25,000-50,000)\n", bod));
    out.push_str(&format!("  COD: {:.0} mg/L (BOD*2)\n", cod));
    out.push_str(&format!("  TSS: {:.0} mg/L, O&G: {:.0} mg/L, TN: {:.0} mg/L\n", tss, fog, tn));
    out.push_str(&format!("  pH: {:.1} (acidic)\n\n", ph));

    if has_pond_system {
        out.push_str("-- Pond System Design (KLHK P.05/2014) --\n\n");
        let cooling_hrt = 1.0; let anaerobic_hrt = 45.0; let aerobic_hrt = 20.0; let maturation_hrt = 15.0;
        let total_hrt = cooling_hrt + anaerobic_hrt + aerobic_hrt + maturation_hrt;
        let v_cooling = pome_volume * cooling_hrt;
        let v_anaerobic = pome_volume * anaerobic_hrt;
        let v_aerobic = pome_volume * aerobic_hrt;
        let v_maturation = pome_volume * maturation_hrt;
        out.push_str(&format!("  Cooling pond: HRT {} day, V={:.0} m3\n", cooling_hrt, v_cooling));
        out.push_str(&format!("  Anaerobic pond: HRT {} day, V={:.0} m3 (BOD removal ~90%)\n", anaerobic_hrt, v_anaerobic));
        out.push_str(&format!("  Aerobic pond: HRT {} day, V={:.0} m3\n", aerobic_hrt, v_aerobic));
        out.push_str(&format!("  Maturation pond: HRT {} day, V={:.0} m3\n", maturation_hrt, v_maturation));
        out.push_str(&format!("  Total HRT: {} days, Total V: {:.0} m3\n\n", total_hrt, v_cooling+v_anaerobic+v_aerobic+v_maturation));
        let effluent_bod = bod * 0.05; // 95% removal
        out.push_str(&format!("  >> Effluent BOD: {:.0} mg/L (95% removal)\n", effluent_bod));
        if effluent_bod <= target_bod_mg_l { out.push_str("  [OK] Meets target\n"); }
        else { out.push_str(&format!("  [WARN] Exceeds target {:.0}. Add polishing pond or biogas.\n", target_bod_mg_l)); }
    }
    // Biogas potential
    let biogas_m3_day = cod * pome_volume / 1000.0 * 0.35; // 0.35 m3 CH4/kg COD
    out.push_str(&format!("\n  Biogas potential: {:.0} m3 CH4/day ({:.1} MMBtu)\n", biogas_m3_day, biogas_m3_day*0.0372));
    out.push_str("\n  Ref: KLHK P.05/2014; Rana 2017; Setiawan 2025\n");
    out
}
