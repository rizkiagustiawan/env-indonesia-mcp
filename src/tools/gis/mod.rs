pub mod advanced;
pub mod cartography;
pub mod coords;
pub mod drought;
pub mod geojson_ops;
pub mod landcover;
pub mod landslide;
pub mod ndvi;
pub mod route_tools;
pub mod spatial_ops;
pub mod spatial_validation;
pub mod viewshed;
pub mod water;

#[cfg(test)]
mod route_tools_contract_tests {
    use super::route_tools::{
        parse_route_nodes, validate_gravity_request, validate_qgis_export_request,
    };

    #[test]
    fn gravity_request_rejects_missing_inputs_and_existing_output() {
        let error = validate_gravity_request(
            "/tmp/missing.dem",
            "/tmp/missing-nodes.csv",
            "/tmp/missing-edges.csv",
            "",
        )
        .unwrap_err();
        assert!(error.contains("output"), "unexpected error: {error}");
    }

    #[test]
    fn qgis_export_request_rejects_empty_route_and_overwrite() {
        let error = validate_qgis_export_request("/tmp/missing.shp", "", "").unwrap_err();
        assert!(error.contains("route"), "unexpected error: {error}");
    }

    #[test]
    fn route_parser_rejects_empty_or_duplicate_nodes() {
        assert!(parse_route_nodes("").is_err());
        assert!(parse_route_nodes("A -> A").is_err());
        assert_eq!(parse_route_nodes("A -> B -> C").unwrap(), ["A", "B", "C"]);
    }

    #[test]
    fn helper_scripts_are_shipped_with_the_server() {
        assert!(super::route_tools::script_exists());
    }

    #[test]
    fn gravity_companion_path_uses_nodes_stem() {
        assert_eq!(
            super::route_tools::gravity_nodes_output_path("/tmp/graph/nodes.csv"),
            std::path::PathBuf::from("/tmp/graph/nodes_3d.csv")
        );
    }
}
