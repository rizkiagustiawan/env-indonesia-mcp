#![allow(dead_code)]

use anyhow::Result;
use rmcp::ServiceExt;

pub mod amdal_pipeline;
mod indonesia;
mod result_contract;
mod server;
mod tools;
mod validation;

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

    // Check for --test-tool CLI argument
    let args: Vec<String> = std::env::args().collect();
    if let Some(idx) = args.iter().position(|a| a == "--test-tool") {
        if idx + 2 < args.len() {
            let tool_name = &args[idx + 1];
            let tool_args = &args[idx + 2];
            println!("Executing tool: {}", tool_name);
            // This is a minimal dispatch for the most critical calculators
            // (a full 229-tool dispatch would use the rmcp framework's router)
            match tool_name.as_str() {
                "rusle_erosion" => {
                    let p: server::RusleParam = serde_json::from_str(tool_args)?;
                    let res = tools::calculators::rusle::calculate(p.r_input, p.rain_mm_yr, p.k, p.ls, p.c, p.p);
                    println!("{}", res);
                },
                "aermod_generator" => {
                    let p: server::AermodGeneratorParam = serde_json::from_str(tool_args)?;
                    let res = tools::airquality::aermod_generator::generate_aermod_inp(
                        &p.project_name, p.source_lat, p.source_lon, p.stack_height_m,
                        p.stack_diameter_m, p.exit_velocity_m_s, p.exit_temp_k,
                        p.emission_rate_g_s, &p.pollutant_id, p.is_rural
                    );
                    println!("{}", res);
                },
                "phreeqc_leaching" => {
                    let p: server::PhreeqcLeachingParam = serde_json::from_str(tool_args)?;
                    let res = tools::waste::phreeqc_leaching::generate_phreeqc_script(
                        &p.waste_type, p.solid_mass_g, p.water_volume_l, p.target_ph, &p.initial_metals_mg_kg
                    );
                    println!("{}", res);
                },
                "gaussian_plume" => {
                    let p: server::GaussianParam = serde_json::from_str(tool_args)?;
                    let res = tools::calculators::gaussian_plume::calculate(p.emission_gs, p.wind_ms, p.stack_height_m, p.distance_m, &p.stability_class);
                    println!("{}", res);
                },
                "penman_monteith_et0" => {
                    let p: server::PenmanParam = serde_json::from_str(tool_args)?;
                    let res = tools::calculators::penman_monteith::calculate(p.t_mean_c, p.rh_pct, p.wind_ms, p.rn_mj);
                    println!("{}", res);
                },
                "land_subsidence" => {
                    let p: server::SubsidenceParam = serde_json::from_str(tool_args)?;
                    let res = tools::calculators::land_subsidence::calculate(p.clay_thickness_m, p.delta_stress_kpa, p.cc, p.e0, p.sigma0_kpa);
                    println!("{}", res);
                },
                "stac_search" => {
                    let p: server::StacSearchParam = serde_json::from_str(tool_args)?;
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .user_agent("env-indonesia-mcp/1.0.0")
                        .build()?;
                    let res = tools::satellite::stac::search(
                        &client,
                        p.api.as_deref().unwrap_or("mpc"),
                        &p.collection,
                        &p.bbox,
                        &p.datetime,
                        p.limit.unwrap_or(10).min(100),
                    ).await;
                    println!("{}", res);
                },
                "stac_collections" => {
                    let p: server::StacListParam = serde_json::from_str(tool_args)?;
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .user_agent("env-indonesia-mcp/1.0.0")
                        .build()?;
                    let res = tools::satellite::stac::list_collections(
                        &client,
                        p.api.as_deref().unwrap_or("mpc"),
                    ).await;
                    println!("{}", res);
                },
                "stac_describe" => {
                    let p: server::StacCollectionParam = serde_json::from_str(tool_args)?;
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .user_agent("env-indonesia-mcp/1.0.0")
                        .build()?;
                    let res = tools::satellite::stac::describe_collection(
                        &client,
                        p.api.as_deref().unwrap_or("mpc"),
                        &p.collection,
                    ).await;
                    println!("{}", res);
                },
                "stac_asset_url" => {
                    let p: server::StacAssetParam = serde_json::from_str(tool_args)?;
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .user_agent("env-indonesia-mcp/1.0.0")
                        .build()?;
                    let res = tools::satellite::stac::get_asset_url(
                        &client,
                        p.api.as_deref().unwrap_or("mpc"),
                        &p.collection,
                        &p.item_id,
                        &p.asset_key,
                    ).await;
                    println!("{}", res);
                },
                "flood_sar_mapping" => {
                    let p: server::FloodSarParam = serde_json::from_str(tool_args)?;
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .user_agent("env-indonesia-mcp/1.0.0")
                        .build()?;
                    let res = tools::satellite::flood_sar::search_flood_scenes(
                        &client, p.lat, p.lon, p.buffer_km.unwrap_or(10.0), &p.flood_date
                    ).await;
                    println!("{}", res);
                },
                "karhutla_assessment" => {
                    let p: server::KarhutlaParam = serde_json::from_str(tool_args)?;
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .user_agent("env-indonesia-mcp/1.0.0")
                        .build()?;
                    let res = tools::satellite::karhutla::assess_karhutla(
                        &client, p.lat, p.lon, p.buffer_km.unwrap_or(10.0), &p.fire_date
                    ).await;
                    println!("{}", res);
                },
                "coral_dhw_alert" => {
                    let p: server::CoralAlertParam = serde_json::from_str(tool_args)?;
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .user_agent("env-indonesia-mcp/1.0.0")
                        .build()?;
                    let res = tools::ocean::coral_dhw::query_dhw(&client, p.lat, p.lon).await;
                    println!("{}", res);
                },
                "climate_projection" => {
                    let p: server::ClimateProjParam = serde_json::from_str(tool_args)?;
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .user_agent("env-indonesia-mcp/1.0.0")
                        .build()?;
                    let res = tools::data::climate_projection::search_climate_projection(
                        &client, p.lat, p.lon,
                        p.scenario.as_deref().unwrap_or("ssp585"),
                        p.period.as_deref().unwrap_or("2050")
                    ).await;
                    println!("{}", res);
                },
                _ => println!("Tool '{}' not yet wired in CLI test mode. Use MCP stdio.", tool_name),
            }
            return Ok(());
        }
    }

    // Check for --pipeline flag (AMDAL 20-map parallel generation)
    if args.iter().any(|a| a == "--pipeline") {
        let mut lat = -2.5;
        let mut lon = 118.0;
        let mut buffer_km = 5.0;
        for i in 0..args.len() {
            if args[i] == "--lat" && i + 1 < args.len() {
                lat = args[i + 1].parse().unwrap_or(lat);
            } else if args[i] == "--lon" && i + 1 < args.len() {
                lon = args[i + 1].parse().unwrap_or(lon);
            } else if args[i] == "--buffer" && i + 1 < args.len() {
                buffer_km = args[i + 1].parse().unwrap_or(buffer_km);
            }
        }
        let params = amdal_pipeline::PipelineParams {
            lat,
            lon,
            buffer_km,
            start_date: "2024-01-01".into(),
            end_date: "2024-12-31".into(),
        };
        let report = amdal_pipeline::run_pipeline(&params);
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    tracing::info!("env-indonesia-mcp v1.0.0 — Environmental AI MCP Server for Indonesia");

    let server = server::EnvIndonesiaServer::new();
    let service = server.serve(rmcp::transport::io::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
