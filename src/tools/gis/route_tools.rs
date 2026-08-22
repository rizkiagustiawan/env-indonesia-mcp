use std::path::{Path, PathBuf};
use std::process::Command;

const GRAVITY_SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tools/gis/dem_gravity_extractor.py"
);
const QGIS_EXPORT_SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tools/gis/qgis_exporter.py"
);

fn required_file(path: &str, label: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} path must not be empty"));
    }
    let candidate = PathBuf::from(trimmed);
    if !candidate.exists() {
        return Err(format!("{label} path does not exist: {trimmed}"));
    }
    if !candidate.is_file() {
        return Err(format!("{label} path is not a regular file: {trimmed}"));
    }
    Ok(candidate)
}

fn new_output(path: &str, label: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} output path must not be empty"));
    }
    let candidate = PathBuf::from(trimmed);
    if candidate.exists() {
        return Err(format!(
            "{label} output already exists; refusing to overwrite: {trimmed}"
        ));
    }
    if let Some(parent) = candidate.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(format!(
                "{label} output parent does not exist: {}",
                parent.display()
            ));
        }
    }
    Ok(candidate)
}

pub(crate) fn gravity_nodes_output_path(nodes_csv: &str) -> PathBuf {
    let path = Path::new(nodes_csv);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("nodes");
    path.with_file_name(format!("{stem}_3d.csv"))
}

pub fn validate_gravity_request(
    dem_path: &str,
    nodes_csv: &str,
    edges_csv: &str,
    output_edges_csv: &str,
) -> Result<(), String> {
    new_output(output_edges_csv, "gravity network")?;
    required_file(dem_path, "DEM")?;
    required_file(nodes_csv, "nodes CSV")?;
    required_file(edges_csv, "edges CSV")?;
    let companion = gravity_nodes_output_path(nodes_csv);
    if companion.exists() {
        return Err(format!(
            "gravity network companion output already exists; refusing to overwrite: {}",
            companion.display()
        ));
    }
    Ok(())
}

pub fn validate_qgis_export_request(
    shp_path: &str,
    route: &str,
    output_geojson: &str,
) -> Result<(), String> {
    parse_route_nodes(route)?;
    required_file(shp_path, "Shapefile")?;
    new_output(output_geojson, "QGIS export")?;
    Ok(())
}

pub fn parse_route_nodes(route: &str) -> Result<Vec<String>, String> {
    let nodes: Vec<String> = route
        .split("->")
        .map(str::trim)
        .filter(|node| !node.is_empty())
        .map(str::to_string)
        .collect();
    if nodes.is_empty() {
        return Err("route must contain at least one node".to_string());
    }
    let mut seen = std::collections::HashSet::new();
    if nodes.iter().any(|node| !seen.insert(node)) {
        return Err("route must not contain duplicate nodes".to_string());
    }
    Ok(nodes)
}

fn python() -> String {
    std::env::var("ENV_INDONESIA_PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn run_script(script: &str, args: &[&str]) -> String {
    let output = match Command::new(python()).arg(script).args(args).output() {
        Ok(output) => output,
        Err(error) => {
            return serde_json::json!({
                "status": "error",
                "error": format!("failed to start GIS helper: {error}"),
            })
            .to_string()
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() && !stdout.is_empty() {
        return stdout;
    }
    serde_json::json!({
        "status": "error",
        "error": if stderr.is_empty() { stdout } else { stderr },
    })
    .to_string()
}

pub fn build_gravity_network(
    dem_path: &str,
    nodes_csv: &str,
    edges_csv: &str,
    output_edges_csv: &str,
) -> String {
    if let Err(error) = validate_gravity_request(dem_path, nodes_csv, edges_csv, output_edges_csv) {
        return serde_json::json!({ "status": "invalid_request", "error": error }).to_string();
    }
    run_script(
        GRAVITY_SCRIPT,
        &[dem_path, nodes_csv, edges_csv, output_edges_csv],
    )
}

pub fn export_qgis_route(shp_path: &str, route: &str, output_geojson: &str) -> String {
    if let Err(error) = validate_qgis_export_request(shp_path, route, output_geojson) {
        return serde_json::json!({ "status": "invalid_request", "error": error }).to_string();
    }
    run_script(QGIS_EXPORT_SCRIPT, &[shp_path, route, output_geojson])
}

pub fn script_exists() -> bool {
    Path::new(GRAVITY_SCRIPT).is_file() && Path::new(QGIS_EXPORT_SCRIPT).is_file()
}
