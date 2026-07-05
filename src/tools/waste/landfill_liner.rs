/// Desain Sistem Liner TPA
/// Ref: PermenPU 3/2013 tentang Penyelenggaraan Prasarana dan Sarana Persampahan
/// Ref: Giroud & Bonaparte (1989) — Leakage through Liners

pub fn design(liner_type: &str, area_m2: f64, head_on_liner_m: f64, k_clay: f64, clay_thickness_m: f64) -> String {
    if area_m2 <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if head_on_liner_m < 0.0 { return "ERROR [E102]: Parameter tidak boleh negatif.".into(); }
    if k_clay <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }
    if clay_thickness_m <= 0.0 { return "ERROR [E102]: Parameter harus > 0.".into(); }

    let liner_lower = liner_type.to_lowercase();
    let (liner_name, leakage_rate_m3_s, geomembrane_mm, description) = match liner_lower.as_str() {
        "single_clay" => {
            // Q = k × i × A, i = (h + d) / d
            let gradient = (head_on_liner_m + clay_thickness_m) / clay_thickness_m;
            let q = k_clay * gradient * area_m2; // m³/s
            (
                "Single Clay Liner",
                q,
                0.0,
                "Liner tunggal lempung padat. Cocok untuk TPA kelas III (non-B3, rendah risiko).",
            )
        }
        "composite" => {
            // Giroud-Bonaparte (good contact):
            // Q = 0.21 × a^0.1 × h^0.9 × ks^0.74
            // a = defect area per hole ≈ 1 cm² = 1e-4 m², assume 2.5 holes/ha
            let defect_area = 1e-4_f64; // m² per hole
            let holes_per_m2 = 2.5 / 10000.0; // 2.5 holes per hectare
            let n_holes = (area_m2 * holes_per_m2).max(1.0);
            let q_per_hole = 0.21 * defect_area.powf(0.1) * head_on_liner_m.max(0.01).powf(0.9) * k_clay.powf(0.74);
            let q = q_per_hole * n_holes;
            (
                "Composite Liner (Geomembrane + Clay)",
                q,
                1.5,
                "Geomembrane HDPE di atas lapisan clay padat. Standar TPA kelas I.",
            )
        }
        "double_liner" => {
            // Double liner: primary + leak detection + secondary
            // Very low leakage, use composite formula with extra reduction factor
            let defect_area = 1e-4_f64;
            let holes_per_m2 = 1.0 / 10000.0; // stricter QC, 1 hole/ha
            let n_holes = (area_m2 * holes_per_m2).max(1.0);
            let q_per_hole = 0.21 * defect_area.powf(0.1) * head_on_liner_m.max(0.01).powf(0.9) * k_clay.powf(0.74);
            let q = q_per_hole * n_holes * 0.1; // secondary liner reduces by ~10x
            (
                "Double Liner (Primary + LDS + Secondary)",
                q,
                2.0,
                "Sistem liner ganda dengan leak detection system. Wajib untuk TPA B3.",
            )
        }
        _ => {
            return format!(
                "ERROR: Tipe liner '{}' tidak dikenal.\nPilihan: single_clay, composite, double_liner",
                liner_type
            );
        }
    };

    let leakage_l_day = leakage_rate_m3_s * 86400.0 * 1000.0;
    let leakage_l_ha_day = if area_m2 > 0.0 { leakage_l_day / (area_m2 / 10000.0) } else { 0.0 };

    // Compliance checks per PermenPU 3/2013
    let k_compliant = k_clay <= 1e-9;
    let thickness_compliant = clay_thickness_m >= 0.6;
    let k_status = if k_compliant { "MEMENUHI (≤ 1×10⁻⁹ m/s)" } else { "TIDAK MEMENUHI (harus ≤ 1×10⁻⁹ m/s)" };
    let thick_status = if thickness_compliant { "MEMENUHI (≥ 60 cm)" } else { "TIDAK MEMENUHI (harus ≥ 60 cm)" };

    let mut result = String::new();
    result.push_str("══════════════════════════════════════════════\n");
    result.push_str("DESAIN SISTEM LINER TPA\n");
    result.push_str("Ref: PermenPU 3/2013, Giroud & Bonaparte (1989)\n");
    result.push_str("══════════════════════════════════════════════\n\n");

    result.push_str(&format!("Tipe Liner       : {}\n", liner_name));
    result.push_str(&format!("Deskripsi        : {}\n", description));
    result.push_str(&format!("Luas area        : {:.2} m² ({:.2} Ha)\n", area_m2, area_m2 / 10000.0));
    result.push_str(&format!("Head on liner    : {:.3} m\n", head_on_liner_m));
    result.push_str(&format!("K clay           : {:.2e} m/s\n", k_clay));
    result.push_str(&format!("Tebal clay       : {:.2} m\n\n", clay_thickness_m));

    result.push_str("HASIL PERHITUNGAN:\n");
    result.push_str(&format!("• Laju kebocoran       : {:.4e} m³/s\n", leakage_rate_m3_s));
    result.push_str(&format!("• Laju kebocoran       : {:.2} L/hari\n", leakage_l_day));
    result.push_str(&format!("• Laju kebocoran       : {:.2} L/Ha/hari\n\n", leakage_l_ha_day));

    if geomembrane_mm > 0.0 {
        result.push_str("SPESIFIKASI GEOMEMBRANE:\n");
        result.push_str(&format!("• Material             : HDPE\n"));
        result.push_str(&format!("• Ketebalan minimum    : {:.1} mm\n", geomembrane_mm));
        result.push_str("• Sambungan            : Double wedge weld\n");
        result.push_str("• Uji sambungan        : Vakum + tekanan udara\n\n");
    }

    result.push_str("KEPATUHAN PermenPU 3/2013:\n");
    result.push_str(&format!("• K clay               : {}\n", k_status));
    result.push_str(&format!("• Tebal clay           : {}\n", thick_status));

    result.push_str("\nPERSYARATAN DRAINASE:\n");
    result.push_str("• Lapisan drainase leachate min 30 cm (kerikil/geonet)\n");
    result.push_str("• K drainase ≥ 1×10⁻² m/s\n");
    result.push_str("• Kemiringan dasar min 2% menuju pipa pengumpul\n");

    result.push_str("\nPERSYARATAN QC:\n");
    result.push_str("• Uji densitas clay (Proctor ≥ 95%)\n");
    result.push_str("• Uji permeabilitas lapangan (Boutwell/BAT)\n");
    result.push_str("• Uji sambungan geomembrane (destructive & non-destructive)\n");
    result.push_str("• Survei kebocoran elektrik (pasca-pemasangan)\n");
    result.push_str("══════════════════════════════════════════════\n");

    result
}
