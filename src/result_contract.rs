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

    pub fn validate(&self) -> Result<(), String> {
        if self.source_kind.trim().is_empty() || self.source_identifier.trim().is_empty() {
            return Err("Provenance source fields must not be empty".to_string());
        }
        chrono::DateTime::parse_from_rfc3339(&self.acquisition_timestamp)
            .map_err(|_| "Provenance acquisition_timestamp must be RFC3339".to_string())?;
        if self.source_kind == "fallback"
            && self
                .fallback_reason
                .as_deref()
                .map_or(true, |reason| reason.trim().is_empty())
        {
            return Err("Fallback sources require a non-empty fallback reason".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrsReference {
    pub code: String,
    pub name: Option<String>,
}

impl CrsReference {
    pub fn new(code: &str) -> Result<Self, String> {
        let reference = Self {
            code: code.trim().to_string(),
            name: None,
        };
        reference.validate()?;
        Ok(reference)
    }

    pub fn epsg(code: u32) -> Self {
        Self {
            code: format!("EPSG:{}", code),
            name: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        let upper = self.code.to_ascii_uppercase();
        let valid_epsg = upper
            .strip_prefix("EPSG:")
            .and_then(|code| code.parse::<u32>().ok())
            .is_some_and(|code| code > 0);
        let valid_ogc = matches!(upper.as_str(), "OGC:CRS84" | "CRS84");
        if !valid_epsg && !valid_ogc {
            return Err(format!("Invalid CRS reference: {}", self.code));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactLineage {
    pub artifact_id: String,
    pub source_url: String,
    pub collection: Option<String>,
    pub item_id: Option<String>,
    pub asset_key: Option<String>,
    pub byte_length: u64,
    pub sha256: String,
    pub retrieved_at: String,
}

impl ArtifactLineage {
    pub fn new(artifact_id: &str, source_url: &str, byte_length: u64, sha256: &str, retrieved_at: &str) -> Self {
        Self {
            artifact_id: artifact_id.to_string(),
            source_url: source_url.to_string(),
            collection: None,
            item_id: None,
            asset_key: None,
            byte_length,
            sha256: sha256.to_string(),
            retrieved_at: retrieved_at.to_string(),
        }
    }

    pub fn with_identity(mut self, collection: &str, item_id: &str, asset_key: &str) -> Self {
        self.collection = Some(collection.to_string());
        self.item_id = Some(item_id.to_string());
        self.asset_key = Some(asset_key.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.artifact_id.trim().is_empty() || self.source_url.trim().is_empty() {
            return Err("Artifact lineage identity fields must not be empty".to_string());
        }
        let url = reqwest::Url::parse(&self.source_url)
            .map_err(|_| "Artifact lineage source_url must be a valid URL".to_string())?;
        if url.scheme() != "https" {
            return Err("Artifact lineage source_url must use HTTPS".to_string());
        }
        for (name, value) in [
            ("collection", self.collection.as_deref()),
            ("item_id", self.item_id.as_deref()),
            ("asset_key", self.asset_key.as_deref()),
        ] {
            if let Some(value) = value {
                if value.trim().is_empty() {
                    return Err(format!("Artifact lineage {} must not be empty", name));
                }
            }
        }
        if self.byte_length == 0 {
            return Err("Artifact lineage byte_length must be positive".to_string());
        }
        if self.sha256.len() != 64 || !self.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Artifact lineage sha256 must be a 64-character hexadecimal value".to_string());
        }
        chrono::DateTime::parse_from_rfc3339(&self.retrieved_at)
            .map_err(|_| "Artifact lineage retrieved_at must be RFC3339".to_string())?;
        Ok(())
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
    pub confidence: Option<f64>,
    pub crs: Option<CrsReference>,
    pub artifact_lineage: Option<ArtifactLineage>,
    pub artifact_path: Option<String>,
    pub manifest_path: Option<String>,
    pub limitations: Vec<String>,
    #[serde(default)]
    pub synthetic: bool,
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
            confidence: None,
            crs: None,
            artifact_lineage: None,
            artifact_path: None,
            manifest_path: None,
            limitations: vec![],
            synthetic: false,
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

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn with_crs(mut self, crs: CrsReference) -> Self {
        self.crs = Some(crs);
        self
    }

    pub fn with_artifact_lineage(mut self, lineage: ArtifactLineage) -> Self {
        self.artifact_lineage = Some(lineage);
        self
    }

    pub fn with_artifact_paths(mut self, artifact_path: &str, manifest_path: &str) -> Self {
        self.artifact_path = Some(artifact_path.to_string());
        self.manifest_path = Some(manifest_path.to_string());
        self
    }

    pub fn with_limitation(mut self, limitation: &str) -> Self {
        self.limitations.push(limitation.to_string());
        self
    }

    pub fn with_synthetic(mut self, synthetic: bool) -> Self {
        self.synthetic = synthetic;
        self
    }

    /// Fail-stop emission (EnviSmart audited-handoff pattern): runs `validate()`
    /// and marks the result `validation_failed` if it violates the scientific
    /// contract, then serialises to JSON for downstream agent chaining.
    pub fn emit_validated(mut self) -> String {
        if self.validate().is_err() {
            self.status = ResultStatus::ValidationFailed;
        }
        serde_json::to_string(&self).unwrap_or_default()
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.value.is_finite() {
            return Err("Value must be finite".to_string());
        }

        if let Some(u) = &self.uncertainty {
            if !u.lower.is_finite() || !u.upper.is_finite() || u.lower > u.upper {
                return Err("Uncertainty lower bound cannot be greater than upper bound".to_string());
            }
            if u.method.trim().is_empty() {
                return Err("Uncertainty method must not be empty".to_string());
            }
            if matches!(
                u.uncertainty_type,
                UncertaintyType::ConfidenceInterval | UncertaintyType::CredibleInterval
            ) {
                let level = u
                    .confidence_level
                    .ok_or("Confidence and credible intervals require a confidence_level")?;
                if !level.is_finite() || !(0.0..=1.0).contains(&level) {
                    return Err("Uncertainty confidence_level must be between 0 and 1".to_string());
                }
                if u.seed.is_none() {
                    return Err("Stochastic uncertainty requires a reproducible seed".to_string());
                }
            } else if let Some(level) = u.confidence_level {
                if !level.is_finite() || !(0.0..=1.0).contains(&level) {
                    return Err("Uncertainty confidence_level must be between 0 and 1".to_string());
                }
            }
        }

        if let Some(p) = &self.provenance {
            p.validate()?;
            if let Some(max_age) = p.max_age_days {
                if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&p.acquisition_timestamp) {
                    let age = chrono::Utc::now().signed_duration_since(ts.with_timezone(&chrono::Utc));
                    if age.num_days() > max_age as i64 {
                        return Err(format!("Source is stale: age {} days exceeds max {} days", age.num_days(), max_age));
                    }
                }
            }
        }

        if let Some(confidence) = self.confidence {
            if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
                return Err("Confidence must be between 0 and 1".to_string());
            }
        }
        if let Some(crs) = &self.crs {
            crs.validate()?;
        }
        if let Some(lineage) = &self.artifact_lineage {
            lineage.validate()?;
        }

        if matches!(self.status, ResultStatus::ScreeningOnly) {
            for claim in &self.claims {
                let lower = claim.claim_type.to_lowercase();
                if lower == "compliant" || lower == "approved" || lower == "safe" || lower == "legal" {
                    return Err(format!("Regulatory claim '{}' forbidden for screening-only results", claim.claim_type));
                }
            }
        }

        if self.synthetic && matches!(self.status, ResultStatus::Valid) {
            return Err("Synthetic data cannot produce a Valid result".into());
        }

        Ok(())
    }
}
