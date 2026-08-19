#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_polygon_produces_wgs84_bbox_and_all_domain_plan() {
        let request = IntegratedStudyRequest {
            aoi_geojson: r#"{"type":"Polygon","coordinates":[[[101.0,0.0],[101.2,0.0],[101.2,0.2],[101.0,0.2],[101.0,0.0]]]}"#.into(),
            domains: None,
            satellite_fallback: false,
            flood: None,
            leachate: None,
            amd: None,
        };

        let plan = plan_study(&request).expect("valid AOI should plan");
        assert_eq!(plan.bbox, [101.0, 0.0, 101.2, 0.2]);
        assert_eq!(plan.domains.len(), 3);
        assert!(plan.data_gaps.iter().any(|gap| gap.contains("flood")));
    }

    #[test]
    fn malformed_geojson_is_rejected() {
        let request = IntegratedStudyRequest {
            aoi_geojson: "not geojson".into(),
            domains: Some(vec!["urban_flood".into()]),
            satellite_fallback: false,
            flood: None,
            leachate: None,
            amd: None,
        };

        let error = plan_study(&request).expect_err("malformed AOI must fail");
        assert!(error.contains("GeoJSON"));
    }

    #[test]
    fn empty_domain_list_is_rejected() {
        let request = IntegratedStudyRequest {
            aoi_geojson: r#"{"type":"Point","coordinates":[101.0,0.0]}"#.into(),
            domains: Some(Vec::new()),
            satellite_fallback: false,
            flood: None,
            leachate: None,
            amd: None,
        };

        let error = plan_study(&request).expect_err("empty domain list must fail");
        assert!(error.contains("domain"));
    }

    #[test]
    fn point_aoi_gets_non_degenerate_satellite_search_bbox() {
        let request = IntegratedStudyRequest {
            aoi_geojson: r#"{"type":"Point","coordinates":[101.0,0.0]}"#.into(),
            domains: Some(vec!["flood".into()]),
            satellite_fallback: true,
            flood: None,
            leachate: None,
            amd: None,
        };

        let plan = plan_study(&request).expect("valid point should plan");
        let bbox = satellite_search_bbox(plan.bbox);
        assert!(bbox[0] < bbox[2]);
        assert!(bbox[1] < bbox[3]);
    }

    #[test]
    fn area_aoi_preserves_requested_satellite_search_extent() {
        let bbox = satellite_search_bbox([100.0, -1.0, 100.5, -0.5]);
        assert_eq!(bbox, [100.0, -1.0, 100.5, -0.5]);
    }

    #[test]
    fn missing_observations_never_produce_validated_status() {
        let request = IntegratedStudyRequest {
            aoi_geojson: r#"{"type":"Point","coordinates":[101.0,0.0]}"#.into(),
            domains: Some(vec!["acid_mine_drainage".into()]),
            satellite_fallback: true,
            flood: None,
            leachate: None,
            amd: None,
        };

        let plan = plan_study(&request).expect("valid point should plan");
        let report = run_baselines(&request, &plan);
        assert_ne!(report.status, "validated_operational");
        assert!(report.domain_results[0].status == "insufficient_data");
    }

    #[test]
    fn supplied_leachate_and_amd_inputs_run_screening_baselines() {
        let request = IntegratedStudyRequest {
            aoi_geojson: r#"{"type":"Point","coordinates":[101.0,0.0]}"#.into(),
            domains: Some(vec!["lindi".into(), "amd".into()]),
            satellite_fallback: false,
            flood: None,
            leachate: Some(LeachateBaselineInput {
                area_m2: 10_000.0,
                monthly_rainfall_mm: vec![200.0; 12],
                monthly_et_mm: vec![100.0; 12],
                soil_storage_mm: 50.0,
                runoff_coeff: 0.2,
            }),
            amd: Some(AmdBaselineInput {
                sulfur_pct: 2.0,
                anc_kg_h2so4_t: 10.0,
                nag_ph: Some(3.0),
            }),
        };

        let plan = plan_study(&request).expect("valid study should plan");
        let report = run_baselines(&request, &plan);
        assert_eq!(report.status, "screening_only");
        assert_eq!(report.domain_results.len(), 2);
        assert!(report.domain_results.iter().all(|result| result.output.is_none()));
        assert!(report.domain_results.iter().all(|result| result.status == "screening_only"));
        let leachate_summary = &report.domain_results[0].summary;
        assert_eq!(leachate_summary[0]["value"], 7200.0);
        let amd_summary = &report.domain_results[1].summary;
        assert_eq!(amd_summary[1]["value"], 61.2);
        assert_eq!(amd_summary[0]["value"], 51.2);
    }

    #[test]
    fn feature_collection_and_multipolygon_bbox_are_supported() {
        let feature_collection = IntegratedStudyRequest {
            aoi_geojson: r#"{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[100.0,0.0]},"properties":{}},{"type":"Feature","geometry":{"type":"MultiPolygon","coordinates":[[[[101.0,1.0],[101.2,1.0],[101.2,1.2],[101.0,1.0]]]]},"properties":{}}]}"#.into(),
            domains: Some(vec!["flood".into()]),
            satellite_fallback: false,
            flood: None,
            leachate: None,
            amd: None,
        };
        let plan = plan_study(&feature_collection).expect("FeatureCollection should plan");
        assert_eq!(plan.bbox, [100.0, 0.0, 101.2, 1.2]);
    }

    #[test]
    fn invalid_flood_grid_is_reported_as_insufficient_data() {
        let request = IntegratedStudyRequest {
            aoi_geojson: r#"{"type":"Point","coordinates":[101.0,0.0]}"#.into(),
            domains: Some(vec!["flood".into()]),
            satellite_fallback: false,
            flood: Some(FloodBaselineInput {
                dem: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
                dx_m: 10.0,
                manning_n: 0.03,
                duration_s: 60.0,
                dt_max_s: 1.0,
                second_order: false,
                inflow_discharge_m3s: 1.0,
                inflow_x: 0,
                inflow_y: 0,
                inflow_width: 1,
            }),
            leachate: None,
            amd: None,
        };

        let plan = plan_study(&request).expect("valid study should plan");
        let report = run_baselines(&request, &plan);
        assert_eq!(report.domain_results[0].status, "insufficient_data");
        assert!(report.domain_results[0].limitations[0].contains("rectangular"));
    }

    #[test]
    fn flood_baseline_produces_positive_depth_for_positive_inflow() {
        let request = IntegratedStudyRequest {
            aoi_geojson: r#"{"type":"Polygon","coordinates":[[[101.0,0.0],[101.05,0.0],[101.05,0.05],[101.0,0.05],[101.0,0.0]]]}"#.into(),
            domains: Some(vec!["flood".into()]),
            satellite_fallback: false,
            flood: Some(FloodBaselineInput {
                dem: vec![vec![10.0; 5]; 5],
                dx_m: 10.0,
                manning_n: 0.03,
                duration_s: 10.0,
                dt_max_s: 0.1,
                second_order: false,
                inflow_discharge_m3s: 10.0,
                inflow_x: 2,
                inflow_y: 2,
                inflow_width: 1,
            }),
            leachate: None,
            amd: None,
        };

        let plan = plan_study(&request).expect("valid flood study should plan");
        let report = run_baselines(&request, &plan);
        let summary = &report.domain_results[0].summary;
        assert!(summary["max_depth_m"].as_f64().unwrap() > 0.0);
        assert!(summary["flooded_cells"].as_u64().unwrap() > 0);
    }
}
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::tools::{advanced_physics::swe_solver, calculators::acid_mine_drainage, waste::leachate};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IntegratedStudyRequest {
    pub aoi_geojson: String,
    pub domains: Option<Vec<String>>,
    #[serde(default)]
    pub satellite_fallback: bool,
    pub flood: Option<FloodBaselineInput>,
    pub leachate: Option<LeachateBaselineInput>,
    pub amd: Option<AmdBaselineInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FloodBaselineInput {
    pub dem: Vec<Vec<f64>>,
    pub dx_m: f64,
    pub manning_n: f64,
    pub duration_s: f64,
    pub dt_max_s: f64,
    #[serde(default)]
    pub second_order: bool,
    pub inflow_discharge_m3s: f64,
    pub inflow_x: usize,
    pub inflow_y: usize,
    pub inflow_width: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LeachateBaselineInput {
    pub area_m2: f64,
    pub monthly_rainfall_mm: Vec<f64>,
    pub monthly_et_mm: Vec<f64>,
    pub soil_storage_mm: f64,
    pub runoff_coeff: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AmdBaselineInput {
    pub sulfur_pct: f64,
    pub anc_kg_h2so4_t: f64,
    pub nag_ph: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainPlan {
    pub domain: String,
    pub evidence_level: String,
    pub method: String,
    pub data_gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StudyPlan {
    pub bbox: [f64; 4],
    pub domains: Vec<DomainPlan>,
    pub data_gaps: Vec<String>,
    pub satellite_fallback: SatelliteFallbackPlan,
}

#[derive(Debug, Clone, Serialize)]
pub struct SatelliteFallbackPlan {
    pub enabled: bool,
    pub collections: Vec<String>,
    pub purpose: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainResult {
    pub domain: String,
    pub status: String,
    pub method: String,
    pub output: Option<String>,
    pub summary: serde_json::Value,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegratedStudyReport {
    pub status: String,
    pub study_plan: StudyPlan,
    pub domain_results: Vec<DomainResult>,
    pub validation: ValidationSummary,
    pub uncertainty: UncertaintySummary,
    pub satellite_discovery: Option<SatelliteDiscovery>,
    pub provenance: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationSummary {
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UncertaintySummary {
    pub status: String,
    pub method: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SatelliteDiscovery {
    pub sources: Vec<SatelliteSourceResult>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SatelliteSourceResult {
    pub collection: String,
    pub status: String,
    pub matched_scenes: usize,
    pub search_bbox: [f64; 4],
}

pub async fn discover_satellite_sources(
    client: &reqwest::Client,
    plan: &StudyPlan,
) -> SatelliteDiscovery {
    if !plan.satellite_fallback.enabled {
        return SatelliteDiscovery { sources: Vec::new(), warnings: vec!["Satellite fallback disabled by request".into()] };
    }

    let url = "https://earth-search.aws.element84.com/v1/search";
    let bbox = satellite_search_bbox(plan.bbox);
    let collections = ["sentinel-1-grd", "sentinel-2-l2a", "cop-dem-glo-30"];
    let mut sources = Vec::new();
    let mut warnings = vec!["Scene suitability still requires cloud/coverage/temporal checks".into()];
    for collection in collections {
        let body = serde_json::json!({"collections": [collection], "bbox": bbox, "limit": 3});
        match client.post(url).json(&body).send().await {
            Ok(response) if response.status().is_success() => match response.json::<serde_json::Value>().await {
                Ok(value) => {
                    let count = value.get("features").and_then(serde_json::Value::as_array).map_or(0, Vec::len);
                    sources.push(SatelliteSourceResult { collection: collection.into(), status: "matched".into(), matched_scenes: count, search_bbox: bbox });
                }
                Err(error) => warnings.push(format!("{} JSON parse failed: {}", collection, error)),
            },
            Ok(response) => warnings.push(format!("{} request returned HTTP {}", collection, response.status())),
            Err(error) => warnings.push(format!("{} request failed: {}", collection, error)),
        }
    }
    SatelliteDiscovery { sources, warnings }
}

fn satellite_search_bbox(bbox: [f64; 4]) -> [f64; 4] {
    if bbox[0] < bbox[2] && bbox[1] < bbox[3] {
        return bbox;
    }
    let epsilon = 0.01;
    let west = (bbox[0] - epsilon).max(-180.0);
    let south = (bbox[1] - epsilon).max(-90.0);
    let east = (bbox[2] + epsilon).min(180.0);
    let north = (bbox[3] + epsilon).min(90.0);
    [west, south, east, north]
}

pub fn plan_study(request: &IntegratedStudyRequest) -> Result<StudyPlan, String> {
    let bbox = geojson_bbox(&request.aoi_geojson)?;
    let requested = request
        .domains
        .clone()
        .unwrap_or_else(|| vec!["urban_flood".into(), "landfill_leachate".into(), "acid_mine_drainage".into()]);
    if requested.is_empty() {
        return Err("At least one study domain is required".into());
    }

    let mut domains = Vec::new();
    let mut data_gaps = Vec::new();
    for raw_domain in requested {
        let domain = normalize_domain(&raw_domain).ok_or_else(|| {
            format!("Unsupported domain '{}'. Use urban_flood, landfill_leachate, or acid_mine_drainage.", raw_domain)
        })?;
        let (evidence_level, method, gaps) = match domain.as_str() {
            "urban_flood" => (
                if request.flood.is_some() { "screening" } else { "insufficient_data" },
                "2D SWE baseline; urban sewer coupling is not available",
                if request.flood.is_some() { vec!["independent flood observations".into(), "rainfall-runoff and sewer network inputs".into()] } else { vec!["rectangular DEM and hydraulic forcing".into()] },
            ),
            "landfill_leachate" => (
                if request.leachate.is_some() { "screening" } else { "insufficient_data" },
                "monthly landfill water balance",
                if request.leachate.is_some() { vec!["leachate quality time series".into(), "layered liner and vadose-zone parameters".into()] } else { vec!["landfill area, rainfall, ET, storage, and runoff inputs".into()] },
            ),
            "acid_mine_drainage" => (
                if request.amd.is_some() { "screening" } else { "insufficient_data" },
                "ABA MPA/NAPP static screening",
                if request.amd.is_some() { vec!["kinetic test data".into(), "mineralogy and reactive transport parameters".into(), "pH/sulfate/metals observations".into()] } else { vec!["sulfur, ANC, and optional NAG pH inputs".into()] },
            ),
            _ => unreachable!(),
        };
        data_gaps.extend(gaps.iter().map(|gap| format!("{}: {}", domain, gap)));
        domains.push(DomainPlan { domain, evidence_level: evidence_level.into(), method: method.into(), data_gaps: gaps });
    }

    Ok(StudyPlan {
        bbox,
        domains,
        data_gaps,
        satellite_fallback: SatelliteFallbackPlan {
            enabled: request.satellite_fallback,
            collections: vec!["sentinel-1-grd".into(), "sentinel-2-l2a".into(), "cop-dem-glo-30".into(), "gpm-imerg-hhr".into()],
            purpose: vec!["flood extent and change detection".into(), "land cover and surface anomaly context".into(), "terrain context".into(), "rainfall context".into()],
            limitations: vec!["satellite data do not replace hydraulic or chemistry observations".into(), "surface proxies require local calibration".into()],
        },
    })
}

pub fn run_baselines(request: &IntegratedStudyRequest, plan: &StudyPlan) -> IntegratedStudyReport {
    let mut domain_results = Vec::new();
    for domain_plan in &plan.domains {
        let result = match domain_plan.domain.as_str() {
            "urban_flood" => request.flood.as_ref().map_or_else(
                || insufficient_result(domain_plan, "No DEM and hydraulic inputs supplied"),
                |input| run_flood(input),
            ),
            "landfill_leachate" => request.leachate.as_ref().map_or_else(
                || insufficient_result(domain_plan, "No landfill water-balance inputs supplied"),
                |input| run_leachate(input),
            ),
            "acid_mine_drainage" => request.amd.as_ref().map_or_else(
                || insufficient_result(domain_plan, "No ABA inputs supplied"),
                |input| run_amd(input),
            ),
            _ => insufficient_result(domain_plan, "Unsupported domain"),
        };
        domain_results.push(result);
    }

    let all_have_inputs = domain_results.iter().all(|result| result.status != "insufficient_data");
    IntegratedStudyReport {
        status: if all_have_inputs { "screening_only".into() } else { "insufficient_data".into() },
        study_plan: plan.clone(),
        domain_results,
        validation: ValidationSummary { status: "not_run".into(), reason: "No independent paired observations were supplied to the workflow".into() },
        uncertainty: UncertaintySummary { status: "not_available".into(), method: "No ensemble or parameter-distribution run was requested".into() },
        satellite_discovery: None,
        provenance: vec!["user-supplied GeoJSON and optional baseline inputs".into()],
        limitations: vec!["This vertical slice does not perform sewer coupling, PHREEQC reactive transport, calibration, or trained AI inference".into()],
    }
}

fn insufficient_result(plan: &DomainPlan, reason: &str) -> DomainResult {
    DomainResult { domain: plan.domain.clone(), status: "insufficient_data".into(), method: plan.method.clone(), output: None, summary: serde_json::json!({}), limitations: vec![reason.into()] }
}

fn run_flood(input: &FloodBaselineInput) -> DomainResult {
    let ny = input.dem.len();
    let nx = input.dem.first().map_or(0, Vec::len);
    if nx < 3 || ny < 3 || input.dem.iter().any(|row| row.len() != nx) {
        return DomainResult { domain: "urban_flood".into(), status: "insufficient_data".into(), method: "2D SWE baseline".into(), output: None, summary: serde_json::json!({}), limitations: vec!["DEM must be a rectangular grid with at least 3x3 cells".into()] };
    }
    let result = swe_solver::solve(
        &input.dem,
        &swe_solver::SweParams { nx, ny, dx: input.dx_m, manning_n: input.manning_n, duration_s: input.duration_s, dt_max: input.dt_max_s, second_order: input.second_order },
        input.inflow_discharge_m3s,
        input.inflow_x,
        input.inflow_y,
        input.inflow_width,
    );
    DomainResult { domain: "urban_flood".into(), status: "screening_only".into(), method: "2D SWE HLLC/MUSCL baseline".into(), output: Some(result.summary), summary: serde_json::json!({"max_depth_m": result.max_depth, "flooded_cells": result.flooded_cells, "total_cells": result.total_cells, "flooded_area_m2": result.flooded_area_m2}), limitations: vec!["No observed flood extent/depth validation".into(), "No 1D sewer network or rainfall-runoff coupling".into()] }
}

fn run_leachate(input: &LeachateBaselineInput) -> DomainResult {
    let json_string = leachate::calculate(input.area_m2, &input.monthly_rainfall_mm, &input.monthly_et_mm, input.soil_storage_mm, input.runoff_coeff);
    
    // Parse the JSON array returned by the leachate tool
    let output_json: serde_json::Value = serde_json::from_str(&json_string).unwrap_or(serde_json::json!({}));
    
    DomainResult { 
        domain: "landfill_leachate".into(), 
        status: "screening_only".into(), 
        method: "monthly water balance".into(), 
        output: None, // String report is removed, using strict JSON
        summary: output_json, 
        limitations: vec!["No quality generation, liner transport, reactive chemistry, or field validation".into()] 
    }
}

fn run_amd(input: &AmdBaselineInput) -> DomainResult {
    let json_string = acid_mine_drainage::calculate(input.sulfur_pct, input.anc_kg_h2so4_t, input.nag_ph);
    let output_json: serde_json::Value = serde_json::from_str(&json_string).unwrap_or(serde_json::json!({}));
    
    DomainResult { 
        domain: "acid_mine_drainage".into(), 
        status: "screening_only".into(), 
        method: "ABA MPA/NAPP static screening".into(), 
        output: None, 
        summary: output_json, 
        limitations: vec!["No kinetic test simulation, PHREEQC execution, reactive transport, or field validation".into()] 
    }
}

fn normalize_domain(domain: &str) -> Option<String> {
    match domain.trim().to_lowercase().as_str() {
        "urban_flood" | "flood" | "banjir" => Some("urban_flood".into()),
        "landfill_leachate" | "leachate" | "lindi" => Some("landfill_leachate".into()),
        "acid_mine_drainage" | "amd" | "air_asam_tambang" => Some("acid_mine_drainage".into()),
        _ => None,
    }
}

fn geojson_bbox(input: &str) -> Result<[f64; 4], String> {
    let geojson = input.parse::<geojson::GeoJson>().map_err(|error| format!("GeoJSON parse error: {}", error))?;
    let mut bbox = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    match geojson {
        geojson::GeoJson::Geometry(geometry) => collect_geometry_coordinates(&geometry.value, &mut bbox),
        geojson::GeoJson::Feature(feature) => {
            if let Some(geometry) = feature.geometry { collect_geometry_coordinates(&geometry.value, &mut bbox); }
        }
        geojson::GeoJson::FeatureCollection(collection) => {
            for feature in collection.features { if let Some(geometry) = feature.geometry { collect_geometry_coordinates(&geometry.value, &mut bbox); } }
        }
    }
    if bbox[0].is_infinite() || bbox.iter().any(|value| !value.is_finite()) { return Err("GeoJSON contains no finite coordinates".into()); }
    if bbox[0] < -180.0 || bbox[2] > 180.0 || bbox[1] < -90.0 || bbox[3] > 90.0 { return Err("GeoJSON coordinates must be WGS84 longitude/latitude".into()); }
    Ok(bbox)
}

fn collect_geometry_coordinates(value: &geojson::Value, bbox: &mut [f64; 4]) {
    match value {
        geojson::Value::Point(point) => collect_position(point, bbox),
        geojson::Value::MultiPoint(points) | geojson::Value::LineString(points) => for point in points { collect_position(point, bbox) },
        geojson::Value::MultiLineString(lines) | geojson::Value::Polygon(lines) => for line in lines { for point in line { collect_position(point, bbox) } },
        geojson::Value::MultiPolygon(polygons) => for polygon in polygons { for line in polygon { for point in line { collect_position(point, bbox) } } },
        geojson::Value::GeometryCollection(_) => {}
    }
}

fn collect_position(position: &[f64], bbox: &mut [f64; 4]) {
    if position.len() >= 2 && position[0].is_finite() && position[1].is_finite() {
        bbox[0] = bbox[0].min(position[0]); bbox[1] = bbox[1].min(position[1]);
        bbox[2] = bbox[2].max(position[0]); bbox[3] = bbox[3].max(position[1]);
    }
}
