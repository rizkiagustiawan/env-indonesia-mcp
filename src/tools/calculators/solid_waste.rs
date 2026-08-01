/// Solid Waste Calculator
/// Ref: UU 18/2008, SNI 19-2454:2002, Jakstranas 2025

pub fn calculate(population: u64, generation_rate_kg: f64) -> String {
    let daily_ton = (population as f64) * generation_rate_kg / 1000.0;
    let annual_ton = daily_ton * 365.0;

    // Komposisi sampah tipikal Indonesia (KLH data)
    let organik_pct = 60.0;
    let plastik_pct = 15.0;
    let kertas_pct = 10.0;
    let logam_pct = 3.0;
    let kaca_pct = 2.0;
    let lainnya_pct = 10.0;

    // Target Jakstranas 2025: pengurangan 30%, penanganan 70%
    let target_reduce = 0.30;
    let target_handle = 0.70;

    let mut out = String::from("=== Solid Waste Calculator ===\n");
    out.push_str("Ref: UU 18/2008, SNI 19-2454:2002, Jakstranas 2025\n\n");
    out.push_str(&format!(
        "Input:\n  Populasi = {} jiwa\n  Laju timbulan = {:.2} kg/orang/hari\n\n",
        population, generation_rate_kg
    ));
    out.push_str(&format!(
        "Timbulan:\n  Harian = {:.1} ton/hari\n  Tahunan = {:.0} ton/tahun\n\n",
        daily_ton, annual_ton
    ));
    out.push_str(&format!("Komposisi (tipikal Indonesia):\n  Organik: {:.0}% ({:.1} ton/hari)\n  Plastik: {:.0}% ({:.1} ton/hari)\n  Kertas: {:.0}% ({:.1} ton/hari)\n  Logam: {:.0}% ({:.1} ton/hari)\n  Kaca: {:.0}% ({:.1} ton/hari)\n  Lainnya: {:.0}% ({:.1} ton/hari)\n\n",
        organik_pct, daily_ton * organik_pct / 100.0,
        plastik_pct, daily_ton * plastik_pct / 100.0,
        kertas_pct, daily_ton * kertas_pct / 100.0,
        logam_pct, daily_ton * logam_pct / 100.0,
        kaca_pct, daily_ton * kaca_pct / 100.0,
        lainnya_pct, daily_ton * lainnya_pct / 100.0));
    out.push_str(&format!("Target Jakstranas 2025:\n  Pengurangan (30%): {:.1} ton/hari\n  Penanganan (70%): {:.1} ton/hari\n  Sisa ke TPA: {:.1} ton/hari\n\n", daily_ton * target_reduce, daily_ton * target_handle, daily_ton * (1.0 - target_reduce - target_handle).max(0.0)));

    // Emisi CH4 dari landfill (estimasi kasar)
    let ch4_ton_yr = annual_ton * 0.05 * 0.5 * 16.0 / 12.0; // 5% degradable, 50% CH4
    let co2e = ch4_ton_yr * 28.0;
    out.push_str(&format!("Estimasi emisi GRK dari TPA:\n  CH₄ ≈ {:.0} ton/tahun\n  CO₂e (GWP=28) ≈ {:.0} ton/tahun\n", ch4_ton_yr, co2e));
    out
}
