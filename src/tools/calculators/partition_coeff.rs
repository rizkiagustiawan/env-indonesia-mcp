/// Partition Coefficient & Retardation Factor
/// Kd = Koc × foc, R = 1 + (ρb/n) × Kd
/// Ref: Schwarzenbach et al. (2003), Environmental Organic Chemistry

pub fn calculate(compound: &str, foc: f64, bulk_density_kgm3: f64, porosity: f64) -> String {
    let mut out = String::from("=== Koefisien Partisi & Faktor Retardasi ===\n");
    out.push_str("Ref: Schwarzenbach et al. (2003), Environmental Organic Chemistry\n\n");

    if foc <= 0.0 || foc > 1.0 { return "ERROR: foc (fraksi karbon organik) harus antara 0-1.".into(); }
    if bulk_density_kgm3 <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if porosity <= 0.0 || porosity >= 1.0 { return "ERROR: Porositas harus antara 0 dan 1.".into(); }

    let compound_lower = compound.to_lowercase();

    // Koc values (L/kg) for common contaminants
    let (koc, name) = match compound_lower.as_str() {
        "benzene" | "benzena" => (83.0, "Benzena"),
        "toluene" | "toluena" => (300.0, "Toluena"),
        "naphthalene" | "naftalena" => (1300.0, "Naftalena"),
        "phenol" | "fenol" => (27.0, "Fenol"),
        "atrazine" | "atrazin" => (100.0, "Atrazin"),
        "ddt" => (240000.0, "DDT"),
        "pcb" => (100000.0, "PCB"),
        "xylene" | "xilena" => (240.0, "Xilena"),
        "ethylbenzene" | "etilbenzena" => (204.0, "Etilbenzena"),
        _ => return format!("ERROR: Senyawa '{}' tidak tersedia.\nPilihan: benzene, toluene, naphthalene, phenol, atrazine, ddt, pcb, xylene, ethylbenzene.", compound),
    };

    // Kd = Koc × foc
    let kd = koc * foc; // L/kg

    // Convert bulk density to kg/L for consistent units
    let rho_b_kgl = bulk_density_kgm3 / 1000.0; // kg/m³ → kg/L

    // Retardation factor R = 1 + (ρb/n) × Kd
    let r_factor = 1.0 + (rho_b_kgl / porosity) * kd;

    // Effective velocity ratio
    let v_ratio = 1.0 / r_factor;

    // Mobility assessment
    let mobility = if r_factor < 2.0 {
        "Tinggi (high) — kontaminan bergerak hampir secepat air tanah"
    } else if r_factor < 10.0 {
        "Sedang (medium) — kontaminan tertahan moderat"
    } else if r_factor < 100.0 {
        "Rendah (low) — kontaminan sangat tertahan"
    } else {
        "Immobile — kontaminan praktis tidak bergerak"
    };

    out.push_str(&format!("Input:\n  Senyawa = {}\n  Koc = {:.0} L/kg\n  foc = {:.4}\n  Bulk density (ρb) = {:.0} kg/m³\n  Porositas (n) = {:.2}\n\n",
        name, koc, foc, bulk_density_kgm3, porosity));

    out.push_str("Perhitungan:\n");
    out.push_str(&format!("  Kd = Koc × foc = {:.0} × {:.4} = {:.2} L/kg\n", koc, foc, kd));
    out.push_str(&format!("  R = 1 + (ρb/n) × Kd = 1 + ({:.3}/{:.2}) × {:.2} = {:.2}\n\n",
        rho_b_kgl, porosity, kd, r_factor));

    out.push_str("Hasil:\n");
    out.push_str(&format!("  Koefisien distribusi (Kd) = {:.2} L/kg\n", kd));
    out.push_str(&format!("  Faktor retardasi (R) = {:.2}\n", r_factor));
    out.push_str(&format!("  Rasio kecepatan efektif (v/v_air) = {:.4} ({:.2}%)\n", v_ratio, v_ratio * 100.0));
    out.push_str(&format!("  Mobilitas: {}\n\n", mobility));

    // Reference table
    out.push_str("Tabel Koc senyawa tersedia:\n");
    out.push_str("  Fenol         :       27 L/kg (mobile)\n");
    out.push_str("  Benzena       :       83 L/kg\n");
    out.push_str("  Atrazin       :      100 L/kg\n");
    out.push_str("  Etilbenzena   :      204 L/kg\n");
    out.push_str("  Xilena        :      240 L/kg\n");
    out.push_str("  Toluena       :      300 L/kg\n");
    out.push_str("  Naftalena     :    1,300 L/kg\n");
    out.push_str("  PCB           :  100,000 L/kg (immobile)\n");
    out.push_str("  DDT           :  240,000 L/kg (immobile)\n");

    out
}
