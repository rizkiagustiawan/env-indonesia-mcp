pub fn check_sensor_resolution(resolution_m: f64, area_sqm: f64) -> Result<(), String> {
    let pixel_area = resolution_m * resolution_m;
    if area_sqm <= 4.0 * pixel_area {
        return Err(format!("Sensor resolution too coarse: pixel area {} sqm is too large for target area {} sqm. Need at least 4 pixels.", pixel_area, area_sqm));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_sensor_resolution_fails_if_too_coarse() {
        // 30m resolution (900 sqm pixel) for a 1000 sqm area should fail 
        // if we require at least 4 pixels (e.g. area must be > 4 * pixel_area)
        let result = check_sensor_resolution(30.0, 1000.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Sensor resolution"));
    }

    #[test]
    fn test_check_sensor_resolution_passes_if_fine_enough() {
        // 10m resolution (100 sqm pixel) for a 1000 sqm area (10 pixels) should pass
        let result = check_sensor_resolution(10.0, 1000.0);
        assert!(result.is_ok());
    }
}
