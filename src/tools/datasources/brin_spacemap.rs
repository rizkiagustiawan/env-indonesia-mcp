/// BRIN Coral Data — STUB (not implemented, dead code, not wired to server.rs)
/// NOTE: Returns hardcoded index=1.0. BRIN Spacemap has no public REST API.
/// This is dead code — not exposed as an MCP tool.

use crate::result_contract::{ScientificResult, ResultStatus, Claim};

pub fn fetch_brin_coral_data(_lat: f64, _lon: f64) -> Result<ScientificResult, String> {
    // STUB: returns placeholder index=1.0. No real API call.
    let res = ScientificResult::new("brin_coral_health_index", 1.0, "index")
        .with_status(ResultStatus::ValidWithAssumptions)
        .with_claim(Claim::new(
            "data_source",
            "STUB: BRIN Spacemap has no public REST API. Returns placeholder index=1.0."
        ));

    Ok(res)
}
