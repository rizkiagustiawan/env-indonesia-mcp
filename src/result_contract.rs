use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    Valid,
    ValidWithAssumptions,
    ScreeningOnly,
    InsufficientData,
    OutOfDomain,
    ValidationFailed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyType {
    ConfidenceInterval,
    PredictionInterval,
    CredibleInterval,
    Bound,
    SensitivityRange,
    NotAvailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Uncertainty {
    pub uncertainty_type: UncertaintyType,
    pub lower: f64,
    pub upper: f64,
    pub method: String,
    pub confidence_level: Option<f64>,
    pub seed: Option<u64>,
}

impl Uncertainty {
    pub fn bound(lower: f64, upper: f64, method: &str) -> Self {
        Self {
            uncertainty_type: UncertaintyType::Bound,
            lower,
            upper,
            method: method.to_string(),
            confidence_level: None,
            seed: None,
        }
    }

    pub fn confidence_interval(lower: f64, upper: f64, level: f64) -> Self {
        Self {
            uncertainty_type: UncertaintyType::ConfidenceInterval,
            lower,
            upper,
            method: "statistical".to_string(),
            confidence_level: Some(level),
            seed: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub source_kind: String,
    pub source_identifier: String,
    pub acquisition_timestamp: String,
    pub fallback_reason: Option<String>,
    pub max_age_days: Option<u32>,
}

impl Provenance {
    pub fn new(source_kind: &str, identifier: &str, timestamp: &str) -> Self {
        Self {
            source_kind: source_kind.to_string(),
            source_identifier: identifier.to_string(),
            acquisition_timestamp: timestamp.to_string(),
            fallback_reason: None,
            max_age_days: None,
        }
    }

    pub fn with_max_age_days(mut self, days: u32) -> Self {
        self.max_age_days = Some(days);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    pub claim_type: String,
    pub description: String,
}

impl Claim {
    pub fn new(claim_type: &str, description: &str) -> Self {
        Self {
            claim_type: claim_type.to_string(),
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScientificResult {
    pub parameter: String,
    pub value: f64,
    pub unit: String,
    pub status: ResultStatus,
    pub uncertainty: Option<Uncertainty>,
    pub provenance: Option<Provenance>,
    pub claims: Vec<Claim>,
}

impl ScientificResult {
    pub fn new(parameter: &str, value: f64, unit: &str) -> Self {
        Self {
            parameter: parameter.to_string(),
            value,
            unit: unit.to_string(),
            status: ResultStatus::Valid,
            uncertainty: None,
            provenance: None,
            claims: vec![],
        }
    }

    pub fn with_status(mut self, status: ResultStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_uncertainty(mut self, uncertainty: Uncertainty) -> Self {
        self.uncertainty = Some(uncertainty);
        self
    }

    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    pub fn with_claim(mut self, claim: Claim) -> Self {
        self.claims.push(claim);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.value.is_finite() {
            return Err("Value must be finite".to_string());
        }

        if let Some(u) = &self.uncertainty {
            if u.lower > u.upper {
                return Err("Uncertainty lower bound cannot be greater than upper bound".to_string());
            }
            if matches!(u.uncertainty_type, UncertaintyType::ConfidenceInterval | UncertaintyType::CredibleInterval) && u.seed.is_none() {
                return Err("Stochastic uncertainty requires a reproducible seed".to_string());
            }
        }

        if let Some(p) = &self.provenance {
            if p.source_kind == "fallback" && p.fallback_reason.is_none() {
                return Err("Fallback sources require an explicit fallback reason".to_string());
            }
            if let Some(max_age) = p.max_age_days {
                if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&p.acquisition_timestamp) {
                    let age = chrono::Utc::now().signed_duration_since(ts.with_timezone(&chrono::Utc));
                    if age.num_days() > max_age as i64 {
                        return Err(format!("Source is stale: age {} days exceeds max {} days", age.num_days(), max_age));
                    }
                }
            }
        }

        if matches!(self.status, ResultStatus::ScreeningOnly) {
            for claim in &self.claims {
                let lower = claim.claim_type.to_lowercase();
                if lower == "compliant" || lower == "approved" || lower == "safe" || lower == "legal" {
                    return Err(format!("Regulatory claim '{}' forbidden for screening-only results", claim.claim_type));
                }
            }
        }

        Ok(())
    }
}
