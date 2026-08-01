/// Simplified LCA (Life Cycle Assessment)
/// Ref: ISO 14040/14044, IPCC AR6 GWP100

pub fn calculate(material: &str, mass_kg: f64) -> String {
    if mass_kg <= 0.0 {
        return "ERROR [E102]: Parameter harus > 0.".into();
    }

    let (ef_co2, ef_water, ef_energy, desc) = match material.to_lowercase().as_str() {
        "baja" | "steel" => (1.85, 20.0, 20.0, "Baja (BOF steelmaking)"),
        "semen" | "cement" => (0.622, 1.5, 3.4, "Semen Portland (proses klinker)"),
        "plastik" | "plastic" | "pe" => (2.5, 10.0, 40.0, "Plastik PE (polietilen)"),
        "aluminium" => (8.0, 100.0, 170.0, "Aluminium (smelting)"),
        "kayu" | "wood" | "timber" => (-1.5, 5.0, 3.0, "Kayu (carbon sink, hutan lestari)"),
        "kertas" | "paper" => (1.1, 30.0, 17.0, "Kertas (virgin pulp)"),
        "beton" | "concrete" => (0.13, 1.0, 0.7, "Beton siap pakai"),
        "kaca" | "glass" => (0.86, 7.0, 15.0, "Kaca float"),
        "bata" | "brick" => (0.24, 1.2, 2.0, "Bata tanah liat"),
        _ => return format!("Material '{}' tidak ditemukan. Tersedia: baja, semen, plastik, aluminium, kayu, kertas, beton, kaca, bata.", material),
    };

    let co2_total = ef_co2 * mass_kg;
    let water_total = ef_water * mass_kg;
    let energy_total = ef_energy * mass_kg;

    let mut out = format!(
        "=== Simplified LCA ===\nRef: ISO 14040/14044, IPCC AR6\n\nMaterial: {} ({:.0} kg)\n\n",
        desc, mass_kg
    );
    out.push_str(&format!("Impact Categories (cradle-to-gate):\n  GWP: {:.2} kgCO₂e (EF={:.3} kgCO₂/kg)\n  Water: {:.0} L (EF={:.1} L/kg)\n  Energy: {:.0} MJ (EF={:.1} MJ/kg)\n", co2_total, ef_co2, water_total, ef_water, energy_total, ef_energy));
    out
}
