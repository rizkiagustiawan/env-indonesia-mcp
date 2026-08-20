#![allow(dead_code)]

use anyhow::Result;
use rmcp::ServiceExt;

pub mod amdal_pipeline;
mod indonesia;
mod result_contract;
mod server;
mod tools;
mod validation;
pub mod artifacts;
pub mod calibration;
pub mod phreeqc_runner;
pub mod modflow_runner;
pub mod pyrite_kinetics;
pub mod reactive_transport;
pub mod computation;
pub mod coupling;
pub mod evidence;
pub mod honesty;
pub mod swmm_runner;

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
    fn synthetic_result_cannot_be_valid() {
        let result = valid_result()
            .with_status(ResultStatus::Valid)
            .with_synthetic(true);
        assert!(result.validate().unwrap_err().contains("Synthetic"));
    }

    #[test]
    fn synthetic_with_assumptions_is_allowed() {
        let result = valid_result().with_synthetic(true);
        assert!(result.validate().is_ok());
    }

    #[test]
    fn rejects_regulatory_claims_from_screening_results() {
        let result = valid_result()
            .with_status(ResultStatus::ScreeningOnly)
            .with_claim(Claim::new("compliant", "screening output"));
        assert!(result.validate().unwrap_err().contains("screening"));
    }

    #[test]
    fn validates_confidence_provenance_crs_and_lineage_fields() {
        let result = valid_result()
            .with_confidence(0.8)
            .with_crs(CrsReference::epsg(4326))
            .with_artifact_lineage(ArtifactLineage::new(
                "artifact-1",
                "https://example.test/data.tif",
                4,
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "2026-08-02T00:00:00Z",
            ));
        assert!(result.validate().is_ok());
    }

    #[test]
    fn rejects_invalid_evidence_metadata() {
        let invalid_confidence = valid_result().with_confidence(1.1);
        assert!(invalid_confidence.validate().unwrap_err().contains("Confidence"));

        let invalid_crs = valid_result().with_crs(CrsReference {
            code: "not-a-crs".to_string(),
            name: None,
        });
        assert!(invalid_crs.validate().unwrap_err().contains("CRS"));

        let invalid_lineage = valid_result().with_artifact_lineage(ArtifactLineage::new(
            "artifact-1",
            "https://example.test/data.tif",
            0,
            "not-a-sha256",
            "not-rfc3339",
        ));
        assert!(invalid_lineage.validate().is_err());
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
            eprintln!("Executing tool: {}", tool_name);
            // This is a minimal dispatch for the most critical calculators
            // (a full 229-tool dispatch would use the rmcp framework's router)
            match tool_name.as_str() {
                "rusle_erosion" => {
                    let p: server::RusleParam = serde_json::from_str(tool_args)?;
                    let res = tools::calculators::rusle::calculate(p.r_input, p.rain_mm_yr, p.k, p.ls, p.c, p.p);
                    println!("{}", res);
                },
                "peatland_subsidence" => {
                    let p: tools::advanced_physics::peatland_subsidence::PeatlandSubsidenceParam = serde_json::from_str(tool_args)?;
                    let res = tools::advanced_physics::peatland_subsidence::calculate_peatland_subsidence(&p);
                    println!("{}", res);
                },
                "hpal_tailings" => {
                    let p: tools::waste::hpal_tailings::HpalTailingsParam = serde_json::from_str(tool_args)?;
                    let res = tools::waste::hpal_tailings::evaluate_hpal_tailings(&p);
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
                "integrated_environment_study" => {
                    let p: tools::integrated_study::IntegratedStudyRequest = serde_json::from_str(tool_args)?;
                    let plan = tools::integrated_study::plan_study(&p)
                        .map_err(|error| anyhow::anyhow!(error))?;
                    let mut report = tools::integrated_study::run_baselines(&p, &plan);
                    if p.satellite_fallback {
                        let client = reqwest::Client::builder()
                            .timeout(std::time::Duration::from_secs(30))
                            .user_agent("env-indonesia-mcp/1.0.0")
                            .build()?;
                        report.satellite_discovery = Some(
                            tools::integrated_study::discover_satellite_sources(&client, &plan).await,
                        );
                    }
                    println!("{}", serde_json::to_string_pretty(&report)?);
                },
                "assess_data_maturity" => {
                    let p: server::MaturityParam = serde_json::from_str(tool_args)?;
                    let decision = honesty::gate(honesty::parse_level(&p.requested_level), &p.availability);
                    println!("{}", serde_json::to_string_pretty(&decision)?);
                },
                "record_computation" => {
                    let p: server::ComputationParam = serde_json::from_str(tool_args)?;
                    println!("{}", computation::record_json(&p.record));
                },
                "evidence_assess" => {
                    let p: evidence::EvidenceAssessmentRequest = serde_json::from_str(tool_args)?;
                    println!("{}", evidence::assess_request(&p));
                },
                "pyrite_oxidation_kinetics" => {
                    let p: pyrite_kinetics::PyriteKineticsRequest = serde_json::from_str(tool_args)?;
                    match pyrite_kinetics::run_pyrite_kinetics(&p).await {
                        Err(error) => println!("{}", serde_json::json!({"status":"validation_failed","error":error})),
                        Ok(run) => println!("{}", serde_json::to_string_pretty(&run)?),
                    }
                },
                "reactive_transport" => {
                    let p: reactive_transport::ReactiveTransportRequest = serde_json::from_str(tool_args)?;
                    match reactive_transport::run_reactive_transport(&p).await {
                        Err(error) => println!("{}", serde_json::json!({"status":"validation_failed","error":error})),
                        Ok(run) => println!("{}", serde_json::to_string_pretty(&run)?),
                    }
                },
                "modflow_groundwater" => {
                    let p: modflow_runner::ModflowRequest = serde_json::from_str(tool_args)?;
                    match modflow_runner::run_modflow(&p).await {
                        Err(error) => println!("{}", serde_json::json!({"status":"validation_failed","error":error})),
                        Ok(run) => println!("{}", serde_json::to_string_pretty(&run)?),
                    }
                },
                "phreeqc_speciation" => {
                    let p: phreeqc_runner::PhreeqcRequest = serde_json::from_str(tool_args)?;
                    match phreeqc_runner::run_phreeqc(&p).await {
                        Err(error) => println!("{}", serde_json::json!({"status":"validation_failed","error":error})),
                        Ok(run) => println!("{}", serde_json::to_string_pretty(&run)?),
                    }
                },
                "calibrate_and_validate" => {
                    let p: server::CalibrateValidateParam = serde_json::from_str(tool_args)?;
                    let train_fraction = p.train_fraction.unwrap_or(0.7);
                    let confidence_level = p.confidence_level.unwrap_or(0.95);
                    match calibration::validate_split_sample(&p.predicted, &p.observed, train_fraction, confidence_level) {
                        Err(error) => println!("{}", serde_json::json!({"status":"validation_failed","error":error})),
                        Ok(evidence) => {
                            let earned = calibration::earned_level(&evidence);
                            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                                "earned_level": format!("{:?}", earned).to_lowercase(),
                                "evidence": evidence
                            }))?);
                        }
                    }
                },
                "swmm_1d2d_coupling" => {
                    let p: server::SwmmCouplingParam = serde_json::from_str(tool_args)?;
                    let ny = p.dem.len();
                    let nx = p.dem.first().map_or(0, Vec::len);
                    if nx < 3 || ny < 3 || p.dem.iter().any(|row| row.len() != nx) {
                        println!("{}", coupling::coupling_failure("DEM must be a rectangular grid with at least 3x3 cells").emit_validated());
                    } else {
                        match swmm_runner::run_swmm(&p.inp_path, p.timeout_secs.unwrap_or(120)).await {
                            Err(error) => println!("{}", coupling::coupling_failure(&error).emit_validated()),
                            Ok(run) => match coupling::build_sources(&run, &p.node_mapping, p.duration_s) {
                                Err(error) => println!("{}", coupling::coupling_failure(&error).emit_validated()),
                                Ok(sources) => {
                                    let params = tools::advanced_physics::swe_solver::SweParams {
                                        nx, ny, dx: p.dx_m, manning_n: p.manning_n,
                                        duration_s: p.duration_s, dt_max: p.dt_max_s, second_order: false,
                                    };
                                    let swe = tools::advanced_physics::swe_solver::solve_multi_source(&p.dem, &params, &sources, 1.0);
                                    let tolerance = p.mass_tolerance_pct.unwrap_or(coupling::DEFAULT_MASS_TOLERANCE_PCT);
                                    let gate = coupling::check_mass_balance(run.routing.flooding_m3, swe.total_volume_m3, tolerance);
                                    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                                        "gate": gate,
                                        "swmm_routing": run.routing,
                                        "swmm_nodes": run.nodes,
                                        "swe": {
                                            "max_depth_m": swe.max_depth,
                                            "flooded_cells": swe.flooded_cells,
                                            "total_cells": swe.total_cells,
                                            "flooded_area_m2": swe.flooded_area_m2,
                                            "total_volume_m3": swe.total_volume_m3
                                        },
                                        "contract": serde_json::from_str::<serde_json::Value>(&coupling::coupling_result(&gate, swe.max_depth, swe.flooded_cells).emit_validated()).unwrap_or_default()
                                    }))?);
                                }
                            }
                        }
                    }
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
                "stac_download_asset" => {
                    let p: server::StacDownloadParam = serde_json::from_str(tool_args)?;
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(30))
                        .user_agent("env-indonesia-mcp/1.0.0")
                        .build()?;
                    let res = tools::satellite::stac::download_asset(
                        &client,
                        p.api.as_deref().unwrap_or("mpc"),
                        &p.collection,
                        &p.item_id,
                        &p.asset_key,
                        &p.output_dir,
                    ).await;
                    match res {
                        Ok(json) => println!("{}", json),
                        Err(e) => println!("{}", e)
                    }
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
