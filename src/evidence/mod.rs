use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Official,
    Sensor,
    Scientific,
    LicensedMedia,
    Ngo,
    Industry,
    Social,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimType {
    Observation,
    Allegation,
    OfficialFinding,
    ModelResult,
    Contextual,
}

impl Default for ClaimType {
    fn default() -> Self {
        Self::Contextual
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    ScreeningOnly,
    InsufficientData,
    Corroborated,
    Contradictory,
    HumanReview,
    OfficiallyConfirmed,
    Retracted,
}

impl Default for EvidenceStatus {
    fn default() -> Self {
        Self::InsufficientData
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IncidentState {
    Detected,
    Corroborating,
    HumanReview,
    ScientificallyAssessed,
    OfficiallyConfirmed,
    Retracted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecisionType {
    AcceptAsSignal,
    RequestMoreEvidence,
    Reject,
    ConfirmOfficialSource,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SourceRecord {
    pub source_id: String,
    pub source_kind: Option<SourceKind>,
    pub authority_tier: u8,
    pub publisher: String,
    pub canonical_url: Option<String>,
    pub license: Option<String>,
    pub jurisdiction: Option<String>,
    pub independence_group: String,
    pub published_at: Option<String>,
    pub acquired_at: String,
    pub content_sha256: String,
}

impl SourceRecord {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_id.trim().is_empty() {
            return Err("source_id is required".into());
        }
        if !(1..=5).contains(&self.authority_tier) {
            return Err("authority_tier must be between 1 and 5".into());
        }
        if self.independence_group.trim().is_empty() {
            return Err("independence_group is required".into());
        }
        if self.content_sha256.trim().is_empty() {
            return Err("content_sha256 is required".into());
        }
        if chrono::DateTime::parse_from_rfc3339(&self.acquired_at).is_err() {
            return Err("acquired_at must be RFC3339".into());
        }
        Ok(())
    }

    #[cfg(test)]
    fn test_official(id: &str, hash: &str, acquired_at: &str) -> Self {
        Self {
            source_id: id.into(),
            source_kind: Some(SourceKind::Official),
            authority_tier: 1,
            publisher: "test".into(),
            independence_group: id.into(),
            acquired_at: acquired_at.into(),
            content_sha256: hash.into(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactRecord {
    pub artifact_id: String,
    pub source_id: String,
    pub content_sha256: String,
    pub media_type: String,
    pub locator: Option<String>,
    pub bbox: Option<[f64; 4]>,
    pub crs: Option<String>,
    pub valid_time_start: Option<String>,
    pub valid_time_end: Option<String>,
    pub acquired_at: String,
    pub published_at: Option<String>,
}

impl ArtifactRecord {
    pub fn from_bytes(
        artifact_id: &str,
        source_id: &str,
        media_type: &str,
        payload: &[u8],
        acquired_at: &str,
    ) -> Result<Self, String> {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let content_sha256 = format!("{:x}", hasher.finalize());
        let artifact = Self {
            artifact_id: artifact_id.into(),
            source_id: source_id.into(),
            content_sha256,
            media_type: media_type.into(),
            acquired_at: acquired_at.into(),
            ..Self::default()
        };
        artifact.validate()?;
        Ok(artifact)
    }

    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("artifact_id", &self.artifact_id),
            ("source_id", &self.source_id),
            ("content_sha256", &self.content_sha256),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{} is required", name));
            }
        }
        if chrono::DateTime::parse_from_rfc3339(&self.acquired_at).is_err() {
            return Err("acquired_at must be RFC3339".into());
        }
        if let Some(bbox) = self.bbox {
            if bbox.iter().any(|value| !value.is_finite()) || bbox[0] > bbox[2] || bbox[1] > bbox[3]
            {
                return Err("bbox must be finite and ordered".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AuditEvent {
    pub event_id: String,
    pub activity: String,
    pub agent: String,
    pub target_id: String,
    pub payload_sha256: String,
    pub occurred_at: String,
    pub previous_event_sha256: Option<String>,
    pub event_sha256: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AuditLog {
    pub events: Vec<AuditEvent>,
}

impl AuditLog {
    pub fn append(
        &mut self,
        event_id: &str,
        activity: &str,
        agent: &str,
        target_id: &str,
        payload: &[u8],
        occurred_at: &str,
    ) -> Result<(), String> {
        if event_id.trim().is_empty()
            || activity.trim().is_empty()
            || agent.trim().is_empty()
            || target_id.trim().is_empty()
        {
            return Err("audit event identity fields are required".into());
        }
        if chrono::DateTime::parse_from_rfc3339(occurred_at).is_err() {
            return Err("occurred_at must be RFC3339".into());
        }
        let mut payload_hasher = Sha256::new();
        payload_hasher.update(payload);
        let payload_sha256 = format!("{:x}", payload_hasher.finalize());
        let previous_event_sha256 = self.events.last().map(|event| event.event_sha256.clone());
        let event_sha256 = hash_event(
            event_id,
            activity,
            agent,
            target_id,
            &payload_sha256,
            occurred_at,
            previous_event_sha256.as_deref(),
        );
        self.events.push(AuditEvent {
            event_id: event_id.into(),
            activity: activity.into(),
            agent: agent.into(),
            target_id: target_id.into(),
            payload_sha256,
            occurred_at: occurred_at.into(),
            previous_event_sha256,
            event_sha256,
        });
        Ok(())
    }

    pub fn verify_chain(&self) -> Result<(), String> {
        let mut previous = None;
        for event in &self.events {
            if event.previous_event_sha256 != previous {
                return Err(format!("audit chain link mismatch at {}", event.event_id));
            }
            let expected = hash_event(
                &event.event_id,
                &event.activity,
                &event.agent,
                &event.target_id,
                &event.payload_sha256,
                &event.occurred_at,
                event.previous_event_sha256.as_deref(),
            );
            if event.event_sha256 != expected {
                return Err(format!("audit event hash mismatch at {}", event.event_id));
            }
            previous = Some(event.event_sha256.clone());
        }
        Ok(())
    }
}

fn hash_event(
    event_id: &str,
    activity: &str,
    agent: &str,
    target_id: &str,
    payload_sha256: &str,
    occurred_at: &str,
    previous_event_sha256: Option<&str>,
) -> String {
    let canonical = format!(
        "{event_id}\n{activity}\n{agent}\n{target_id}\n{payload_sha256}\n{occurred_at}\n{}",
        previous_event_sha256.unwrap_or_default()
    );
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClaimRecord {
    pub claim_id: String,
    pub artifact_id: String,
    pub source_id: String,
    pub claim_type: ClaimType,
    pub subject_id: String,
    pub predicate: String,
    pub object_text: String,
    pub event_type: String,
    pub event_start: Option<String>,
    pub event_end: Option<String>,
    pub location: Option<[f64; 2]>,
    pub extraction_method: String,
    pub confidence: Option<f64>,
    pub status: EvidenceStatus,
    pub quote: Option<String>,
}

impl ClaimRecord {
    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("claim_id", &self.claim_id),
            ("artifact_id", &self.artifact_id),
            ("source_id", &self.source_id),
            ("subject_id", &self.subject_id),
            ("predicate", &self.predicate),
            ("object_text", &self.object_text),
            ("event_type", &self.event_type),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{} is required", name));
            }
        }
        if let Some(confidence) = self.confidence {
            if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
                return Err("confidence must be finite and between 0 and 1".into());
            }
        }
        if let Some(location) = self.location {
            if location.iter().any(|value| !value.is_finite()) {
                return Err("location must be finite".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentRecord {
    pub incident_id: String,
    pub event_type: String,
    pub subject_id: String,
    pub claim_ids: Vec<String>,
    pub state: IncidentState,
    pub evidence_status: EvidenceStatus,
    pub confidence: Option<f64>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewDecision {
    pub target_id: String,
    pub reviewer_id: String,
    pub reviewed_at: String,
    pub decision: ReviewDecisionType,
    pub rationale: String,
    pub right_of_reply_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EvidenceAssessmentRequest {
    pub sources: Vec<SourceRecord>,
    pub artifacts: Vec<ArtifactRecord>,
    pub claims: Vec<ClaimRecord>,
    #[schemars(description = "Minimum confidence required for corroboration, between 0 and 1")]
    pub confidence_threshold: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceAssessment {
    pub status: EvidenceStatus,
    pub state: IncidentState,
    pub confidence: Option<f64>,
    pub independent_source_count: usize,
    pub conflict_count: usize,
    pub claim_ids: Vec<String>,
    pub reasons: Vec<String>,
}

pub fn normalize_token(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn claim_key(claim: &ClaimRecord) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        normalize_token(&claim.subject_id),
        normalize_token(&claim.predicate),
        normalize_token(&claim.object_text),
        normalize_token(&claim.event_type),
        claim
            .event_start
            .as_deref()
            .map(normalize_token)
            .unwrap_or_default(),
        claim
            .location
            .map(|point| format!("{:.6},{:.6}", point[0], point[1]))
            .unwrap_or_default()
    )
}

fn claim_base_key(claim: &ClaimRecord) -> String {
    format!(
        "{}|{}|{}",
        normalize_token(&claim.subject_id),
        normalize_token(&claim.predicate),
        normalize_token(&claim.event_type)
    )
}

pub fn lineage_key(source: &SourceRecord) -> String {
    normalize_token(&source.independence_group)
}

pub fn assess_claims(
    sources: &[SourceRecord],
    artifacts: &[ArtifactRecord],
    claims: &[ClaimRecord],
    confidence_threshold: f64,
) -> Result<EvidenceAssessment, String> {
    if !confidence_threshold.is_finite() || !(0.0..=1.0).contains(&confidence_threshold) {
        return Err("confidence_threshold must be finite and between 0 and 1".into());
    }

    let mut source_map = HashMap::new();
    for source in sources {
        source.validate()?;
        source_map.insert(source.source_id.clone(), source);
    }
    let mut artifact_map = HashMap::new();
    for artifact in artifacts {
        artifact.validate()?;
        if !source_map.contains_key(&artifact.source_id) {
            return Err(format!(
                "artifact references unknown source: {}",
                artifact.source_id
            ));
        }
        artifact_map.insert(artifact.artifact_id.clone(), artifact);
    }
    for claim in claims {
        claim.validate()?;
        if !source_map.contains_key(&claim.source_id) {
            return Err(format!(
                "claim references unknown source: {}",
                claim.source_id
            ));
        }
        if !artifact_map.contains_key(&claim.artifact_id) {
            return Err(format!(
                "claim references unknown artifact: {}",
                claim.artifact_id
            ));
        }
    }
    if claims.is_empty() {
        return Ok(EvidenceAssessment {
            status: EvidenceStatus::InsufficientData,
            state: IncidentState::Detected,
            confidence: None,
            independent_source_count: 0,
            conflict_count: 0,
            claim_ids: vec![],
            reasons: vec!["no claims supplied".into()],
        });
    }

    let mut by_base: HashMap<String, HashMap<String, HashSet<String>>> = HashMap::new();
    for claim in claims {
        let source = source_map
            .get(&claim.source_id)
            .expect("claims were validated against source_map");
        by_base
            .entry(claim_base_key(claim))
            .or_default()
            .entry(normalize_token(&claim.object_text))
            .or_default()
            .insert(lineage_key(source));
    }

    let conflict_count = by_base
        .values()
        .filter(|objects| {
            let object_lineages = objects.values().collect::<Vec<_>>();
            object_lineages.iter().enumerate().any(|(index, lineages)| {
                object_lineages[index + 1..]
                    .iter()
                    .any(|other| {
                        lineages
                            .iter()
                            .any(|lineage| other.iter().any(|other_lineage| lineage != other_lineage))
                    })
            })
        })
        .count();
    let claim_ids = claims
        .iter()
        .map(|claim| claim.claim_id.clone())
        .collect::<Vec<_>>();
    let mut reasons = Vec::new();
    if conflict_count > 0 {
        reasons.push("independent claims conflict".into());
    }

    let strongest_group = claims.iter().filter_map(|claim| {
        let source = source_map.get(&claim.source_id)?;
        let confidence = claim.confidence.or_else(|| {
            (source.authority_tier == 1 && claim.claim_type == ClaimType::OfficialFinding)
                .then_some(1.0)
        })?;
        Some((
            lineage_key(source),
            confidence,
            source.authority_tier,
            claim.claim_type,
        ))
    });
    let mut group_confidence: HashMap<String, (f64, u8, ClaimType)> = HashMap::new();
    for (group, confidence, tier, claim_type) in strongest_group {
        group_confidence
            .entry(group)
            .and_modify(|existing| {
                if confidence > existing.0 {
                    *existing = (confidence, tier, claim_type);
                }
            })
            .or_insert((confidence, tier, claim_type));
    }
    let independent_source_count = group_confidence.len();
    let confidences = group_confidence
        .values()
        .map(|entry| entry.0)
        .collect::<Vec<_>>();
    let confidence = confidences.into_iter().min_by(f64::total_cmp);
    let official_single = group_confidence
        .values()
        .any(|(_, tier, claim_type)| *tier == 1 && *claim_type == ClaimType::OfficialFinding);

    let status = if conflict_count > 0 {
        EvidenceStatus::Contradictory
    } else if official_single || independent_source_count >= 2 {
        if confidence.unwrap_or(0.0) >= confidence_threshold {
            EvidenceStatus::Corroborated
        } else {
            reasons.push("confidence below configured threshold".into());
            EvidenceStatus::InsufficientData
        }
    } else {
        reasons.push("fewer than two independent evidence lineages".into());
        EvidenceStatus::InsufficientData
    };
    let state = match status {
        EvidenceStatus::Contradictory => IncidentState::HumanReview,
        EvidenceStatus::Corroborated => IncidentState::Corroborating,
        _ => IncidentState::Detected,
    };

    Ok(EvidenceAssessment {
        status,
        state,
        confidence,
        independent_source_count,
        conflict_count,
        claim_ids,
        reasons,
    })
}

/// Render an enum using its serde snake_case representation so tool output
/// matches the wire format (`human_review`, not `humanreview`).
fn snake_case_label<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn assessment_to_scientific_result(assessment: &EvidenceAssessment) -> ScientificResult {
    let mut result = ScientificResult::new(
        "evidence_independent_source_count",
        assessment.independent_source_count as f64,
        "count",
    )
    .with_status(ResultStatus::ScreeningOnly)
    .with_provenance(Provenance::new(
        "evidence_assessment",
        "env-indonesia-evidence-core",
        &chrono::Utc::now().to_rfc3339(),
    ))
    .with_claim(Claim::new(
        "evidence_status",
        &snake_case_label(&assessment.status),
    ))
    .with_claim(Claim::new(
        "incident_state",
        &snake_case_label(&assessment.state),
    ))
    .with_claim(Claim::new(
        "conflict_count",
        &assessment.conflict_count.to_string(),
    ))
    .with_claim(Claim::new(
        "claim_count",
        &assessment.claim_ids.len().to_string(),
    ))
    .with_claim(Claim::new(
        "abstention_reason",
        "Evidence core does not make legal or regulatory conclusions",
    ));
    if let Some(confidence) = assessment.confidence {
        result = result.with_confidence(confidence);
        result = result.with_claim(Claim::new("screening_confidence", &confidence.to_string()));
    }
    for reason in &assessment.reasons {
        result = result.with_claim(Claim::new("abstention_reason", reason));
    }
    result
}

pub fn emit_assessment(assessment: &EvidenceAssessment) -> String {
    assessment_to_scientific_result(assessment).emit_validated()
}

pub fn assess_request(request: &EvidenceAssessmentRequest) -> String {
    match assess_claims(
        &request.sources,
        &request.artifacts,
        &request.claims,
        request.confidence_threshold.unwrap_or(0.8),
    ) {
        Ok(assessment) => emit_assessment(&assessment),
        Err(error) => ScientificResult::new("evidence_assessment_error", 0.0, "dimensionless")
            .with_status(ResultStatus::ValidationFailed)
            .with_provenance(Provenance::new(
                "evidence_assessment",
                "env-indonesia-evidence-core",
                &chrono::Utc::now().to_rfc3339(),
            ))
            .with_claim(Claim::new("validation_error", &error))
            .emit_validated(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_hash_is_sha256_and_stable() {
        let artifact = ArtifactRecord::from_bytes(
            "artifact-1",
            "source-1",
            "text/plain",
            b"same payload",
            "2026-08-19T00:00:00Z",
        )
        .unwrap();
        assert_eq!(artifact.content_sha256.len(), 64);
        assert_eq!(
            artifact.content_sha256,
            ArtifactRecord::from_bytes(
                "artifact-1",
                "source-1",
                "text/plain",
                b"same payload",
                "2026-08-19T00:00:00Z",
            )
            .unwrap()
            .content_sha256
        );
    }

    #[test]
    fn audit_log_detects_tampering_and_preserves_append_only_chain() {
        let mut log = AuditLog::default();
        log.append(
            "ingest-1",
            "ingest",
            "connector",
            "artifact-1",
            b"payload",
            "2026-08-19T00:00:00Z",
        )
        .unwrap();
        log.append(
            "review-1",
            "review",
            "reviewer",
            "claim-1",
            b"accepted as signal",
            "2026-08-19T00:01:00Z",
        )
        .unwrap();
        assert!(log.verify_chain().is_ok());
        assert_eq!(log.events.len(), 2);
        assert_eq!(
            log.events[1].previous_event_sha256,
            Some(log.events[0].event_sha256.clone())
        );
        log.events[0].payload_sha256 = "tampered".into();
        assert!(log.verify_chain().is_err());
    }

    fn test_source(id: &str, group: &str) -> SourceRecord {
        SourceRecord {
            source_id: id.into(),
            source_kind: Some(SourceKind::Sensor),
            authority_tier: 2,
            publisher: "test".into(),
            independence_group: group.into(),
            acquired_at: "2026-08-19T00:00:00Z".into(),
            content_sha256: format!("hash-{id}"),
            ..SourceRecord::default()
        }
    }

    fn test_claim_from(source_id: &str) -> ClaimRecord {
        ClaimRecord {
            claim_id: format!("claim-{source_id}"),
            artifact_id: format!("artifact-{source_id}"),
            source_id: source_id.into(),
            claim_type: ClaimType::Observation,
            subject_id: "river-1".into(),
            predicate: "pollution".into(),
            object_text: "high".into(),
            event_type: "pollution".into(),
            extraction_method: "manual".into(),
            confidence: Some(0.95),
            status: EvidenceStatus::ScreeningOnly,
            ..ClaimRecord::default()
        }
    }

    fn test_artifacts(claims: &[ClaimRecord]) -> Vec<ArtifactRecord> {
        claims
            .iter()
            .map(|claim| ArtifactRecord {
                artifact_id: claim.artifact_id.clone(),
                source_id: claim.source_id.clone(),
                content_sha256: format!("artifact-{}", claim.source_id),
                acquired_at: "2026-08-19T00:00:00Z".into(),
                ..ArtifactRecord::default()
            })
            .collect()
    }

    fn test_sources_ab() -> Vec<SourceRecord> {
        vec![test_source("a", "lineage-a"), test_source("b", "lineage-b")]
    }

    #[test]
    fn source_round_trips_with_explicit_provenance_fields() {
        let source = SourceRecord::test_official("klhk-1", "hash-a", "2026-08-19T00:00:00Z");
        let json = serde_json::to_string(&source).unwrap();
        let decoded: SourceRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.source_id, "klhk-1");
        assert_eq!(decoded.content_sha256, "hash-a");
    }

    #[test]
    fn source_without_hash_or_acquisition_time_is_rejected() {
        let source = SourceRecord {
            content_sha256: String::new(),
            acquired_at: String::new(),
            ..SourceRecord::default()
        };
        assert!(source.validate().is_err());
    }

    #[test]
    fn normalization_collapses_case_and_whitespace() {
        assert_eq!(normalize_token("  River   Code  "), "river code");
    }

    #[test]
    fn claims_with_same_semantics_have_same_key() {
        let mut a = test_claim_from("a");
        let mut b = test_claim_from("b");
        a.subject_id = " River-1 ".into();
        b.subject_id = "river-1".into();
        a.object_text = " HIGH ".into();
        b.object_text = "high".into();
        assert_eq!(claim_key(&a), claim_key(&b));
    }

    #[test]
    fn duplicate_reporting_lineage_does_not_corroborate() {
        let sources = vec![
            test_source("a", "same-lineage"),
            test_source("b", "same-lineage"),
        ];
        let claims = vec![test_claim_from("a"), test_claim_from("b")];
        let result = assess_claims(&sources, &test_artifacts(&claims), &claims, 0.8).unwrap();
        assert_eq!(result.status, EvidenceStatus::InsufficientData);
    }

    #[test]
    fn independent_matching_claims_are_corroborated() {
        let claims = vec![test_claim_from("a"), test_claim_from("b")];
        let result =
            assess_claims(&test_sources_ab(), &test_artifacts(&claims), &claims, 0.8).unwrap();
        assert_eq!(result.status, EvidenceStatus::Corroborated);
        assert_eq!(result.independent_source_count, 2);
    }

    #[test]
    fn independent_conflicting_claims_abstain_for_review() {
        let mut claims = vec![test_claim_from("a"), test_claim_from("b")];
        claims[1].object_text = "low".into();
        let result =
            assess_claims(&test_sources_ab(), &test_artifacts(&claims), &claims, 0.8).unwrap();
        assert_eq!(result.status, EvidenceStatus::Contradictory);
        assert_eq!(result.state, IncidentState::HumanReview);
    }

    #[test]
    fn same_lineage_conflicting_claims_do_not_count_as_conflict() {
        let sources = vec![
            test_source("a", "same-lineage"),
            test_source("b", "same-lineage"),
        ];
        let mut claims = vec![test_claim_from("a"), test_claim_from("b")];
        claims[1].object_text = "low".into();
        let result = assess_claims(&sources, &test_artifacts(&claims), &claims, 0.8).unwrap();

        assert_eq!(result.conflict_count, 0);
        assert_eq!(result.status, EvidenceStatus::InsufficientData);
    }

    #[test]
    fn official_finding_without_confidence_is_sufficient_alone() {
        let mut source = test_source("official", "official-lineage");
        source.source_kind = Some(SourceKind::Official);
        source.authority_tier = 1;
        let mut claim = test_claim_from("official");
        claim.claim_type = ClaimType::OfficialFinding;
        claim.confidence = None;

        let result = assess_claims(
            &[source],
            &test_artifacts(&[claim.clone()]),
            &[claim],
            0.8,
        )
        .unwrap();

        assert_eq!(result.status, EvidenceStatus::Corroborated);
        assert_eq!(result.independent_source_count, 1);
        assert_eq!(result.confidence, Some(1.0));
    }

    #[test]
    fn corroboration_confidence_uses_lowest_strongest_lineage() {
        let mut claims = vec![test_claim_from("a"), test_claim_from("b")];
        claims[0].confidence = Some(0.95);
        claims[1].confidence = Some(0.85);

        let result = assess_claims(&test_sources_ab(), &test_artifacts(&claims), &claims, 0.8).unwrap();

        assert_eq!(result.status, EvidenceStatus::Corroborated);
        assert_eq!(result.confidence, Some(0.85));
    }

    #[test]
    fn screening_output_never_makes_a_legal_claim() {
        let claims = vec![test_claim_from("a")];
        let assessment = assess_claims(
            &[test_source("a", "lineage-a")],
            &test_artifacts(&claims),
            &claims,
            0.8,
        )
        .unwrap();
        let output: ScientificResult = serde_json::from_str(&emit_assessment(&assessment)).unwrap();
        assert_eq!(output.status, ResultStatus::ScreeningOnly);
        assert!(!output
            .claims
            .iter()
            .any(|claim| claim.claim_type == "legal"));
    }

    #[test]
    fn emitted_enum_labels_use_serde_snake_case() {
        let mut claims = vec![test_claim_from("a"), test_claim_from("b")];
        claims[1].object_text = "low".into();
        let assessment =
            assess_claims(&test_sources_ab(), &test_artifacts(&claims), &claims, 0.8).unwrap();
        assert_eq!(assessment.state, IncidentState::HumanReview);

        let output: ScientificResult = serde_json::from_str(&emit_assessment(&assessment)).unwrap();
        let state = output
            .claims
            .iter()
            .find(|claim| claim.claim_type == "incident_state")
            .expect("incident_state claim");
        // Debug formatting would produce "humanreview", which does not match the
        // serde wire format that consumers deserialize against.
        assert_eq!(state.description, "human_review");
    }
}
