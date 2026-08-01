//! Multi-Source Noise Calculator — Logarithmic Addition + Leq
//! Lp_total = 10·log₁₀(Σ 10^(Lpi/10))
//! Leq = 10·log₁₀((1/T) Σ ti × 10^(Li/10))
//! Ref: ISO 9613-2, KepmenLH 48/1996

pub fn calculate(sources_json: &str) -> String {
    // Parse JSON: [{"name":"Genset","db":85,"duration_hours":8},...]
    let sources: Vec<serde_json::Value> = match serde_json::from_str(sources_json) {
        Ok(v) => v,
        Err(e) => return format!("ERROR: JSON tidak valid — {}", e),
    };

    if sources.is_empty() {
        return "ERROR: Minimal 1 sumber bising.".to_string();
    }

    let mut result = "=== MULTI-SOURCE NOISE (Logarithmic Addition) ===\n\
                      Ref: ISO 9613-2, KepmenLH 48/1996\n\n\
                      SUMBER BISING:\n"
        .to_string();

    let mut sum_power = 0.0_f64;
    let mut sum_leq = 0.0_f64;
    let mut total_time = 0.0_f64;

    for (i, src) in sources.iter().enumerate() {
        let name = src
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let db = src.get("db").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let dist = src
            .get("distance_m")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let dur = src
            .get("duration_hours")
            .and_then(|v| v.as_f64())
            .unwrap_or(24.0);

        // Distance attenuation (point source, hemispherical spreading)
        let db_at_receiver = db - 20.0 * dist.log10() - 8.0; // -8 for hemispherical

        sum_power += 10.0_f64.powf(db_at_receiver / 10.0);
        sum_leq += dur * 10.0_f64.powf(db_at_receiver / 10.0);
        total_time += dur;

        result.push_str(&format!(
            "  {}. {} — {} dBA @ sumber, {} dBA @ {:.0}m, {} jam/hari\n",
            i + 1,
            name,
            db,
            format!("{:.1}", db_at_receiver),
            dist,
            dur
        ));
    }

    let l_total = 10.0 * sum_power.log10();
    let leq = if total_time > 0.0 {
        10.0 * (sum_leq / total_time).log10()
    } else {
        0.0
    };

    // Day-Night Level (Ldn) — +10 dB penalty for nighttime (22:00-06:00)
    let ldn = leq + 3.0; // simplified approximation

    result.push_str(&format!(
        "\nHASIL:\n  L_total (simultaneous) = {:.1} dBA\n  Leq (time-weighted) = {:.1} dBA\n  Ldn (approx) = {:.1} dBA\n\n\
         CEK BAKU MUTU (KepmenLH 48/1996):\n  Perumahan (55 dBA): {}\n  Industri (70 dBA): {}\n  Rumah Sakit (55 dBA): {}\n  Sekolah (55 dBA): {}\n",
        l_total, leq, ldn,
        if leq <= 55.0 { "MEMENUHI" } else { "MELEBIHI" },
        if leq <= 70.0 { "MEMENUHI" } else { "MELEBIHI" },
        if leq <= 55.0 { "MEMENUHI" } else { "MELEBIHI" },
        if leq <= 55.0 { "MEMENUHI" } else { "MELEBIHI" },
    ));

    result
}
