/// Composting Calculator (C/N Ratio Optimizer)
/// Ref: USDA, SNI

pub fn calculate(materials: &[(String, f64, f64, f64)]) -> String {
    // materials: Vec<(name, mass_kg, c_pct, n_pct)>
    if materials.is_empty() {
        return "ERROR: Masukkan minimal 1 material.".into();
    }

    let total_mass: f64 = materials.iter().map(|(_, m, _, _)| m).sum();
    let total_c: f64 = materials.iter().map(|(_, m, c, _)| m * c / 100.0).sum();
    let total_n: f64 = materials.iter().map(|(_, m, _, n)| m * n / 100.0).sum();
    let cn_ratio = if total_n > 0.0 {
        total_c / total_n
    } else {
        999.0
    };

    let mut out = String::from("=== Composting C/N Ratio ===\n\n");
    for (name, mass, c, n) in materials {
        out.push_str(&format!(
            "  {}: {:.0} kg (C={:.1}%, N={:.1}%)\n",
            name, mass, c, n
        ));
    }
    out.push_str(&format!(
        "\nTotal: {:.0} kg\nC total: {:.1} kg\nN total: {:.2} kg\nC/N ratio: {:.1}\n\n",
        total_mass, total_c, total_n, cn_ratio
    ));

    let status = if cn_ratio < 20.0 {
        "Terlalu rendah — tambah bahan kaya karbon (serbuk gergaji, jerami)"
    } else if cn_ratio <= 35.0 {
        "OPTIMAL (25-35:1) — proses komposting efisien"
    } else {
        "Terlalu tinggi — tambah bahan kaya nitrogen (kotoran ayam, daun hijau)"
    };
    out.push_str(&format!("Status: {}\nOptimal: 25-35:1\n", status));
    out
}
