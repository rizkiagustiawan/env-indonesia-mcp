use std::sync::LazyLock;

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_router,
};

use crate::tools;
pub use crate::tools::physics_validator::ValidatorParam;

// Calculator & Compliance Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RusleParam { pub r: f64, pub k: f64, pub ls: f64, pub c: f64, pub p: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ScsCnParam { pub rainfall_mm: f64, pub cn: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PenmanParam { pub t_mean_c: f64, pub rh_pct: f64, pub wind_ms: f64, pub rn_mj: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StreeterPhelpsParam { pub k1: f64, pub k2: f64, pub l0: f64, pub d0: f64, pub velocity_ms: f64, pub distance_km: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DoSatParam { pub water_temp_c: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaterBalanceParam { pub p_mm: f64, pub et_mm: f64, pub q_mm: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GaussianParam { pub emission_gs: f64, pub wind_ms: f64, pub stack_height_m: f64, pub distance_m: f64, pub stability_class: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NoiseParam { pub source_db: f64, pub distance_m: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandfillParam { pub waste_ton: f64, pub years_open: u32, pub k_decay: f64, pub l0_potential: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SolidWasteParam { pub population: u64, pub generation_rate_kg: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProperParam { pub has_izin: bool, pub compliance_pct: f64, pub beyond_compliance: bool, pub community_dev: bool, pub circular_economy: bool }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IklhParam { pub ika: f64, pub iku: f64, pub iktl: f64 }

// Fase 1+2 Calculator Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WastewaterParam { pub q_m3d: f64, pub bod_influent: f64, pub bod_target: f64, pub temp_c: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PeatlandParam { pub water_table_depth_cm: f64, pub area_ha: f64, pub years: u32 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MangroveNdmiParam { pub nir_b8a: f64, pub swir_b11: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TsunamiParam { pub depth_m: f64, pub distance_km: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HeatIndexParam { pub temp_c: f64, pub rh_pct: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EutrophicationParam { pub secchi_depth_m: Option<f64>, pub chlorophyll_ugl: Option<f64>, pub total_phosphorus_ugl: Option<f64> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SoilTextureParam { pub sand_pct: f64, pub silt_pct: f64, pub clay_pct: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EflowParam { pub maf_m3s: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdfParam { pub r24_mm: f64, pub duration_hours: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RainwaterParam { pub roof_area_m2: f64, pub rainfall_mm: f64, pub runoff_coeff: f64, pub demand_liters_day: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FootprintParam { pub electricity_kwh: f64, pub vehicle_km: f64, pub meat_kg_week: f64, pub waste_kg_day: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LcaParam { pub material: String, pub mass_kg: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UvParam { pub solar_zenith_deg: f64, pub altitude_m: f64, pub ozone_du: f64, pub cloud_cover_pct: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanAcidParam { pub ph: f64, pub pco2_uatm: f64, pub temp_c: f64, pub salinity_psu: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SubsidenceParam { pub clay_thickness_m: f64, pub delta_stress_kpa: f64, pub cc: f64, pub e0: f64, pub sigma0_kpa: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ThermalParam { pub q_river_m3s: f64, pub t_river_c: f64, pub q_discharge_m3s: f64, pub t_discharge_c: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SlrParam { pub elevation_m: f64, pub slr_m: f64, pub storm_surge_m: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WteParam { pub waste_ton_day: f64, pub moisture_pct: f64, pub organic_pct: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AcidRainParam { pub so2_ugm3: f64, pub nox_ugm3: f64, pub rainfall_mm_yr: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MicroplasticParam { pub water_type: String, pub particles_per_liter: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MangroveCarbonParam { pub dbh_cm: f64, pub wood_density: f64, pub trees_per_ha: f64 }

// Processing Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PdfReportParam { pub title: String, pub sections_json: String, pub output_path: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GeotiffCropParam { pub input_path: String, pub output_path: String, pub bbox: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WatershedParam { pub dem_path: String, pub pour_x: f64, pub pour_y: f64, pub output_path: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdwParam { pub points: Vec<Vec<f64>>, pub target_x: f64, pub target_y: f64, pub power: Option<f64> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Terrain3dParam { pub dem_path: String, pub output_path: String, pub title: String, pub exaggeration: Option<f64> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Timeseries4dParam { pub values: String, pub output_path: String, pub title: String, pub labels: Option<String>, pub ylabel: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Flood3dParam { pub dem_path: String, pub output_path: String, pub water_level_m: f64, pub title: String, pub exaggeration: Option<f64> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Flood4dParam { pub dem_path: String, pub output_path: String, pub water_start_m: f64, pub water_end_m: f64, pub steps: Option<u32>, pub title: String, pub exaggeration: Option<f64> }

// Air Quality Dispersion Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StabilityParam { pub wind_speed_ms: f64, pub solar_radiation: String, pub cloud_cover_eighths: u32 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PlumeRiseParam { pub stack_height_m: f64, pub exit_velocity_ms: f64, pub exit_temp_k: f64, pub ambient_temp_k: f64, pub wind_speed_ms: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Dispersion2dParam { pub sources_json: String, pub wind_speed: f64, pub wind_dir: f64, pub stability: String, pub output_path: String, pub title: String, pub grid_size: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Dispersion4dParam { pub sources_json: String, pub wind_speeds: String, pub wind_dirs: String, pub stability: String, pub output_path: String, pub title: String, pub grid_size: Option<u32> }

// Ocean Modeling Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanBathyParam { pub lat: f64, pub lon: f64, pub output_path: String, pub title: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanCurrentParam { pub lat: f64, pub lon: f64, pub wind_speed: f64, pub wind_dir: f64, pub output_path: String, pub title: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanThermalParam { pub discharge_temp: f64, pub ambient_temp: f64, pub output_path: String, pub title: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanPollutionParam { pub current_speeds: String, pub current_dirs: String, pub output_path: String, pub title: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaveParam { pub wind_speed_ms: f64, pub fetch_m: f64, pub depth_m: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CoralDhwParam { pub sst_weekly: String, pub sst_max_monthly_mean: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SedimentParam { pub hs_m: f64, pub wave_angle_deg: f64, pub beach_slope_deg: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OilSpillParam { pub volume_m3: f64, pub oil_type: String, pub wind_speed: f64, pub wind_dir: f64, pub current_speed: f64, pub current_dir: f64, pub hours: u32, pub output_path: String }

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("env-ntb-mcp/0.1.0")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
});

#[derive(Debug, Clone)]
pub struct EnvNtbServer {
    tool_router: ToolRouter<Self>,
}

impl EnvNtbServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

// ====== Parameter structs ======

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LocationParam {
    #[schemars(description = "City name (mataram/bima/sumbawa/dompu) or BMKG adm4 code")]
    pub location: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DaysParam {
    #[schemars(description = "Number of days (1-10, default 1)")]
    pub days: Option<u32>,
    #[schemars(description = "Bounding box: south,west,north,east. Default: Indonesia (-11,95,6,141)")]
    pub bbox: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SatelliteSearchParam {
    #[schemars(description = "Satellite: SENTINEL-1, SENTINEL-2, SENTINEL-3, SENTINEL-5P")]
    pub collection: String,
    #[schemars(description = "Max results (default 5)")]
    pub limit: Option<u32>,
    #[schemars(description = "Bounding box: south,west,north,east. Default: Indonesia")]
    pub bbox: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LatLonParam {
    #[schemars(description = "Latitude (default -8.58 Mataram)")]
    pub lat: Option<f64>,
    #[schemars(description = "Longitude (default 116.10 Mataram)")]
    pub lon: Option<f64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LatLonRequired {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryParam {
    #[schemars(description = "Search query")]
    pub query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SectorParam {
    #[schemars(description = "Sector: power, agriculture, mining, energy, waste, forestry")]
    pub sector: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NdviParam {
    #[schemars(description = "Near-infrared band value (Sentinel-2 B8)")]
    pub nir: f64,
    #[schemars(description = "Red band value (Sentinel-2 B4)")]
    pub red: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaterQualityParam {
    #[schemars(description = "Green band (B3)")]
    pub green: f64,
    #[schemars(description = "Red band (B4)")]
    pub red: f64,
    #[schemars(description = "NIR band (B8)")]
    pub nir: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DroughtParam {
    #[schemars(description = "Monthly precipitation in mm")]
    pub precipitation_mm: f64,
    #[schemars(description = "Long-term average precipitation mm")]
    pub avg_mm: f64,
    #[schemars(description = "Standard deviation mm")]
    pub std_mm: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CarbonParam {
    #[schemars(description = "Activity: electricity_kwh, diesel, gasoline, lpg_kg, waste_ton, flight_km, vehicle_km, rice_paddy_ha, deforestation_ha")]
    pub activity: String,
    #[schemars(description = "Amount (numeric)")]
    pub amount: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OjkParam {
    #[schemars(description = "Entity: bank, insurance, securities, financing")]
    pub entity_type: String,
    #[schemars(description = "Comma-separated disclosures already made")]
    pub disclosures: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TcfdParam {
    #[schemars(description = "Sector: agriculture, mining, energy, tourism, fisheries")]
    pub sector: String,
    #[schemars(description = "Location in NTB: Lombok, Sumbawa, Bima, Mataram")]
    pub location: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GeoJsonParam {
    #[schemars(description = "GeoJSON string")]
    pub geojson: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CoordParam {
    #[schemars(description = "X / Longitude")]
    pub x: f64,
    #[schemars(description = "Y / Latitude")]
    pub y: f64,
    #[schemars(description = "wgs84_to_utm or utm_to_wgs84")]
    pub direction: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MapGenParam {
    #[schemars(description = "GeoJSON String dari area/titik yang mau digambar")]
    pub geojson: String,
    #[schemars(description = "Path lengkap untuk menyimpan gambar (misal: /tmp/peta_banjir.png)")]
    pub output_path: String,
    #[schemars(description = "Judul Peta untuk Kop SNI")]
    pub title: String,
    #[schemars(description = "Jika true, otomatis mendownload Sentinel-2 (Bebas awan 30 hari terakhir) via GEE sbg Basemap.")]
    pub realtime_satellite: Option<bool>,
}

// ====== Tool implementations ======

use rmcp::handler::server::wrapper::Parameters;

#[tool_router]
impl EnvNtbServer {
    // --- DATA INDONESIA ---
    #[tool(description = "BMKG weather forecast for NTB cities")]
    async fn bmkg_weather(&self, Parameters(p): Parameters<LocationParam>) -> String {
        tools::data::bmkg::weather(&HTTP, &p.location).await
    }

    #[tool(description = "BMKG 15 latest earthquakes near Indonesia")]
    async fn bmkg_earthquake(&self) -> String {
        tools::data::bmkg::earthquake(&HTTP).await
    }

    #[tool(description = "NASA FIRMS fire hotspots (VIIRS satellite). Bbox default to Indonesia.")]
    async fn firms_fire(&self, Parameters(p): Parameters<DaysParam>) -> String {
        tools::data::firms::fire_hotspots(&HTTP, p.days.unwrap_or(1), p.bbox).await
    }

    #[tool(description = "Global Forest Watch deforestation alerts Indonesia")]
    async fn gfw_deforestation(&self) -> String {
        tools::data::gfw::deforestation_alerts(&HTTP).await
    }

    #[tool(description = "Search Copernicus Sentinel imagery catalog")]
    async fn copernicus_search(&self, Parameters(p): Parameters<SatelliteSearchParam>) -> String {
        tools::satellite::copernicus::search(&HTTP, &p.collection, p.limit.unwrap_or(5), p.bbox).await
    }

    #[tool(description = "Air pollution AQI PM2.5 NO2 O3 SO2 CO (Open-Meteo CAMS)")]
    async fn air_pollution(&self, Parameters(p): Parameters<LatLonParam>) -> String {
        tools::data::openweather::air_pollution(&HTTP, p.lat.unwrap_or(-8.58), p.lon.unwrap_or(116.10)).await
    }

    #[tool(description = "Weather 7-day forecast (Open-Meteo, free, no API key)")]
    async fn open_meteo_weather(&self, Parameters(p): Parameters<LatLonParam>) -> String {
        tools::data::open_meteo::weather(&HTTP, p.lat.unwrap_or(-8.58), p.lon.unwrap_or(116.10)).await
    }

    #[tool(description = "NASA POWER solar irradiance GHI DNI monthly for energy potential")]
    async fn nasa_power_solar(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        tools::data::nasa_power::solar(&HTTP, p.lat, p.lon, None, None).await
    }

    #[tool(description = "Search Satu Data Indonesia (data.go.id) environmental datasets")]
    async fn satu_data_search(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::data::satu_data::search(&HTTP, &p.query, 5).await
    }

    #[tool(description = "Climate TRACE GHG emissions Indonesia by sector")]
    async fn climate_trace_emissions(&self, Parameters(p): Parameters<SectorParam>) -> String {
        tools::data::climate_trace::emissions(&HTTP, p.sector).await
    }

    // --- GIS ANALYSIS ---
    #[tool(description = "NDVI vegetation index from Sentinel-2 bands. NDVI=(NIR-Red)/(NIR+Red)")]
    fn ndvi_compute(&self, Parameters(p): Parameters<NdviParam>) -> String {
        tools::gis::ndvi::compute(p.nir, p.red)
    }

    #[tool(description = "Water quality indices from Sentinel-2: NDWI, turbidity, chlorophyll")]
    fn water_quality(&self, Parameters(p): Parameters<WaterQualityParam>) -> String {
        tools::gis::water::quality(p.green, p.red, p.nir, None)
    }

    #[tool(description = "Drought index SPI from precipitation data")]
    fn drought_index(&self, Parameters(p): Parameters<DroughtParam>) -> String {
        tools::gis::drought::index(p.precipitation_mm, p.avg_mm, p.std_mm)
    }

    #[tool(description = "Analyze GeoJSON: type, features, geometry")]
    fn geojson_analyze(&self, Parameters(p): Parameters<GeoJsonParam>) -> String {
        tools::gis::geojson_ops::analyze(&p.geojson)
    }

    #[tool(description = "Coordinate transform WGS84 <-> UTM Zone 50S (NTB)")]
    fn coordinate_transform(&self, Parameters(p): Parameters<CoordParam>) -> String {
        tools::gis::coords::transform(p.x, p.y, &p.direction)
    }

    // --- ESG ANALYTICS ---
    #[tool(description = "Carbon footprint calculator with Indonesia emission factors (IPCC + Perpres 98/2021)")]
    fn carbon_calculator(&self, Parameters(p): Parameters<CarbonParam>) -> String {
        tools::esg::carbon::calculate(&p.activity, p.amount)
    }

    #[tool(description = "Map activity to UN SDGs (17 Sustainable Development Goals)")]
    fn sdg_mapper(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::esg::sdg::map_activity(&p.query)
    }

    #[tool(description = "OJK POJK 51/2017 ESG compliance checker for Indonesian financial institutions")]
    fn ojk_compliance(&self, Parameters(p): Parameters<OjkParam>) -> String {
        tools::esg::ojk::check_compliance(&p.entity_type, &p.disclosures)
    }

    #[tool(description = "TCFD climate risk assessment for NTB sectors")]
    fn climate_risk_tcfd(&self, Parameters(p): Parameters<TcfdParam>) -> String {
        tools::esg::tcfd::risk_assessment(&p.sector, &p.location)
    }

    // --- OCEAN & MARINE ---
    #[tool(description = "Coral reef health NTB: Gili Islands, Lombok, Sumbawa, Moyo Island")]
    fn coral_reef_health(&self) -> String {
        tools::ocean::coral::reef_health()
    }

    #[tool(description = "Marine protected areas NTB: TWP Gili Matra, TNGR, Tambora from WDPA")]
    fn marine_protected_areas(&self) -> String {
        tools::ocean::mpa::protected_areas()
    }

    // --- WRAPPERS (Existing Projects) ---
    #[tool(description = "Wrapper: Trigger ESG Audit pipeline in GeoESG-Final (Port 8000)")]
    async fn wrapper_esg_audit(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::wrappers::trigger_esg_audit(&HTTP, &p.query).await
    }

    #[tool(description = "Wrapper: Predict flood via geo-ntb-flood-ai (Port 8001)")]
    async fn wrapper_flood_predict(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        tools::wrappers::predict_flood(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "Wrapper: Get methane plumes data via Gas-Metana-NTB (Port 8002)")]
    async fn wrapper_methane_plumes(&self) -> String {
        tools::wrappers::get_methane_plumes(&HTTP).await
    }

    #[tool(description = "Wrapper: Get groundwater monitoring status (Port 8003)")]
    async fn wrapper_groundwater(&self) -> String {
        tools::wrappers::get_groundwater_status(&HTTP).await
    }

    #[tool(description = "Wrapper: Get air quality monitoring health (Port 8004)")]
    async fn wrapper_air_quality(&self) -> String {
        tools::wrappers::get_air_quality(&HTTP).await
    }

    // --- SATELLITE TOOLS ---
    #[tool(description = "Status Gunung Api NTB (Rinjani, Tambora) dari MAGMA Indonesia")]
    async fn magma_volcano(&self) -> String {
        tools::data::magma::status(&HTTP).await
    }

    #[tool(description = "BPS Environmental Statistics for NTB. keyword: hutan/sampah/air/ekonomi")]
    async fn bps_environment(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::data::bps::statistics(&HTTP, &p.query).await
    }

    #[tool(description = "InaRISK BNPB Disaster Risk Assessment. location: lombok/sumbawa/bima")]
    async fn inarisk_hazard(&self, Parameters(p): Parameters<LocationParam>) -> String {
        tools::data::inarisk::disaster_risk(&HTTP, &p.location).await
    }

    #[tool(description = "USGS Landsat Archive Search information for NTB.")]
    async fn satellite_landsat(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::satellite::landsat::search(&HTTP, &p.query).await
    }

    #[tool(description = "NASA MODIS products information for environmental monitoring.")]
    async fn satellite_modis(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::satellite::modis::query(&HTTP, &p.query).await
    }

    #[tool(description = "NASA VIIRS products information (Nighttime lights, active fires).")]
    async fn satellite_viirs(&self) -> String {
        tools::satellite::viirs::query(&HTTP).await
    }

    #[tool(description = "SRTM 30m Digital Elevation Model information for NTB.")]
    async fn satellite_srtm(&self) -> String {
        tools::satellite::srtm::info(&HTTP).await
    }

    #[tool(description = "CHIRPS Rainfall dataset information for drought analysis.")]
    async fn satellite_chirps(&self) -> String {
        tools::satellite::chirps::query(&HTTP).await
    }

    #[tool(description = "NASA GRACE / GRACE-FO Groundwater Storage anomaly information.")]
    async fn satellite_grace(&self) -> String {
        tools::satellite::grace::query(&HTTP).await
    }

    #[tool(description = "Google Dynamic World 10m near real-time land cover info.")]
    async fn satellite_dynamic_world(&self) -> String {
        tools::satellite::dynamic_world::query(&HTTP).await
    }

    #[tool(description = "ECMWF ERA5 Climate Reanalysis information for long-term trends.")]
    async fn satellite_era5(&self) -> String {
        tools::satellite::era5::query(&HTTP).await
    }

    // --- ADVANCED GIS & ESG ---
    #[tool(description = "Parse Sustainability Report (PDF) for ESG Analytics.")]
    async fn esg_report_parser(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::esg::report_parser::parse_esg_report(&p.query).await
    }

    #[tool(description = "Calculate Slope/Gradient from SRTM DEM using GDAL/Rust.")]
    async fn gis_dem_slope(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::gis::advanced::dem_slope(&p.query).await
    }

    #[tool(description = "Zonal Raster Statistics (Mean/Max/Sum) based on GeoJSON polygon boundary.")]
    async fn gis_raster_stats(&self, Parameters(p): Parameters<GeoJsonParam>) -> String {
        tools::gis::advanced::raster_stats(&p.geojson, &p.geojson).await
    }

    #[tool(description = "Land Cover Classifier using Random Forest ML (Rust Linfa crate).")]
    async fn gis_land_cover_classifier(&self) -> String {
        tools::gis::advanced::land_cover_classifier().await
    }

    #[tool(description = "Generate layout Peta (PNG) standar kartografi dari GeoJSON menggunakan citra satelit.")]
    async fn generate_map_sni(&self, Parameters(p): Parameters<MapGenParam>) -> String {
        tools::gis::cartography::generate_map(&p.geojson, &p.output_path, &p.title, p.realtime_satellite.unwrap_or(false))
    }

    #[tool(description = "VALIDATOR FISIKA EKUATORIAL: Wajib dipanggil oleh AI sebelum mengonfirmasi angka analisis untuk banjir, polusi udara, atau vegetasi (NDVI) guna memastikan tidak ada hukum alam yang dilanggar.")]
    async fn physics_check(&self, Parameters(p): Parameters<ValidatorParam>) -> String {
        crate::tools::physics_validator::validate(p)
    }

    // =======================================
    // CALCULATORS — Deterministik, akurat 99%
    // =======================================

    #[tool(description = "RUSLE Soil Erosion: A = R × K × LS × C × P (ton/ha/tahun). Ref: USDA Handbook 703.")]
    fn rusle_erosion(&self, Parameters(p): Parameters<RusleParam>) -> String {
        tools::calculators::rusle::calculate(p.r, p.k, p.ls, p.c, p.p)
    }

    #[tool(description = "SCS-CN Runoff: Q = (P-0.2S)²/(P+0.8S). Ref: USDA TR-55.")]
    fn scs_cn_runoff(&self, Parameters(p): Parameters<ScsCnParam>) -> String {
        tools::calculators::scs_cn::calculate(p.rainfall_mm, p.cn)
    }

    #[tool(description = "Penman-Monteith ET0 (FAO-56). Evapotranspirasi referensi.")]
    fn penman_monteith_et0(&self, Parameters(p): Parameters<PenmanParam>) -> String {
        tools::calculators::penman_monteith::calculate(p.t_mean_c, p.rh_pct, p.wind_ms, p.rn_mj)
    }

    #[tool(description = "Streeter-Phelps DO Sag Curve. Titik kritis DO minimum sungai.")]
    fn streeter_phelps_do(&self, Parameters(p): Parameters<StreeterPhelpsParam>) -> String {
        tools::calculators::streeter_phelps::calculate(p.k1, p.k2, p.l0, p.d0, p.velocity_ms, p.distance_km)
    }

    #[tool(description = "DO Saturation: kelarutan oksigen di air berdasarkan suhu. Ref: APHA.")]
    fn do_saturation(&self, Parameters(p): Parameters<DoSatParam>) -> String {
        tools::calculators::do_saturation::calculate(p.water_temp_c)
    }

    #[tool(description = "Water Balance: P = ET + Q + ΔS. Neraca air (konservasi massa).")]
    fn water_balance(&self, Parameters(p): Parameters<WaterBalanceParam>) -> String {
        tools::calculators::water_balance::calculate(p.p_mm, p.et_mm, p.q_mm)
    }

    #[tool(description = "Gaussian Plume Dispersion. Sebaran polutan cerobong. stability: A-F.")]
    fn gaussian_plume(&self, Parameters(p): Parameters<GaussianParam>) -> String {
        tools::calculators::gaussian_plume::calculate(p.emission_gs, p.wind_ms, p.stack_height_m, p.distance_m, &p.stability_class)
    }

    #[tool(description = "Noise dB Attenuation. Kebisingan vs jarak. Ref: ISO 9613.")]
    fn noise_attenuation(&self, Parameters(p): Parameters<NoiseParam>) -> String {
        tools::calculators::noise_db::attenuation_distance(p.source_db, p.distance_m)
    }

    #[tool(description = "Landfill Gas CH4 Estimator. Emisi metana TPA. Ref: EPA LandGEM.")]
    fn landfill_gas(&self, Parameters(p): Parameters<LandfillParam>) -> String {
        tools::calculators::landfill_gas::calculate(p.waste_ton, p.years_open, p.k_decay, p.l0_potential)
    }

    #[tool(description = "Solid Waste Calculator. Timbulan sampah & target Jakstranas 2025.")]
    fn solid_waste_calc(&self, Parameters(p): Parameters<SolidWasteParam>) -> String {
        tools::calculators::solid_waste::calculate(p.population, p.generation_rate_kg)
    }

    // =======================================
    // COMPLIANCE — Regulasi Indonesia
    // =======================================

    #[tool(description = "PROPER Scoring: HITAM-MERAH-BIRU-HIJAU-EMAS. Ref: PermenLHK P.1/2021.")]
    fn proper_score(&self, Parameters(p): Parameters<ProperParam>) -> String {
        tools::compliance::proper::score(p.has_izin, p.compliance_pct, p.beyond_compliance, p.community_dev, p.circular_economy)
    }

    #[tool(description = "IKLH: Indeks Kualitas Lingkungan Hidup = (IKA×30%)+(IKU×30%)+(IKTL×40%). Ref: PermenLHK P.27/2021.")]
    fn iklh_calculator(&self, Parameters(p): Parameters<IklhParam>) -> String {
        tools::compliance::iklh::calculate(p.ika, p.iku, p.iktl)
    }

    #[tool(description = "Klasifikasi Limbah B3 (Bahan Berbahaya & Beracun). Ref: PP 101/2014.")]
    fn b3_classifier(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::compliance::b3_classifier::classify(&p.query)
    }

    // =======================================
    // FASE 1: 10 TOOLS KRITIS
    // =======================================

    #[tool(description = "Desain IPAL Activated Sludge (Monod kinetics). Ref: Metcalf & Eddy 2003.")]
    fn wastewater_design(&self, Parameters(p): Parameters<WastewaterParam>) -> String {
        tools::calculators::wastewater::design(p.q_m3d, p.bod_influent, p.bod_target, p.temp_c)
    }

    #[tool(description = "Peatland Subsidence & CO2 Emission. Ref: Hooijer 2012.")]
    fn peatland_subsidence(&self, Parameters(p): Parameters<PeatlandParam>) -> String {
        tools::calculators::peatland::calculate(p.water_table_depth_cm, p.area_ha, p.years)
    }

    #[tool(description = "Mangrove NDMI (Gao 1996). Band: Sentinel-2 B8A & B11.")]
    fn mangrove_ndmi(&self, Parameters(p): Parameters<MangroveNdmiParam>) -> String {
        tools::calculators::mangrove::ndmi(p.nir_b8a, p.swir_b11)
    }

    #[tool(description = "Tsunami Travel Time: c=√(g×d). Ref: shallow water wave theory.")]
    fn tsunami_travel_time(&self, Parameters(p): Parameters<TsunamiParam>) -> String {
        tools::calculators::tsunami::travel_time(p.depth_m, p.distance_km)
    }

    #[tool(description = "Heat Index (Rothfusz/NWS). Valid T≥27°C, RH≥40%.")]
    fn heat_index(&self, Parameters(p): Parameters<HeatIndexParam>) -> String {
        tools::calculators::heat_index::calculate(p.temp_c, p.rh_pct)
    }

    #[tool(description = "Carlson TSI Eutrophication Index. Hanya valid untuk DANAU. Ref: Carlson 1977.")]
    fn eutrophication_tsi(&self, Parameters(p): Parameters<EutrophicationParam>) -> String {
        tools::calculators::eutrophication::calculate(p.secchi_depth_m, p.chlorophyll_ugl, p.total_phosphorus_ugl)
    }

    #[tool(description = "Soil Texture Classification (USDA triangle). Input: sand%, silt%, clay%.")]
    fn soil_texture(&self, Parameters(p): Parameters<SoilTextureParam>) -> String {
        tools::calculators::soil_quality::classify_texture(p.sand_pct, p.silt_pct, p.clay_pct)
    }

    #[tool(description = "Environmental Flow Tennant Method. ⚠️ Screening awal saja.")]
    fn environmental_flow(&self, Parameters(p): Parameters<EflowParam>) -> String {
        tools::calculators::eflow::calculate(p.maf_m3s)
    }

    #[tool(description = "IDF Curve Mononobe. Intensitas hujan dari R24 & durasi. Ref: standar Indonesia.")]
    fn idf_mononobe(&self, Parameters(p): Parameters<IdfParam>) -> String {
        tools::calculators::idf_curve::mononobe(p.r24_mm, p.duration_hours)
    }

    #[tool(description = "AMDAL Leopold Matrix scoring. Ref: Leopold 1971, PP 22/2021.")]
    fn amdal_leopold(&self) -> String {
        // Demo: empty matrix info
        tools::calculators::amdal::score(&[
            ("Pembersihan Lahan".into(), "Kualitas Air".into(), -6, 8),
            ("Konstruksi".into(), "Kebisingan".into(), -4, 5),
            ("Operasi".into(), "Ekonomi Lokal".into(), 7, 9),
        ])
    }

    // =======================================
    // FASE 2: 14 TOOLS PENTING
    // =======================================

    #[tool(description = "Rainwater Harvesting Calculator. Sizing tangki penampungan air hujan.")]
    fn rainwater_harvest(&self, Parameters(p): Parameters<RainwaterParam>) -> String {
        tools::calculators::rainwater::calculate(p.roof_area_m2, p.rainfall_mm, p.runoff_coeff, p.demand_liters_day)
    }

    #[tool(description = "Ecological Footprint (gha). Jejak ekologis personal.")]
    fn ecological_footprint(&self, Parameters(p): Parameters<FootprintParam>) -> String {
        tools::calculators::ecological_footprint::calculate(p.electricity_kwh, p.vehicle_km, p.meat_kg_week, p.waste_kg_day)
    }

    #[tool(description = "Simplified LCA. Cradle-to-gate emission. Materials: baja/semen/plastik/aluminium/kayu/kertas/beton/kaca/bata.")]
    fn lca_simplified(&self, Parameters(p): Parameters<LcaParam>) -> String {
        tools::calculators::lca::calculate(&p.material, p.mass_kg)
    }

    #[tool(description = "UV Index dari solar zenith, altitude, ozone, cloud. Ref: WHO/WMO.")]
    fn uv_index(&self, Parameters(p): Parameters<UvParam>) -> String {
        tools::calculators::uv_index::calculate(p.solar_zenith_deg, p.altitude_m, p.ozone_du, p.cloud_cover_pct)
    }

    #[tool(description = "Ocean Acidification: Ω aragonite dari pH, pCO2, suhu, salinitas. Ref: Zeebe 2001.")]
    fn ocean_acidification(&self, Parameters(p): Parameters<OceanAcidParam>) -> String {
        tools::calculators::ocean_acidification::calculate(p.ph, p.pco2_uatm, p.temp_c, p.salinity_psu)
    }

    #[tool(description = "Land Subsidence Terzaghi 1D Consolidation. Jakarta/Semarang/Pekalongan.")]
    fn land_subsidence(&self, Parameters(p): Parameters<SubsidenceParam>) -> String {
        tools::calculators::land_subsidence::calculate(p.clay_thickness_m, p.delta_stress_kpa, p.cc, p.e0, p.sigma0_kpa)
    }

    #[tool(description = "Thermal Pollution mixing zone. Suhu campuran sungai + buangan PLTU. Baku mutu: ΔT maks 3°C.")]
    fn thermal_pollution(&self, Parameters(p): Parameters<ThermalParam>) -> String {
        tools::calculators::thermal_pollution::calculate(p.q_river_m3s, p.t_river_c, p.q_discharge_m3s, p.t_discharge_c)
    }

    #[tool(description = "Sea Level Rise Inundation (bathtub model). Skenario IPCC AR6.")]
    fn sea_level_rise(&self, Parameters(p): Parameters<SlrParam>) -> String {
        tools::calculators::sea_level_rise::calculate(p.elevation_m, p.slr_m, p.storm_surge_m)
    }

    #[tool(description = "Waste to Energy Calculator. Nilai kalori sampah → listrik.")]
    fn waste_to_energy(&self, Parameters(p): Parameters<WteParam>) -> String {
        tools::calculators::waste_to_energy::calculate(p.waste_ton_day, p.moisture_pct, p.organic_pct)
    }

    #[tool(description = "Acid Rain Risk. Deposisi S/N vs critical load. Ref: EMEP.")]
    fn acid_rain_risk(&self, Parameters(p): Parameters<AcidRainParam>) -> String {
        tools::calculators::acid_rain::calculate(p.so2_ugm3, p.nox_ugm3, p.rainfall_mm_yr)
    }

    #[tool(description = "Microplastic Risk Scoring. Emerging contaminant.")]
    fn microplastic_risk(&self, Parameters(p): Parameters<MicroplasticParam>) -> String {
        tools::calculators::microplastic::score(&p.water_type, p.particles_per_liter)
    }

    #[tool(description = "Mangrove Carbon Stock allometric. Ref: Komiyama 2005.")]
    fn mangrove_carbon(&self, Parameters(p): Parameters<MangroveCarbonParam>) -> String {
        tools::calculators::mangrove::carbon_stock(p.dbh_cm, p.wood_density, p.trees_per_ha)
    }

    #[tool(description = "Soil pH Assessment. Kategori masam/netral/basa.")]
    fn soil_ph(&self, Parameters(p): Parameters<DoSatParam>) -> String {
        tools::calculators::soil_quality::assess_ph(p.water_temp_c)
    }

    // =======================================
    // PROCESSING — Pipeline & Analysis
    // =======================================

    #[tool(description = "Generate laporan PDF formal (AMDAL/ESG/Environmental Report). sections: JSON array [[title,body],...]")]
    fn generate_pdf_report(&self, Parameters(p): Parameters<PdfReportParam>) -> String {
        tools::processing::pdf_report::generate(&p.title, &p.sections_json, &p.output_path)
    }

    #[tool(description = "GeoTIFF info via GDAL. Metadata citra satelit (CRS, resolusi, band, extent).")]
    fn geotiff_info(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::processing::geotiff::info(&p.query)
    }

    #[tool(description = "Crop/clip GeoTIFF ke bounding box. bbox: 'minlon minlat maxlon maxlat'")]
    fn geotiff_crop(&self, Parameters(p): Parameters<GeotiffCropParam>) -> String {
        tools::processing::geotiff::crop(&p.input_path, &p.output_path, &p.bbox)
    }

    #[tool(description = "Watershed/DAS delineation dari DEM (pysheds D8). Input: DEM .tif + pour point (x,y).")]
    fn watershed_delineation(&self, Parameters(p): Parameters<WatershedParam>) -> String {
        tools::processing::watershed::delineate(&p.dem_path, p.pour_x, p.pour_y, &p.output_path)
    }

    #[tool(description = "IDW Spatial Interpolation. Interpolasi data titik ke lokasi target.")]
    fn spatial_interpolation_idw(&self, Parameters(p): Parameters<IdwParam>) -> String {
        let points: Vec<(f64, f64, f64)> = p.points.iter().map(|pt| (pt[0], pt[1], pt[2])).collect();
        tools::processing::interpolation::idw(&points, p.target_x, p.target_y, p.power.unwrap_or(2.0))
    }

    #[tool(description = "3D Terrain Visualization dari DEM GeoTIFF. Render surface 3D dengan color map elevasi.")]
    fn terrain_3d(&self, Parameters(p): Parameters<Terrain3dParam>) -> String {
        tools::processing::terrain3d::render(&p.dem_path, &p.output_path, &p.title, p.exaggeration.unwrap_or(2.0))
    }

    #[tool(description = "4D Terrain Rotation Animation (GIF). Rotasi 360° dari terrain 3D — simulasi perspektif temporal.")]
    fn terrain_4d_rotation(&self, Parameters(p): Parameters<Terrain3dParam>) -> String {
        tools::processing::viz4d::terrain_rotation(&p.dem_path, &p.output_path, &p.title, p.exaggeration.unwrap_or(2.0), 36)
    }

    #[tool(description = "4D Time Series Animation (GIF). Animasi data lingkungan berkembang seiring waktu. values: comma-separated, labels: comma-separated.")]
    fn timeseries_4d(&self, Parameters(p): Parameters<Timeseries4dParam>) -> String {
        tools::processing::viz4d::timeseries_animation(&p.values, &p.labels.clone().unwrap_or_default(), &p.output_path, &p.title, &p.ylabel.clone().unwrap_or("Value".into()))
    }

    #[tool(description = "3D Flood Simulation: terrain + genangan air pada level tertentu. Menghitung area genangan & kedalaman.")]
    fn flood_3d(&self, Parameters(p): Parameters<Flood3dParam>) -> String {
        tools::processing::flood_sim::flood_3d(&p.dem_path, &p.output_path, p.water_level_m, &p.title, p.exaggeration.unwrap_or(2.0))
    }

    #[tool(description = "4D Flood Animation (GIF): simulasi kenaikan level air dari start ke end. Temporal flood inundation model.")]
    fn flood_4d(&self, Parameters(p): Parameters<Flood4dParam>) -> String {
        tools::processing::flood_sim::flood_4d(&p.dem_path, &p.output_path, p.water_start_m, p.water_end_m, p.steps.unwrap_or(15), &p.title, p.exaggeration.unwrap_or(2.0))
    }

    // =======================================
    // AIR QUALITY DISPERSION MODELING
    // =======================================

    #[tool(description = "Stability Class (Turner 1970). Estimasi kelas Pasquill-Gifford dari data met. solar_radiation: strong/moderate/slight/night")]
    fn stability_class(&self, Parameters(p): Parameters<StabilityParam>) -> String {
        tools::airquality::stability::estimate(p.wind_speed_ms, &p.solar_radiation, p.cloud_cover_eighths)
    }

    #[tool(description = "Briggs Plume Rise. Hitung effective stack height. Ref: Briggs (1969-1975), AERMOD.")]
    fn plume_rise(&self, Parameters(p): Parameters<PlumeRiseParam>) -> String {
        tools::airquality::plume_rise::calculate(p.stack_height_m, p.exit_velocity_ms, p.exit_temp_k, p.ambient_temp_k, p.wind_speed_ms)
    }

    #[tool(description = "2D Air Dispersion Contour Map (PNG). Multi-source Gaussian plume grid. sources: JSON [{Q_gs,H_m,x_m,y_m}]")]
    fn dispersion_2d(&self, Parameters(p): Parameters<Dispersion2dParam>) -> String {
        tools::airquality::dispersion::render_2d(&p.sources_json, p.wind_speed, p.wind_dir, &p.stability, &p.output_path, &p.title, p.grid_size.unwrap_or(5000))
    }

    #[tool(description = "3D Air Dispersion Plume Visualization (PNG). 3D surface plot konsentrasi polutan.")]
    fn dispersion_3d(&self, Parameters(p): Parameters<Dispersion2dParam>) -> String {
        tools::airquality::dispersion::render_3d(&p.sources_json, p.wind_speed, p.wind_dir, &p.stability, &p.output_path, &p.title, p.grid_size.unwrap_or(5000))
    }

    #[tool(description = "4D Air Dispersion Animation (GIF). Simulasi perubahan arah/kecepatan angin temporal. wind_speeds & wind_dirs: comma-separated.")]
    fn dispersion_4d(&self, Parameters(p): Parameters<Dispersion4dParam>) -> String {
        tools::airquality::dispersion::render_4d(&p.sources_json, &p.wind_speeds, &p.wind_dirs, &p.stability, &p.output_path, &p.title, p.grid_size.unwrap_or(5000))
    }

    // =======================================
    // OCEAN MODELING 2D/3D/4D
    // =======================================

    #[tool(description = "3D Bathymetry: Visualisasi relief dasar laut. Input: lat, lon pusat area.")]
    fn ocean_bathymetry_3d(&self, Parameters(p): Parameters<OceanBathyParam>) -> String {
        tools::ocean_modeling::ocean_viz::bathymetry_3d(p.lat, p.lon, &p.output_path, &p.title)
    }

    #[tool(description = "2D Ocean Current: Peta vector field arus laut berbasis angin (Ekman). Input: lat, lon, wind.")]
    fn ocean_current_2d(&self, Parameters(p): Parameters<OceanCurrentParam>) -> String {
        tools::ocean_modeling::ocean_viz::current_2d(p.lat, p.lon, p.wind_speed, p.wind_dir, &p.output_path, &p.title)
    }

    #[tool(description = "3D Thermal Mixing: Visualisasi mixing zone polusi termal PLTU di laut. Baku mutu: DeltaT max 3C.")]
    fn ocean_thermal_3d(&self, Parameters(p): Parameters<OceanThermalParam>) -> String {
        tools::ocean_modeling::ocean_viz::thermal_3d(p.discharge_temp, p.ambient_temp, &p.output_path, &p.title)
    }

    #[tool(description = "4D Marine Pollution: Animasi GIF Lagrangian particle tracking polutan di laut. current_speeds & current_dirs: comma-separated.")]
    fn ocean_pollution_4d(&self, Parameters(p): Parameters<OceanPollutionParam>) -> String {
        tools::ocean_modeling::ocean_viz::pollution_4d(&p.current_speeds, &p.current_dirs, &p.output_path, &p.title)
    }

    #[tool(description = "JONSWAP Wave Height: Hitung Hs dari angin, fetch, dan kedalaman. Ref: Hasselmann 1973.")]
    fn wave_jonswap(&self, Parameters(p): Parameters<WaveParam>) -> String {
        tools::ocean_modeling::wave::jonswap(p.wind_speed_ms, p.fetch_m, p.depth_m)
    }

    #[tool(description = "Coral Bleaching DHW: Degree Heating Weeks dari data SST mingguan. Ref: NOAA Coral Reef Watch.")]
    fn coral_bleaching_dhw(&self, Parameters(p): Parameters<CoralDhwParam>) -> String {
        let sst: Vec<f64> = p.sst_weekly.split(',').filter_map(|s| s.trim().parse().ok()).collect();
        tools::ocean_modeling::wave::coral_bleaching_dhw(&sst, p.sst_max_monthly_mean)
    }

    #[tool(description = "CERC Sediment Transport: Longshore transport rate. Ref: SPM 1984, USACE.")]
    fn sediment_transport_cerc(&self, Parameters(p): Parameters<SedimentParam>) -> String {
        tools::ocean_modeling::sediment::cerc_transport(p.hs_m, p.wave_angle_deg, p.beach_slope_deg)
    }

    #[tool(description = "Oil Spill Trajectory & Fate: Drift (3% wind + current) + evaporasi + spreading. oil_type: crude/diesel/gasoline/bunker.")]
    fn oil_spill_model(&self, Parameters(p): Parameters<OilSpillParam>) -> String {
        tools::ocean_modeling::oil_spill::simulate_4d(p.volume_m3, &p.oil_type, p.wind_speed, p.wind_dir, p.current_speed, p.current_dir, p.hours, &p.output_path)
    }
}

#[rmcp::tool_handler]
impl ServerHandler for EnvNtbServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build()
        )
        .with_instructions("Environmental AI MCP Server for NTB Indonesia. 40+ tools covering GIS, RS, ESG Analytics, Data Crawling, and Project Wrappers.")
    }
}
