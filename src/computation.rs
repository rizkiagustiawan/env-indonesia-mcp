use crate::evidence::{AuditEvent, AuditLog};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComputationRecord {
    pub run_id: String,
    pub software: String,
    pub software_version: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub input_sha256s: Vec<String>,
    pub output_sha256s: Vec<String>,
    pub exit_code: i32,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ComputationLog {
    pub records: Vec<ComputationRecord>,
}

impl ComputationRecord {
    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("run_id", &self.run_id),
            ("software", &self.software),
            ("software_version", &self.software_version),
            ("tool_name", &self.tool_name),
        ] {
            if value.trim().is_empty() {
                return Err(format!("Computation {} must not be empty", name));
            }
        }
        for (name, hashes) in [("input_sha256s", &self.input_sha256s), ("output_sha256s", &self.output_sha256s)] {
            for hash in hashes {
                if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(format!("Computation {} must contain 64-char lowercase hex sha256 values", name));
                }
            }
        }
        let started = chrono::DateTime::parse_from_rfc3339(&self.started_at)
            .map_err(|_| "Computation started_at must be RFC3339".to_string())?;
        let finished = chrono::DateTime::parse_from_rfc3339(&self.finished_at)
            .map_err(|_| "Computation finished_at must be RFC3339".to_string())?;
        if finished < started {
            return Err("Computation finished_at must not precede started_at".to_string());
        }
        Ok(())
    }
}

pub fn record_computation(
    log: &mut ComputationLog,
    audit: &mut AuditLog,
    record: &ComputationRecord,
) -> Result<AuditEvent, String> {
    record.validate()?;
    let payload = serde_json::to_vec(record).map_err(|e| format!("serialization error: {e}"))?;
    audit.append(&record.run_id, "computation", &record.software, &record.tool_name, &payload, &record.finished_at)?;
    log.records.push(record.clone());
    audit.events.last().cloned().ok_or_else(|| "audit log empty after append".to_string())
}

pub fn record_json(record: &ComputationRecord) -> String {
    let mut log = ComputationLog::default();
    let mut audit = AuditLog::default();
    match record_computation(&mut log, &mut audit, record) {
        Ok(event) => serde_json::to_string(&event).unwrap_or_default(),
        Err(error) => serde_json::json!({ "status": "validation_failed", "error": error }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_record() -> ComputationRecord {
        ComputationRecord {
            run_id: "run-1".into(),
            software: "qgis".into(),
            software_version: "3.44".into(),
            tool_name: "gdal:warpreproject".into(),
            arguments: serde_json::json!({"crs": "EPSG:32748"}),
            input_sha256s: vec!["a".repeat(64)],
            output_sha256s: vec!["b".repeat(64)],
            exit_code: 0,
            started_at: "2026-08-20T00:00:00Z".into(),
            finished_at: "2026-08-20T00:01:00Z".into(),
        }
    }

    #[test]
    fn valid_record_produces_audit_event_with_chain_hash() {
        let mut log = ComputationLog::default();
        let mut audit = AuditLog::default();
        let event = record_computation(&mut log, &mut audit, &valid_record()).unwrap();
        assert_eq!(event.event_sha256.len(), 64);
        assert_eq!(event.payload_sha256.len(), 64);
        assert_eq!(log.records.len(), 1);
    }

    #[test]
    fn empty_identity_field_is_rejected() {
        let mut rec = valid_record();
        rec.run_id = "  ".into();
        assert!(rec.validate().unwrap_err().contains("run_id"));
    }

    #[test]
    fn non_hex_sha256_is_rejected() {
        let mut rec = valid_record();
        rec.input_sha256s = vec!["not-hex".into()];
        assert!(rec.validate().unwrap_err().contains("sha256"));
    }

    #[test]
    fn non_rfc3339_timestamp_is_rejected() {
        let mut rec = valid_record();
        rec.started_at = "yesterday".into();
        assert!(rec.validate().unwrap_err().contains("RFC3339"));
    }

    #[test]
    fn finished_before_started_is_rejected() {
        let mut rec = valid_record();
        rec.finished_at = "2026-08-19T00:00:00Z".into();
        assert!(rec.validate().unwrap_err().contains("finished_at"));
    }

    #[test]
    fn two_computations_form_a_linked_chain() {
        let mut log = ComputationLog::default();
        let mut audit = AuditLog::default();
        record_computation(&mut log, &mut audit, &valid_record()).unwrap();
        let mut rec2 = valid_record();
        rec2.run_id = "run-2".into();
        record_computation(&mut log, &mut audit, &rec2).unwrap();
        assert_eq!(audit.events[1].previous_event_sha256, Some(audit.events[0].event_sha256.clone()));
        assert!(audit.verify_chain().is_ok());
    }
}
