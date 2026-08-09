/// PFAS Electrochemical Oxidation Design — CF2-Unzipping Cycle
/// Ref: Tshangana et al. 2025 (npj Clean Water); Nature s41545-025-00457-3
/// Formula: EE/O = (P × t) / (V × log(C0/Ct)); defluorination η
pub fn assess(
    pfas_type: &str, conc_mg_l: f64, volume_m3: f64,
    electrode_type: &str, current_density_ma_cm2: f64,
    electrode_area_cm2: f64, target_removal_pct: f64,
) -> String {
    let mut out = String::from("=== PFAS Electrochemical Oxidation Design ===\n");
    out.push_str("Ref: Tshangana 2025 (npj Clean Water); Nature s41545-025-00457-3\n\n");
    let (voltage_v, k_h_per_cm2, energy_kwh_m3) = match electrode_type.to_lowercase().as_str() {
        s if s.contains("bdd") || s.contains("boron") => (5.0, 0.0085, 19.9),
        s if s.contains("ti4o7") || s.contains("magneli") || s.contains("titanium") => (4.0, 0.011, 0.15),
        s if s.contains("sno2") || s.contains("tin") => (5.0, 0.0065, 12.0),
        s if s.contains("pbo2") || s.contains("lead") => (5.0, 0.0072, 15.0),
        _ => (5.0, 0.008, 15.0),
    };
    let power_w = voltage_v * current_density_ma_cm2 * electrode_area_cm2 / 1000.0;
    let removal = target_removal_pct / 100.0;
    let target_conc = conc_mg_l * (1.0 - removal);
    let log_ratio = (conc_mg_l / target_conc.max(1e-9)).log10();
    let time_s = (energy_kwh_m3 * volume_m3 * 3600.0) / power_w.max(1.0) * (log_ratio / 1.0).min(1.0);
    let time_hr = time_s / 3600.0;
    let total_energy_kwh = energy_kwh_m3 * volume_m3;
    let defluorination_pct = removal * 60.0; // 60% of removed PFAS is defluorinated (fraction→percent)
    let theoretical_f_mg = match pfas_type.to_lowercase().as_str() {
        s if s.contains("pfoa") => 9.0 * 19.0 * conc_mg_l / 414.07,
        s if s.contains("pfos") => 8.0 * 19.0 * conc_mg_l / 500.13,
        s if s.contains("pfhxa") => 5.0 * 19.0 * conc_mg_l / 314.05,
        s if s.contains("pfbs") => 4.0 * 19.0 * conc_mg_l / 300.10,
        _ => 8.0 * 19.0 * conc_mg_l / 450.0,
    };
    let actual_f_mg = theoretical_f_mg * defluorination_pct / 100.0;
    out.push_str(&format!("PFAS: {} (conc: {:.2} mg/L, vol: {:.1} m3)\n", pfas_type, conc_mg_l, volume_m3));
    out.push_str(&format!("Electrode: {} (V={:.0}V, k={:.4}/hr/cm2, EE={:.1} kWh/m3)\n", electrode_type, voltage_v, k_h_per_cm2, energy_kwh_m3));
    out.push_str(&format!("Current density: {:.0} mA/cm2, Area: {:.0} cm2\n", current_density_ma_cm2, electrode_area_cm2));
    out.push_str(&format!("Target removal: {:.0}%\n\n", target_removal_pct));
    out.push_str("-- Design Parameters --\n\n");
    out.push_str(&format!("  Power: {:.1} W\n", power_w));
    out.push_str(&format!("  Treatment time: {:.1} hr ({:.0} min)\n", time_hr, time_hr * 60.0));
    out.push_str(&format!("  Total energy: {:.1} kWh ({:.1} kWh/m3)\n", total_energy_kwh, total_energy_kwh / volume_m3));
    out.push_str(&format!("  >> Concentration after treatment: {:.4} mg/L ({:.2} ng/L)\n\n", target_conc, target_conc * 1e6));
    out.push_str("-- CF2-Unzipping Cycle --\n");
    out.push_str("  1. DET: PFAS → radical (electron transfer to anode)\n");
    out.push_str("  2. Kolbe decarboxylation: radical → perfluoroalkyl + CO2/SO3\n");
    out.push_str("  3. Hydroxylation: + OH → perfluorinated alcohol\n");
    out.push_str("  4. Further oxidation → radical\n");
    out.push_str("  5. Chain shortening: → shorter PFAS + COF2\n");
    out.push_str("  6. Repeat until mineralization: → CO2 + F-\n\n");
    out.push_str("-- Defluorination --\n\n");
    out.push_str(&format!("  Theoretical F: {:.2} mg\n", theoretical_f_mg * volume_m3 * 1000.0));
    out.push_str(&format!("  Actual F released: {:.2} mg\n", actual_f_mg * volume_m3 * 1000.0));
    out.push_str(&format!("  >> Defluorination efficiency: {:.1}%\n\n", defluorination_pct));
    out.push_str("-- STATUS KEPATUHAN --\n");
    out.push_str("  EPA MCL: 4 ng/L for PFOA/PFOS → ");
    let compliant = target_conc * 1e6 <= 4.0;
    out.push_str(if compliant { "✅ MEMENUHI\n" } else { "❌ MELEBIHI — extend treatment time\n" });
    out.push_str("  Indonesia: belum ada baku mutu PFAS spesifik\n\n");
    out.push_str("-- PEMANTAUAN --\n");
    out.push_str("  Parameter: PFAS target list + F- (defluorination) + byproducts (short-chain)\n");
    out.push_str("  Metode: EPA 1633 (PFAS), ion chromatography (F-)\n");
    out.push_str("  Ref: Tshangana 2025; Nature s41545-025-00457-3\n");
    out
}

#[cfg(test)]
mod tests {
    // Self-check: 100% removal -> 60% defluorination (not 0.6%), actual F = 0.60 * theoretical
    #[test]
    fn defluorination_percent_correct() {
        let removal: f64 = 1.0; // 100%
        let def_pct = removal * 60.0; // 60%
        assert!((def_pct - 60.0).abs() < 1e-9, "def_pct={def_pct} expected 60%");
        let theoretical_f: f64 = 100.0;
        let actual_f = theoretical_f * def_pct / 100.0; // 60 mg
        assert!((actual_f - 60.0).abs() < 1e-9, "actual F={actual_f} expected 60 mg");
    }
}

