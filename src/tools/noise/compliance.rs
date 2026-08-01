/// Noise Compliance Checker
/// Ref: KepmenLH 48/1996 + ISO 9613-2

/// Get noise limit (dBA) for a given zone per KepmenLH 48/1996
fn zone_limit(zone: &str) -> Option<(f64, &'static str)> {
    match zone.to_lowercase().as_str() {
        "a" | "perumahan" | "residential" => Some((55.0, "Zona A - Perumahan/Pemukiman (55 dBA)")),
        "b" | "perdagangan" | "commercial" => Some((70.0, "Zona B - Perdagangan/Jasa (70 dBA)")),
        "c" | "industri" | "industrial" => Some((73.0, "Zona C - Industri (73 dBA)")),
        "d" | "hijau" | "green" | "konservasi" => {
            Some((50.0, "Zona D - Ruang Terbuka Hijau (50 dBA)"))
        }
        "rumahsakit" | "hospital" | "rs" => {
            Some((55.0, "Rumah Sakit / Fasilitas Kesehatan (55 dBA)"))
        }
        "sekolah" | "school" => Some((55.0, "Sekolah / Lembaga Pendidikan (55 dBA)")),
        "ibadah" | "worship" => Some((55.0, "Tempat Ibadah (55 dBA)")),
        _ => None,
    }
}

/// Calculate noise level at distance using ISO 9613-2 spherical divergence
/// L = Lw - 20*log10(r) - 11
/// Where Lw = sound power level (dB), r = distance (m)
fn noise_at_distance(source_db: f64, distance_m: f64) -> f64 {
    if distance_m <= 0.0 {
        return source_db;
    }
    // ISO 9613-2 geometric divergence for point source in free field
    // Additional ground absorption approximation: A_ground ≈ 4.8 - (2*h/r)*(17 + 300/r)
    // Simplified: use basic spherical divergence + minimal ground attenuation
    let geometric_divergence = 20.0 * distance_m.log10() + 11.0;
    let a_ground = if distance_m > 100.0 {
        3.0
    } else {
        distance_m / 100.0 * 3.0
    };
    source_db - geometric_divergence - a_ground
}

/// Calculate required buffer distance for compliance
fn required_buffer(source_db: f64, limit_db: f64) -> f64 {
    // L = Lw - 20*log10(r) - 11 - A_ground = limit_db
    // Iterative approach since A_ground depends on distance
    let mut r = 1.0_f64;
    loop {
        let level = noise_at_distance(source_db, r);
        if level <= limit_db || r > 50000.0 {
            break;
        }
        r *= 1.1;
    }
    r
}

pub fn check(zone: &str, measured_db: f64, distance_m: f64, source_db: f64) -> String {
    let (limit, zone_desc) = match zone_limit(zone) {
        Some(v) => v,
        None => return format!(
            "Error: Zona '{}' tidak dikenal.\nZona valid: a/perumahan, b/perdagangan, c/industri, d/hijau, rumahsakit, sekolah, ibadah",
            zone
        ),
    };

    // Calculate expected noise at measurement distance
    let expected_db = noise_at_distance(source_db, distance_m);

    // Determine compliance
    let measured_compliant = measured_db <= limit;
    let expected_compliant = expected_db <= limit;

    // Calculate required buffer
    let buffer_m = required_buffer(source_db, limit);

    // Excess above limit
    let measured_excess = measured_db - limit;
    let expected_excess = expected_db - limit;

    // Mitigation recommendations
    let mut mitigations = Vec::new();
    if !measured_compliant || !expected_compliant {
        let excess = if !measured_compliant {
            measured_excess
        } else {
            expected_excess
        };
        if excess > 0.0 && excess <= 5.0 {
            mitigations.push("- Penanaman vegetasi penyerap bunyi (pengurangan 3-5 dBA)");
            mitigations.push("- Pembatasan jam operasi kegiatan bising");
        }
        if excess > 5.0 && excess <= 15.0 {
            mitigations
                .push("- Pemasangan noise barrier (tembok/pagar beton, pengurangan 5-15 dBA)");
            mitigations.push("- Pemindahan sumber kebisingan lebih jauh dari reseptor");
            mitigations.push("- Penanaman jalur hijau peredam bunyi (min. 10m lebar)");
        }
        if excess > 15.0 {
            mitigations.push("- Enklosur penuh sumber kebisingan (pengurangan 15-30 dBA)");
            mitigations.push("- Relokasi sumber bising atau reseptor sensitif");
            mitigations.push("- Pembatasan operasi hanya pada jam tertentu");
            mitigations.push("- Kombinasi noise barrier + vegetasi + enklosur parsial");
        }
    }

    let status_measured = if measured_compliant {
        "MEMENUHI ✓"
    } else {
        "MELEBIHI ✗"
    };
    let status_expected = if expected_compliant {
        "MEMENUHI ✓"
    } else {
        "MELEBIHI ✗"
    };

    let mut result = format!(
        "══════════════════════════════════════════════\n\
         ANALISIS KEPATUHAN KEBISINGAN\n\
         Ref: KepmenLH No. 48/1996 + ISO 9613-2\n\
         ══════════════════════════════════════════════\n\n\
         ZONA: {}\n\
         Baku Mutu: {:.1} dBA\n\n\
         SUMBER KEBISINGAN:\n\
         • Tingkat daya bunyi (Lw): {:.1} dBA\n\
         • Jarak pengukuran: {:.1} m\n\n\
         HASIL PENGUKURAN:\n\
         • Terukur: {:.1} dBA → {}\n",
        zone_desc, limit, source_db, distance_m, measured_db, status_measured
    );

    if !measured_compliant {
        result.push_str(&format!(
            "  → Kelebihan: {:.1} dBA di atas baku mutu\n",
            measured_excess
        ));
    }

    result.push_str(&format!(
        "\nPREDIKSI ISO 9613-2:\n\
         • Prediksi pada {:.1} m: {:.1} dBA → {}\n",
        distance_m, expected_db, status_expected
    ));

    if !expected_compliant {
        result.push_str(&format!(
            "  → Kelebihan prediksi: {:.1} dBA di atas baku mutu\n",
            expected_excess
        ));
    }

    result.push_str(&format!(
        "\nJARAK BUFFER MINIMUM:\n\
         • Jarak minimum untuk memenuhi baku mutu {:.1} dBA: {:.0} m\n",
        limit, buffer_m
    ));

    if !mitigations.is_empty() {
        result.push_str("\nREKOMENDASI MITIGASI:\n");
        for m in &mitigations {
            result.push_str(&format!("{}\n", m));
        }
    } else {
        result.push_str(
            "\nSTATUS: Kebisingan memenuhi baku mutu, tidak diperlukan mitigasi tambahan.\n",
        );
    }

    result.push_str("\n══════════════════════════════════════════════\n");
    result
}
