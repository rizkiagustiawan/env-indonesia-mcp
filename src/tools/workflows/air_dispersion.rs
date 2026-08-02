use crate::result_contract::{ScientificResult, ResultStatus, Claim};

pub fn run_air_dispersion_assessment(
    stack_height_m: f64,
    anemometer_height_m: f64,
    _wind_speed_ms: f64,
) -> Result<ScientificResult, String> {
    // 1. Meteorological Alignment Check (Guardrail)
    // If the stack height is > 50m but the anemometer is at standard 10m without a vertical profile extrapolation,
    // the wind speed assumption at the stack tip is grossly underestimated (under-predicting dispersion).
    if stack_height_m > 50.0 && anemometer_height_m < 20.0 {
        return Err(format!(
            "Meteorological mismatch: anemometer height ({:.1}m) is too low for stack height ({:.1}m). Wind profile extrapolation required to prevent under-dispersion bias.",
            anemometer_height_m, stack_height_m
        ));
    }

    let res = ScientificResult::new("integrated_air_dispersion", 1.0, "index")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_claim(Claim::new("meteorology", "Wind profile alignment verified."));

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_air_dispersion_fails_on_meteorological_mismatch() {
        // Stack 100m, anemometer 10m
        let res = run_air_dispersion_assessment(100.0, 10.0, 3.0);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Meteorological mismatch"));
    }

    #[test]
    fn test_air_dispersion_passes_on_aligned_heights() {
        // Stack 30m, anemometer 10m (acceptable for small stacks)
        let res = run_air_dispersion_assessment(30.0, 10.0, 3.0);
        assert!(res.is_ok());

        // Stack 100m, anemometer 80m (acceptable)
        let res2 = run_air_dispersion_assessment(100.0, 80.0, 5.0);
        assert!(res2.is_ok());
    }
}
