pub fn check_sensor_resolution(resolution_m: f64, area_sqm: f64) -> Result<(), String> {
    let pixel_area = resolution_m * resolution_m;
    if area_sqm <= 4.0 * pixel_area {
        return Err(format!("Sensor resolution too coarse: pixel area {} sqm is too large for target area {} sqm. Need at least 4 pixels.", pixel_area, area_sqm));
    }
    Ok(())
}

pub fn check_temporal_alignment(data_season: &str, target_season: &str) -> Result<(), String> {
    if data_season != target_season {
        return Err(format!("Temporal mismatch: data season '{}' does not match target season '{}'", data_season, target_season));
    }
    Ok(())
}

pub fn check_spatial_independence(coords: &[(f64, f64)], min_distance_m: f64) -> Result<(), String> {
    for i in 0..coords.len() {
        for j in (i + 1)..coords.len() {
            let p1 = coords[i];
            let p2 = coords[j];
            let d = haversine_distance(p1, p2);
            if d < min_distance_m {
                return Err(format!("Spatial bias detected: points {} and {} are too close ({}m < {}m)", i, j, d, min_distance_m));
            }
        }
    }
    Ok(())
}

fn haversine_distance(p1: (f64, f64), p2: (f64, f64)) -> f64 {
    let r = 6371000.0; // Earth radius in meters
    let phi1 = p1.0.to_radians();
    let phi2 = p2.0.to_radians();
    let d_phi = (p2.0 - p1.0).to_radians();
    let d_lambda = (p2.1 - p1.1).to_radians();

    let a = (d_phi / 2.0).sin() * (d_phi / 2.0).sin() +
            phi1.cos() * phi2.cos() *
            (d_lambda / 2.0).sin() * (d_lambda / 2.0).sin();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_sensor_resolution_fails_if_too_coarse() {
        let result = check_sensor_resolution(30.0, 1000.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Sensor resolution"));
    }

    #[test]
    fn test_check_sensor_resolution_passes_if_fine_enough() {
        let result = check_sensor_resolution(10.0, 1000.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_temporal_alignment_fails_on_mismatch() {
        let result = check_temporal_alignment("dry", "wet");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Temporal mismatch"));
    }

    #[test]
    fn test_check_temporal_alignment_passes_on_match() {
        let result = check_temporal_alignment("dry", "dry");
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_spatial_independence_fails_if_clustered() {
        let coords = vec![
            (0.0, 0.0),
            (0.0001, 0.0001), // Very close
        ];
        let result = check_spatial_independence(&coords, 1000.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Spatial bias"));
    }

    #[test]
    fn test_check_spatial_independence_passes_if_separated() {
        let coords = vec![
            (0.0, 0.0),
            (1.0, 1.0), // Far away
        ];
        let result = check_spatial_independence(&coords, 1000.0);
        assert!(result.is_ok());
    }
}
