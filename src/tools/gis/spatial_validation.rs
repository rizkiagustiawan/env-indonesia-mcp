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
}
