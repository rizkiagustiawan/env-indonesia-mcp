use crate::result_contract::ResultStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MaturityLevel {
    InsufficientData,
    Screening,
    Conceptual,
    Calibrated,
    Validated,
}

impl MaturityLevel {
    pub fn rank(self) -> u8 {
        match self {
            MaturityLevel::InsufficientData => 0,
            MaturityLevel::Screening => 1,
            MaturityLevel::Conceptual => 2,
            MaturityLevel::Calibrated => 3,
            MaturityLevel::Validated => 4,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DataAvailability {
    pub satellite_context: bool,
    pub regional_dem: bool,
    pub local_dem: bool,
    pub field_observations: bool,
    pub calibration_observations: bool,
    pub independent_validation: bool,
    pub synthetic_field_data: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateDecision {
    pub requested_level: MaturityLevel,
    pub allowed_level: MaturityLevel,
    pub blocked: bool,
    pub missing: Vec<String>,
    pub synthetic: bool,
}

pub fn assess_level(availability: &DataAvailability) -> MaturityLevel {
    if availability.synthetic_field_data {
        // Synthetic field data provides uncalibrated field context
        // (Conceptual at most); calibration/validation flags are ignored
        // so it can never reach Calibrated or Validated.
        if availability.satellite_context || availability.regional_dem || availability.local_dem {
            return MaturityLevel::Conceptual;
        }
        return MaturityLevel::InsufficientData;
    }
    if availability.independent_validation && availability.calibration_observations {
        return MaturityLevel::Validated;
    }
    if availability.calibration_observations {
        return MaturityLevel::Calibrated;
    }
    if availability.field_observations || availability.local_dem {
        return MaturityLevel::Conceptual;
    }
    if availability.regional_dem || availability.satellite_context {
        return MaturityLevel::Screening;
    }
    MaturityLevel::InsufficientData
}

pub fn gate(requested: MaturityLevel, availability: &DataAvailability) -> GateDecision {
    let allowed = assess_level(availability);
    let blocked = allowed.rank() < requested.rank();
    let missing = if blocked {
        missing_requirements(requested, availability)
    } else {
        Vec::new()
    };
    GateDecision {
        requested_level: requested,
        allowed_level: allowed,
        blocked,
        missing,
        synthetic: availability.synthetic_field_data,
    }
}

fn missing_requirements(requested: MaturityLevel, a: &DataAvailability) -> Vec<String> {
    let mut missing = Vec::new();
    if requested.rank() >= MaturityLevel::Conceptual.rank() && !a.field_observations && !a.local_dem {
        missing.push("field observations or local DEM".into());
    }
    if requested.rank() >= MaturityLevel::Calibrated.rank() && !a.calibration_observations {
        missing.push("calibration observations".into());
    }
    if requested.rank() >= MaturityLevel::Validated.rank() && !a.independent_validation {
        missing.push("independent validation observations".into());
    }
    missing
}

/// Map a maturity level onto the scientific result contract status.
///
/// Only `Validated` earns `ResultStatus::Valid`. `Calibrated` means the model
/// was fitted and checked but its independence was not established, so it still
/// carries assumptions.
pub fn to_result_status(level: MaturityLevel) -> ResultStatus {
    match level {
        MaturityLevel::InsufficientData => ResultStatus::InsufficientData,
        MaturityLevel::Screening => ResultStatus::ScreeningOnly,
        MaturityLevel::Conceptual | MaturityLevel::Calibrated => ResultStatus::ValidWithAssumptions,
        MaturityLevel::Validated => ResultStatus::Valid,
    }
}

pub fn parse_level(input: &str) -> MaturityLevel {
    match input.trim().to_ascii_lowercase().as_str() {
        "validated" => MaturityLevel::Validated,
        "calibrated" => MaturityLevel::Calibrated,
        "conceptual" => MaturityLevel::Conceptual,
        "screening" => MaturityLevel::Screening,
        _ => MaturityLevel::InsufficientData,
    }
}

/// Combine a *declared* data availability with an *earned* validation level.
///
/// The result is the weaker of the two: evidence can cap a claim but never
/// inflate it. With no evidence at all the result is capped at `Conceptual`,
/// because an unproven claim of calibration must not produce a `Valid` status.
pub fn assess_level_with_evidence(
    availability: &DataAvailability,
    earned: Option<MaturityLevel>,
) -> MaturityLevel {
    let declared = assess_level(availability);
    let ceiling = earned.unwrap_or(MaturityLevel::Conceptual);
    if ceiling.rank() < declared.rank() {
        ceiling
    } else {
        declared
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_data_is_insufficient() {
        let a = DataAvailability::default();
        assert_eq!(assess_level(&a), MaturityLevel::InsufficientData);
    }

    #[test]
    fn satellite_only_is_screening() {
        let a = DataAvailability { satellite_context: true, ..Default::default() };
        assert_eq!(assess_level(&a), MaturityLevel::Screening);
    }

    #[test]
    fn synthetic_caps_at_conceptual_and_never_validated() {
        let a = DataAvailability {
            satellite_context: true,
            calibration_observations: true,
            independent_validation: true,
            synthetic_field_data: true,
            ..Default::default()
        };
        assert_eq!(assess_level(&a), MaturityLevel::Conceptual);
    }

    #[test]
    fn gate_blocks_request_above_available() {
        let a = DataAvailability { regional_dem: true, ..Default::default() };
        let d = gate(MaturityLevel::Validated, &a);
        assert!(d.blocked);
        assert_eq!(d.allowed_level, MaturityLevel::Screening);
        assert!(d.missing.iter().any(|m| m.contains("calibration")));
    }

    #[test]
    fn partial_availability_deserializes_with_defaults() {
        let a: DataAvailability = serde_json::from_str(r#"{"regional_dem":true}"#).unwrap();
        assert!(a.regional_dem);
        assert!(!a.synthetic_field_data);
        assert!(!a.satellite_context);
        assert_eq!(assess_level(&a), MaturityLevel::Screening);
    }

    #[test]
    fn evidence_can_only_cap_the_declared_level() {
        let a = DataAvailability {
            calibration_observations: true,
            independent_validation: true,
            ..Default::default()
        };
        assert_eq!(assess_level(&a), MaturityLevel::Validated);
        // Earned evidence is weaker than the claim: the weaker one wins.
        assert_eq!(
            assess_level_with_evidence(&a, Some(MaturityLevel::Screening)),
            MaturityLevel::Screening
        );
        // Earned evidence cannot exceed what the data supports.
        let thin = DataAvailability { satellite_context: true, ..Default::default() };
        assert_eq!(
            assess_level_with_evidence(&thin, Some(MaturityLevel::Validated)),
            MaturityLevel::Screening
        );
    }

    #[test]
    fn unproven_calibration_claim_caps_at_conceptual() {
        let a = DataAvailability {
            calibration_observations: true,
            independent_validation: true,
            ..Default::default()
        };
        assert_eq!(assess_level_with_evidence(&a, None), MaturityLevel::Conceptual);
    }

    #[test]
    fn only_validated_maps_to_valid_status() {
        assert_eq!(to_result_status(MaturityLevel::Validated), ResultStatus::Valid);
        // Calibrated is fitted but not independently confirmed: it must still
        // carry assumptions rather than claim full validity.
        assert_eq!(
            to_result_status(MaturityLevel::Calibrated),
            ResultStatus::ValidWithAssumptions
        );
        assert_eq!(
            to_result_status(MaturityLevel::Conceptual),
            ResultStatus::ValidWithAssumptions
        );
        assert_eq!(
            to_result_status(MaturityLevel::Screening),
            ResultStatus::ScreeningOnly
        );
        assert_eq!(
            to_result_status(MaturityLevel::InsufficientData),
            ResultStatus::InsufficientData
        );
    }
}
