#![allow(dead_code)]

use anyhow::Result;
use rmcp::ServiceExt;

mod indonesia;
mod result_contract;
mod server;
mod tools;

#[cfg(test)]
mod result_contract_tests {
    use super::result_contract::*;

    fn valid_result() -> ScientificResult {
        ScientificResult::new("water_quality", 12.0, "mg/L")
            .with_status(ResultStatus::ValidWithAssumptions)
            .with_uncertainty(Uncertainty::bound(10.0, 14.0, "measurement_range"))
            .with_provenance(Provenance::new("api", "https://example.test/data", "2026-08-02T00:00:00Z"))
    }

    #[test]
    fn accepts_a_complete_finite_result() {
        assert!(valid_result().validate().is_ok());
    }

    #[test]
    fn rejects_non_finite_values() {
        let result = ScientificResult::new("water_quality", f64::NAN, "mg/L");
        assert!(result.validate().unwrap_err().contains("finite"));
    }

    #[test]
    fn rejects_reversed_uncertainty_bounds() {
        let result = valid_result().with_uncertainty(Uncertainty::bound(14.0, 10.0, "range"));
        assert!(result.validate().unwrap_err().contains("lower"));
    }

    #[test]
    fn requires_seed_for_stochastic_uncertainty() {
        let result = valid_result().with_uncertainty(Uncertainty::confidence_interval(10.0, 14.0, 0.95));
        assert!(result.validate().unwrap_err().contains("seed"));
    }

    #[test]
    fn requires_a_reason_for_fallback_sources() {
        let result = valid_result().with_provenance(
            Provenance::new("fallback", "mirror", "2026-08-02T00:00:00Z"),
        );
        assert!(result.validate().unwrap_err().contains("fallback reason"));
    }

    #[test]
    fn rejects_stale_sources() {
        let result = valid_result().with_provenance(
            Provenance::new("api", "https://example.test/data", "2020-01-01T00:00:00Z")
                .with_max_age_days(30),
        );
        assert!(result.validate().unwrap_err().contains("stale"));
    }

    #[test]
    fn rejects_regulatory_claims_from_screening_results() {
        let result = valid_result()
            .with_status(ResultStatus::ScreeningOnly)
            .with_claim(Claim::new("compliant", "screening output"));
        assert!(result.validate().unwrap_err().contains("screening"));
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("env-indonesia-mcp v1.0.0 — Environmental AI MCP Server for Indonesia");

    let server = server::EnvIndonesiaServer::new();
    let service = server.serve(rmcp::transport::io::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
