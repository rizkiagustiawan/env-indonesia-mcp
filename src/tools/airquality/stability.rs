/// Pasquill-Gifford Stability Class Estimation
/// Ref: Turner (1970), EPA AERMOD

pub fn estimate(wind_speed_ms: f64, solar_radiation: &str, cloud_cover_eighths: u32) -> String {
    if wind_speed_ms < 0.0 {
        return "ERROR [E102]: Parameter tidak boleh negatif.".into();
    }

    let is_night = solar_radiation == "night";
    let solar = match solar_radiation.to_lowercase().as_str() {
        "strong" | "kuat" => 3,
        "moderate" | "sedang" => 2,
        "slight" | "lemah" => 1,
        "night" | "malam" => 0,
        _ => return "ERROR: solar_radiation harus: strong/moderate/slight/night".into(),
    };

    let class = if is_night {
        if cloud_cover_eighths >= 4 {
            // overcast night: LESS stable (clouds trap longwave) — Turner 1970
            if wind_speed_ms < 2.0 {
                'E'
            } else if wind_speed_ms < 3.0 {
                'E'
            } else if wind_speed_ms < 5.0 {
                'D'
            } else {
                'D'
            }
        } else {
            // clear night: MORE stable
            if wind_speed_ms < 2.0 {
                'F'
            } else if wind_speed_ms < 3.0 {
                'F'
            } else if wind_speed_ms < 5.0 {
                'E'
            } else {
                'D'
            }
        }
    } else {
        match (wind_speed_ms.round() as u32, solar) {
            (0..=1, 3) => 'A',
            (0..=1, 2) => 'A',
            (0..=1, 1) => 'B',
            (2..=2, 3) => 'A',
            (2..=2, 2) => 'B',
            (2..=2, 1) => 'C',
            (3..=4, 3) => 'B',
            (3..=4, 2) => 'B',
            (3..=4, 1) => 'C',
            (5..=5, 3) => 'C',
            (5..=5, 2) => 'C',
            (5..=5, 1) => 'D',
            _ => 'D',
        }
    };

    let desc = match class {
        'A' => "Sangat Tidak Stabil (very unstable) — konveksi kuat, dispersi maksimal",
        'B' => "Tidak Stabil (moderately unstable) — dispersi baik",
        'C' => "Sedikit Tidak Stabil (slightly unstable)",
        'D' => "Netral — angin kencang atau mendung penuh",
        'E' => "Sedikit Stabil (slightly stable) — malam, dispersi terbatas",
        'F' => "Stabil (moderately stable) — malam tenang, polutan terperangkap",
        _ => "Unknown",
    };

    let mut out = String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  Stability Class (Turner 1970)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str(&format!(
        "Wind: {:.1} m/s\nRadiasi: {}\nCloud: {}/8\n\n",
        wind_speed_ms, solar_radiation, cloud_cover_eighths
    ));
    out.push_str(&format!("Kelas Stabilitas: {} — {}\n", class, desc));
    out
}

#[cfg(test)]
mod tests {
    use super::estimate;

    #[test]
    fn overcast_night_less_stable_than_clear() {
        // Overcast (≥4/8) night, 1.5 m/s → E (was F, duplicated clear branch)
        let overcast = estimate(1.5, "night", 6);
        assert!(overcast.contains("Kelas Stabilitas: E"), "overcast night should be E:\n{overcast}");
        // Clear (≤3/8) night, 1.5 m/s → F
        let clear = estimate(1.5, "night", 2);
        assert!(clear.contains("Kelas Stabilitas: F"), "clear night should be F:\n{clear}");
    }
}

pub fn get_sigma(class: char, x_m: f64) -> (f64, f64) {
    let (sy, sz) = match class {
        'A' => (0.22 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5), 0.20 * x_m),
        'B' => (0.16 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5), 0.12 * x_m),
        'C' => (
            0.11 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5),
            0.08 * x_m * (1.0 + 0.0002 * x_m).powf(-0.5),
        ),
        'D' => (
            0.08 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5),
            0.06 * x_m * (1.0 + 0.0015 * x_m).powf(-0.5),
        ),
        'E' => (
            0.06 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5),
            0.03 * x_m * (1.0 + 0.0003 * x_m).powf(-1.0),
        ),
        'F' | _ => (
            0.04 * x_m * (1.0 + 0.0001 * x_m).powf(-0.5),
            0.016 * x_m * (1.0 + 0.0003 * x_m).powf(-1.0),
        ),
    };
    (sy, sz)
}
