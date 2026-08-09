/// Mine Reclamation & Post-Mining Plan — Kepmen ESDM 1827K/30/MEM/2018
/// 4 kriteria keberhasilan: area compliance, land re-contouring, revegetation, final completion
/// Canopy cover = kriteria paling sulit (35.63% sukses dalam 6 tahun)
/// Bond/jaminan reklamasi wajib di bank nasional
/// Ref: Kepmen ESDM 1827K/30/MEM/2018; Permen ESDM 26/2018; Amanah & Yunanto 2019 (Mine Closure, UWA)
pub fn assess(
    pit_area_ha: f64,
    overburden_area_ha: f64,
    post_mining_land_use: &str,
    revegetation_species: &str,
    target_canopy_cover_pct: f64,
    years_since_reclamation: u32,
    bond_rp: f64,
) -> String {
    let mut out = String::from("=== Mine Reclamation & Post-Mining Plan ===\n");
    out.push_str("Ref: Kepmen ESDM 1827K/30/MEM/2018; Permen ESDM 26/2018; Amanah & Yunanto 2019\n\n");

    let total_area = pit_area_ha + overburden_area_ha;

    out.push_str(&format!("Pit area: {:.1} ha\n", pit_area_ha));
    out.push_str(&format!("Overburden area: {:.1} ha\n", overburden_area_ha));
    out.push_str(&format!("Total reklamasi: {:.1} ha\n", total_area));
    out.push_str(&format!("Post-mining land use: {}\n", post_mining_land_use));
    out.push_str(&format!("Revegetation species: {}\n", revegetation_species));
    out.push_str(&format!("Target canopy cover: {:.0}%\n", target_canopy_cover_pct));
    out.push_str(&format!("Tahun sejak reklamasi: {}\n", years_since_reclamation));
    out.push_str(&format!("Bond/Jaminan: Rp {:.0}\n\n", bond_rp));

    // ─── 4 KRITERIA KEBERHASILAN (Kepmen 1827K/2018) ───
    out.push_str("═══ 4 KRITERIA KEBERHASILAN REKLAMASI ═══\n\n");

    // 1. Area Compliance
    out.push_str("1. AREA COMPLIANCE (Kepatuhan Luas)\n");
    out.push_str(&format!("   Target: {:.1} ha ter reklamasi\n", total_area));
    let area_reclaimed = total_area * (years_since_reclamation as f64 / 5.0).min(1.0);
    let area_pct = (area_reclaimed / total_area * 100.0).min(100.0);
    out.push_str(&format!("   Ter capai: {:.1} ha ({:.0}%)\n", area_reclaimed, area_pct));
    out.push_str(&format!("   Status: {}\n\n", if area_pct >= 100.0 { "✅ LULUS" } else if area_pct >= 80.0 { "⚠️ HAMPIR LULUS" } else { "❌ BELUM LULUS" }));

    // 2. Land Re-contouring
    out.push_str("2. LAND RE-CONTOURING (Penataan Ulang Lahan)\n");
    out.push_str("   Kriteria: stabilitas lereng, drainase, topsoil replacement\n");
    out.push_str("   - Slope stability: factor of safety ≥ 1.3 (static), ≥ 1.1 (pseudostatic)\n");
    out.push_str("   - Topsoil thickness: ≥ 20 cm\n");
    out.push_str("   - Drainase: sediment control structures (drop structure, silt fence)\n");
    let recontour_pct = match years_since_reclamation {
        y if y >= 3 => 100.0,
        y => y as f64 / 3.0 * 100.0,
    };
    out.push_str(&format!("   Status: {:.0}% tercapai {}\n\n", recontour_pct, if recontour_pct >= 100.0 { "✅" } else { "⚠️" }));

    // 3. Revegetation
    out.push_str("3. REVEGETATION (Revegetasi)\n");
    out.push_str("   Kriteria: survival rate ≥ 80%, canopy cover ≥ target\n");
    out.push_str(&format!("   Target canopy cover: {:.0}%\n", target_canopy_cover_pct));
    out.push_str(&format!("   Species: {}\n", revegetation_species));

    // Canopy cover growth curve (Amanah 2019: 6 years for 35.63% success)
    let canopy_growth_rate = target_canopy_cover_pct / 6.0; // linear approx
    let canopy_current = (canopy_growth_rate * years_since_reclamation as f64).min(target_canopy_cover_pct);
    out.push_str(&format!("   Canopy cover saat ini: {:.1}% (target: {:.0}%)\n", canopy_current, target_canopy_cover_pct));
    out.push_str(&format!("   Status: {}\n", if canopy_current >= target_canopy_cover_pct { "✅ LULUS" } else { "❌ BELUM LULUS — canopy cover paling sulit dicapai" }));

    out.push_str("\n   Note: Amanah & Yunanto (2019) menemukan hanya 35.63% dari 277 area\n");
    out.push_str("   mencapai sukses dalam 6 tahun. Canopy cover = kriteria tersulit.\n");
    out.push_str("   Faktor: species selection & plant spacing\n\n");

    // 4. Final Completion
    out.push_str("4. FINAL COMPLETION (Penyelesaian Akhir)\n");
    out.push_str("   Kriteria: semua 3 kriteria di atas tercapai + serah terima\n");
    let all_criteria_met = area_pct >= 100.0 && recontour_pct >= 100.0 && canopy_current >= target_canopy_cover_pct;
    out.push_str(&format!("   Status: {}\n\n", if all_criteria_met { "✅ SIAP SERAH TERIMA" } else { "❌ BELUM SIAP" }));

    // ─── Revegetation Schedule ───
    out.push_str("═══ JADWAL REVEGETASI ═══\n");
    out.push_str("  Tahun 1: Topsoil placement + tanam cover crop (legume: Centrosema, Calopogonium)\n");
    out.push_str("  Tahun 2: Tanam pohon pertama (fast-growing: Sengon, Jabon, Acacia)\n");
    out.push_str("  Tahun 3: Tanam pohon lanjutan + enrichment planting\n");
    out.push_str("  Tahun 4-5: Monitoring survival rate, replanting (gap planting)\n");
    out.push_str("  Tahun 6: Evaluasi canopy cover — target capai ≥ threshold\n");
    out.push_str(&format!("  Tahun {} (saat ini): Canopy = {:.1}%\n\n", years_since_reclamation, canopy_current));

    // ─── Post-Mining Land Use ───
    out.push_str("═══ POST-MINING LAND USE PLAN ═══\n");
    match post_mining_land_use.to_lowercase().as_str() {
        s if s.contains("hutan") || s.contains("forest") => {
            out.push_str("  → Hutan (refores­ta­si): AGB target ≥ 100 ton/ha dalam 10 tahun\n");
            out.push_str("  Species: indigenous + fast-growing mix\n");
        }
        s if s.contains("sawah") || s.contains("irigasi") || s.contains("agrikultur") => {
            out.push_str("  → Lahan pertanian: topsoil ≥ 30cm, drainase, irigasi\n");
        }
        s if s.contains("kolam") || s.contains("air") || s.contains("water") => {
            out.push_str("  → Kolam/sumber air: water quality sesuai PP 22/2021\n");
        }
        s if s.contains("wisata") || s.contains("rekreasi") || s.contains("tourism") => {
            out.push_str("  → Wisata/rekreasi: landscape design, akses jalan, fasilitas\n");
        }
        _ => {
            out.push_str("  → Mixed use (disesuaikan dengan RTRW)\n");
        }
    }
    out.push('\n');

    // ─── Mine Closure Cost ───
    out.push_str("═══ ESTIMASI BIAYA MINE CLOSURE ═══\n");
    let cost_per_ha = 25_000_000.0; // Rp 25 jt/ha (approx)
    let total_cost = total_area * cost_per_ha;
    out.push_str(&format!("  Biaya per ha: Rp {:.0}\n", cost_per_ha));
    out.push_str(&format!("  Total area: {:.1} ha\n", total_area));
    out.push_str(&format!("  >> Total estimasi: Rp {:.0}\n", total_cost));
    out.push_str(&format!("  Bond di bank: Rp {:.0}\n", bond_rp));
    if bond_rp >= total_cost {
        out.push_str("  ✅ Bond mencukupi\n\n");
    } else {
        out.push_str(&format!("  ❌ Bond KURANG Rp {:.0} — tambah jaminan!\n\n", total_cost - bond_rp));
    }

    // ─── Mitigation ───
    out.push_str("═══ REKOMENDASI ═══\n");
    if canopy_current < target_canopy_cover_pct {
        out.push_str("  1. Prioritas: perbaiki canopy cover (paling sulit)\n");
        out.push_str("  2. Evaluasi species selection — pilih yang adaptif\n");
        out.push_str("  3. Sesuaikan plant spacing (lebih rapat jika perlu)\n");
        out.push_str("  4. Intensifikasi maintenance (pemupukan, pest control)\n");
    } else {
        out.push_str("  1. Lanjutkan monitoring\n");
        out.push_str("  2. Siapkan dokumen serah terima\n");
    }

    // ─── Monitoring & Reporting ───
    out.push_str("\n═══ PEMANTAUAN (RPL) ═══\n");
    out.push_str("  Parameter: luas area, slope stability (FS), survival rate, canopy cover\n");
    out.push_str("  Frekuensi: Tahunan (wajib dilaporkan ke ESDM)\n");
    out.push_str("  Lokasi: Seluruh area reklamasi\n\n");

    out.push_str("═══ PELAPORAN & IZIN ═══\n");
    out.push_str("  Permen ESDM 26/2018: Rencana reklamasi wajib dalam dokumen IUP\n");
    out.push_str("  Kepmen ESDM 1827K/2018: Pedoman Kaidah Teknik Pertambangan\n");
    out.push_str("  PP 22/2021: Persetujuan Lingkungan (AMDAL)\n");
    out.push_str("  Amdalnet + OSS: Pelaporan RKL-RPL\n");

    out.push_str("\n  Ref: Kepmen ESDM 1827K/30/MEM/2018; Permen ESDM 26/2018; Amanah & Yunanto 2019\n");
    out
}
