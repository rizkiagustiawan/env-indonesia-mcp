/// Sedimentation Tank Design
/// Overflow rate v₀ = Q/A, Detention time t = V/Q
/// Ref: Metcalf & Eddy (2003), Wastewater Engineering

pub fn design(q_m3d: f64, tank_type: &str, tank_shape: &str) -> String {
    let mut out = String::from("=== Desain Bak Sedimentasi ===\n");
    out.push_str("Ref: Metcalf & Eddy (2003), Wastewater Engineering\n\n");

    if q_m3d <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    let type_lower = tank_type.to_lowercase();
    let shape_lower = tank_shape.to_lowercase();

    // Design criteria based on type
    let (overflow_rate, detention_hr, depth_m, weir_max) = match type_lower.as_str() {
        "primary" | "primer" => (40.0, 2.0, 3.5, 250.0),   // m³/m²/day, hours, m, m³/m/day
        "secondary" | "sekunder" => (25.0, 2.5, 4.0, 125.0),
        _ => return format!("ERROR: Tipe bak '{}' tidak dikenali. Pilihan: primary/primer, secondary/sekunder.", tank_type),
    };

    if !["rectangular" , "persegi", "circular", "bundar"].contains(&shape_lower.as_str()) {
        return format!("ERROR: Bentuk bak '{}' tidak dikenali. Pilihan: rectangular/persegi, circular/bundar.", tank_shape);
    }

    // Surface area
    let surface_area = q_m3d / overflow_rate;

    // Volume
    let volume = q_m3d * detention_hr / 24.0;

    // Actual detention time check
    let actual_detention = volume / q_m3d * 24.0;

    // Weir loading
    let weir_length_min = q_m3d / weir_max;

    out.push_str(&format!("Input:\n  Q = {:.0} m³/hari ({:.2} L/s)\n  Tipe = {} ({})\n  Bentuk = {}\n\n",
        q_m3d, q_m3d / 86.4, tank_type, type_lower, tank_shape));

    out.push_str(&format!("Kriteria desain ({}):\n  Overflow rate = {:.0} m³/m²/hari\n  Detention time = {:.1} jam\n  Kedalaman = {:.1} m\n  Weir loading maks = {:.0} m³/m/hari\n\n",
        type_lower, overflow_rate, detention_hr, depth_m, weir_max));

    out.push_str("Dimensi bak:\n");
    out.push_str(&format!("  Luas permukaan = {:.1} m²\n", surface_area));
    out.push_str(&format!("  Volume = {:.1} m³\n", volume));

    let pi = std::f64::consts::PI;

    match shape_lower.as_str() {
        "circular" | "bundar" => {
            let diameter = (4.0 * surface_area / pi).sqrt();
            out.push_str(&format!("  Diameter = {:.1} m\n", diameter));
            out.push_str(&format!("  Kedalaman = {:.1} m\n", depth_m));
            // Weir = circumference
            let weir_length = pi * diameter;
            let weir_loading = q_m3d / weir_length;
            out.push_str(&format!("  Panjang weir (keliling) = {:.1} m\n", weir_length));
            out.push_str(&format!("  Weir loading = {:.1} m³/m/hari {}\n",
                weir_loading, if weir_loading <= weir_max { "✅" } else { "❌ MELEBIHI" }));
        }
        "rectangular" | "persegi" => {
            // L:W ratio typically 3:1 to 5:1
            let width = (surface_area / 4.0).sqrt(); // L:W = 4:1
            let length = surface_area / width;
            out.push_str(&format!("  Panjang = {:.1} m\n", length));
            out.push_str(&format!("  Lebar = {:.1} m\n", width));
            out.push_str(&format!("  Kedalaman = {:.1} m\n", depth_m));
            out.push_str(&format!("  Rasio L:W = {:.1}:1\n", length / width));
            // Weir at end
            let weir_length = width;
            let weir_loading = q_m3d / weir_length;
            out.push_str(&format!("  Panjang weir = {:.1} m\n", weir_length));
            out.push_str(&format!("  Weir loading = {:.1} m³/m/hari {}\n",
                weir_loading, if weir_loading <= weir_max { "✅" } else { "❌ MELEBIHI — tambah weir" }));
            if weir_loading > weir_max {
                out.push_str(&format!("  Panjang weir minimum = {:.1} m\n", weir_length_min));
            }
        }
        _ => {}
    }

    out.push_str(&format!("\n  Detention time aktual = {:.1} jam\n", actual_detention));

    // Typical removal efficiencies
    out.push_str("\nEfisiensi removal tipikal:\n");
    match type_lower.as_str() {
        "primary" | "primer" => {
            out.push_str("  TSS: 50-70%\n  BOD: 25-40%\n  COD: 25-35%\n");
        }
        "secondary" | "sekunder" => {
            out.push_str("  TSS: 85-95%\n  BOD: 85-95%\n");
        }
        _ => {}
    }

    out
}
