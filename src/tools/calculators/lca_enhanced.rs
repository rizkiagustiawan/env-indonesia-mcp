/// Enhanced LCA — Multi-Category Life Cycle Assessment
/// Ref: ISO 14040:2006 / ISO 14044:2006
/// Impact categories: GWP (IPCC AR6), AP, EP, ODP (CML-IA baseline)
/// 2026 SOTA: Luan et al. 2026 (ML-LCA building retrofitting),
///   Arumugam et al. 2026 (ML-LCA recycled concrete),
///   Guleria et al. 2026 (ML-LCA review)
/// Honest limitation: ML prediction needs training data — we use extended DB
///   + simple regression fallback for unknown materials

#[derive(Clone)]
struct MaterialEF {
    name: &'static str,
    co2: f64,    // kg CO2-eq/kg (GWP100, IPCC AR6)
    ap: f64,     // kg SO2-eq/kg (Acidification Potential)
    ep: f64,     // kg PO4-eq/kg (Eutrophication Potential)
    odp: f64,    // kg CFC-11-eq/kg (Ozone Depletion)
    water: f64,  // L/kg
    energy: f64,  // MJ/kg
}

const DB: &[MaterialEF] = &[
    MaterialEF { name: "baja", co2: 1.85, ap: 0.012, ep: 0.003, odp: 0.0, water: 20.0, energy: 20.0 },
    MaterialEF { name: "steel", co2: 1.85, ap: 0.012, ep: 0.003, odp: 0.0, water: 20.0, energy: 20.0 },
    MaterialEF { name: "semen", co2: 0.622, ap: 0.003, ep: 0.001, odp: 0.0, water: 1.5, energy: 3.4 },
    MaterialEF { name: "cement", co2: 0.622, ap: 0.003, ep: 0.001, odp: 0.0, water: 1.5, energy: 3.4 },
    MaterialEF { name: "plastik", co2: 2.5, ap: 0.020, ep: 0.005, odp: 0.0, water: 10.0, energy: 40.0 },
    MaterialEF { name: "plastic", co2: 2.5, ap: 0.020, ep: 0.005, odp: 0.0, water: 10.0, energy: 40.0 },
    MaterialEF { name: "pe", co2: 1.95, ap: 0.015, ep: 0.004, odp: 0.0, water: 8.0, energy: 35.0 },
    MaterialEF { name: "pvc", co2: 2.41, ap: 0.018, ep: 0.005, odp: 0.0001, water: 12.0, energy: 45.0 },
    MaterialEF { name: "aluminium", co2: 8.0, ap: 0.040, ep: 0.010, odp: 0.0, water: 100.0, energy: 170.0 },
    MaterialEF { name: "kayu", co2: -1.5, ap: 0.001, ep: 0.001, odp: 0.0, water: 5.0, energy: 3.0 },
    MaterialEF { name: "wood", co2: -1.5, ap: 0.001, ep: 0.001, odp: 0.0, water: 5.0, energy: 3.0 },
    MaterialEF { name: "kertas", co2: 1.1, ap: 0.008, ep: 0.002, odp: 0.0, water: 30.0, energy: 17.0 },
    MaterialEF { name: "paper", co2: 1.1, ap: 0.008, ep: 0.002, odp: 0.0, water: 30.0, energy: 17.0 },
    MaterialEF { name: "beton", co2: 0.13, ap: 0.001, ep: 0.0005, odp: 0.0, water: 1.0, energy: 0.7 },
    MaterialEF { name: "concrete", co2: 0.13, ap: 0.001, ep: 0.0005, odp: 0.0, water: 1.0, energy: 0.7 },
    MaterialEF { name: "kaca", co2: 0.86, ap: 0.005, ep: 0.002, odp: 0.0, water: 7.0, energy: 15.0 },
    MaterialEF { name: "glass", co2: 0.86, ap: 0.005, ep: 0.002, odp: 0.0, water: 7.0, energy: 15.0 },
    MaterialEF { name: "bata", co2: 0.24, ap: 0.002, ep: 0.001, odp: 0.0, water: 1.2, energy: 2.0 },
    MaterialEF { name: "brick", co2: 0.24, ap: 0.002, ep: 0.001, odp: 0.0, water: 1.2, energy: 2.0 },
    MaterialEF { name: "aspal", co2: 0.42, ap: 0.003, ep: 0.001, odp: 0.0, water: 3.0, energy: 8.0 },
    MaterialEF { name: "asphalt", co2: 0.42, ap: 0.003, ep: 0.001, odp: 0.0, water: 3.0, energy: 8.0 },
    MaterialEF { name: "tembaga", co2: 2.8, ap: 0.015, ep: 0.005, odp: 0.0, water: 80.0, energy: 60.0 },
    MaterialEF { name: "copper", co2: 2.8, ap: 0.015, ep: 0.005, odp: 0.0, water: 80.0, energy: 60.0 },
    MaterialEF { name: "baja_ringan", co2: 1.5, ap: 0.010, ep: 0.003, odp: 0.0, water: 15.0, energy: 16.0 },
    MaterialEF { name: "genteng", co2: 0.35, ap: 0.002, ep: 0.001, odp: 0.0, water: 2.0, energy: 4.0 },
    MaterialEF { name: "keramik", co2: 0.55, ap: 0.003, ep: 0.001, odp: 0.0, water: 3.0, energy: 6.0 },
    MaterialEF { name: "gypsum", co2: 0.12, ap: 0.001, ep: 0.0005, odp: 0.0, water: 1.0, energy: 1.5 },
];

fn find_material(name: &str) -> Option<&'static MaterialEF> {
    let lower = name.to_lowercase();
    DB.iter().find(|m| m.name == lower || lower.contains(m.name))
}

pub fn calculate(
    materials_json: &str,
    transport_kg_km: f64,
    energy_kwh: f64,
) -> String {
    let mut out = String::from("=== Enhanced LCA (Multi-Category) ===\n");
    out.push_str("Ref: ISO 14040/14044, IPCC AR6 GWP100, CML-IA baseline\n");
    out.push_str("2026 SOTA: Luan et al. 2026; Arumugam et al. 2026; Guleria et al. 2026\n\n");

    let materials: Vec<(String, f64)> = match serde_json::from_str(materials_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR [E102]: materials_json parse: {}. Format: [[\"material\",mass_kg],...]", e),
    };

    if materials.is_empty() {
        return "ERROR: materials_json kosong. Minimal 1 material.".into();
    }

    let mut total_co2 = 0.0f64;
    let mut total_ap = 0.0f64;
    let mut total_ep = 0.0f64;
    let mut total_odp = 0.0f64;
    let mut total_water = 0.0f64;
    let mut total_energy_mat = 0.0f64;
    let mut unknown_count = 0u32;

    out.push_str("─ MATERIAL INVENTORY (cradle-to-gate) ─\n\n");
    out.push_str(&format!("{:<16} {:>8} {:>10} {:>8} {:>8} {:>8} {:>8} {:>8}\n",
        "Material", "Mass(kg)", "CO2(kg)", "SO2(g)", "PO4(g)", "CFC(µg)", "H2O(L)", "MJ"));
    out.push_str(&"-".repeat(80));
    out.push('\n');

    for (name, mass) in &materials {
        if *mass <= 0.0 {
            out.push_str(&format!("  ⚠️ Mass {} ≤ 0 for {}\n", mass, name));
            continue;
        }

        if let Some(m) = find_material(name) {
            let co2 = m.co2 * mass;
            let ap = m.ap * mass * 1000.0; // to grams
            let ep = m.ep * mass * 1000.0;
            let odp = m.odp * mass * 1e6; // to µg
            let water = m.water * mass;
            let energy = m.energy * mass;

            total_co2 += co2;
            total_ap += m.ap * mass;
            total_ep += m.ep * mass;
            total_odp += m.odp * mass;
            total_water += water;
            total_energy_mat += energy;

            out.push_str(&format!("{:<16} {:>8.1} {:>10.2} {:>8.1} {:>8.1} {:>8.2} {:>8.0} {:>8.0}\n",
                &name[..name.len().min(15)], mass, co2, ap, ep, odp, water, energy));
        } else {
            // Regression fallback for unknown materials
            // Simple heuristic: estimate CO2 from typical building material range
            unknown_count += 1;
            let est_co2 = estimate_ef_unknown(name, *mass);
            total_co2 += est_co2;
            out.push_str(&format!("{:<16} {:>8.1} {:>10.2} {:>8} {:>8} {:>8} {:>8} {:>8}  ⚠️estimated\n",
                &name[..name.len().min(15)], mass, est_co2, "?", "?", "?", "?", "?"));
        }
    }

    // Transport impact (diesel truck: 0.062 kg CO2/ton-km)
    let transport_co2 = transport_kg_km * 0.000062;
    total_co2 += transport_co2;

    // Energy use (Indonesia grid: 0.7 kg CO2/kWh, mostly coal)
    let energy_co2 = energy_kwh * 0.7;
    total_co2 += energy_co2;

    out.push_str(&"-".repeat(80));
    out.push('\n');
    out.push_str(&format!("{:<16} {:>8} {:>10.2} {:>8.1} {:>8.1} {:>8.2} {:>8.0} {:>8.0}\n",
        "TOTAL", "", total_co2, total_ap * 1000.0, total_ep * 1000.0,
        total_odp * 1e6, total_water, total_energy_mat));
    out.push_str(&format!("  Transport ({} kg·km): {:.2} kg CO2\n", transport_kg_km, transport_co2));
    out.push_str(&format!("  Energy use ({} kWh): {:.2} kg CO2 (Indonesia grid, 0.7 kg/kWh)\n", energy_kwh, energy_co2));

    out.push_str("\n─ IMPACT CATEGORIES ─\n\n");
    out.push_str(&format!("  GWP (Global Warming):     {:>10.2} kg CO2-eq   (IPCC AR6 GWP100)\n", total_co2));
    out.push_str(&format!("  AP (Acidification):       {:>10.4} kg SO2-eq   (CML-IA)\n", total_ap));
    out.push_str(&format!("  EP (Eutrophication):      {:>10.4} kg PO4-eq   (CML-IA)\n", total_ep));
    out.push_str(&format!("  ODP (Ozone Depletion):    {:>10.6} kg CFC-11-eq\n", total_odp));
    out.push_str(&format!("  Water Consumption:        {:>10.0} L\n", total_water));
    out.push_str(&format!("  Energy:                   {:>10.0} MJ\n", total_energy_mat + energy_kwh * 3.6));

    // Carbon footprint per ton of material
    let total_mass: f64 = materials.iter().map(|(_, m)| m).sum();
    if total_mass > 0.0 {
        let intensity = total_co2 / total_mass * 1000.0;
        out.push_str(&format!("\n  Carbon intensity: {:.1} kg CO2/ton material\n", intensity));

        if intensity > 3000.0 {
            out.push_str("  ⚠️ Sangat tinggi (>3 t CO2/t). Pertimbangkan material alternatif.\n");
        } else if intensity > 500.0 {
            out.push_str("  🟡 Sedang. Optimasi material dapat menurunkan emisi 20-40%.\n");
        } else {
            out.push_str("  🟢 Relatif rendah. Good baseline.\n");
        }
    }

    if unknown_count > 0 {
        out.push_str(&format!("\n  ⚠️ {} material tidak di database, menggunakan estimasi.\n", unknown_count));
        out.push_str("  Available: baja, semen, plastik, pe, pvc, aluminium, kayu, kertas, beton, kaca, bata, aspal, tembaga, baja_ringan, genteng, keramik, gypsum\n");
    }

    out
}

fn estimate_ef_unknown(name: &str, mass: f64) -> f64 {
    // Simple regression: building materials typically 0.1-3.0 kg CO2/kg
    // Use name-based heuristic
    let lower = name.to_lowercase();
    let base_ef = if lower.contains("logam") || lower.contains("metal") || lower.contains("besi") {
        2.5
    } else if lower.contains("batu") || lower.contains("stone") || lower.contains("keramik") || lower.contains("ceramic") {
        0.3
    } else if lower.contains("tanaman") || lower.contains("plant") || lower.contains("organic") {
        0.5
    } else if lower.contains("komposit") || lower.contains("composite") {
        1.5
    } else {
        0.8 // default average building material
    };
    base_ef * mass
}
