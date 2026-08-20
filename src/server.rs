use std::sync::LazyLock;

use crate::validation;
use rmcp::{
    handler::server::router::tool::ToolRouter,
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_router, ServerHandler,
};

use crate::tools;
pub use crate::tools::physics_validator::ValidatorParam;
use crate::tools::advanced_physics::peatland_subsidence::{calculate_peatland_subsidence, PeatlandSubsidenceParam};
use crate::tools::advanced_physics::groundwater_pde::RichardsParam;
use crate::tools::airquality::stability::MoninObukhovParam;
use crate::tools::calculators::land_subsidence::ConsolidationParam;
use crate::tools::airquality::source_apportionment::PmfParam;
use crate::tools::advanced_physics::uq::{GlueParam, DreamParam};
use crate::tools::water::contaminant_transport_1d::AdrSorptionParam;
use crate::tools::advanced_physics::coupled_swe_richards::CoupledParam;
use crate::tools::waste::hpal_tailings::{evaluate_hpal_tailings, HpalTailingsParam};

// Calculator & Compliance Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RusleParam {
    pub r_input: Option<f64>,
    pub rain_mm_yr: Option<f64>,
    pub k: f64,
    pub ls: f64,
    pub c: f64,
    pub p: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ScsCnParam {
    pub rainfall_mm: f64,
    pub cn: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PenmanParam {
    pub t_mean_c: f64,
    pub rh_pct: f64,
    pub wind_ms: f64,
    pub rn_mj: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StreeterPhelpsParam {
    pub k1: f64,
    pub k2: f64,
    pub l0: f64,
    pub d0: f64,
    pub velocity_ms: f64,
    pub distance_km: f64,
    pub temp_c: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DoSatParam {
    pub water_temp_c: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaterBalanceParam {
    pub p_mm: f64,
    pub et_mm: f64,
    pub q_mm: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GaussianParam {
    pub emission_gs: f64,
    pub wind_ms: f64,
    pub stack_height_m: f64,
    pub distance_m: f64,
    pub stability_class: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NoiseParam {
    pub source_db: f64,
    pub distance_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandfillParam {
    pub waste_ton: f64,
    pub years_open: u32,
    pub k_decay: f64,
    pub l0_potential: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MaturityParam {
    pub requested_level: String,
    pub availability: crate::honesty::DataAvailability,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ComputationParam {
    pub record: crate::computation::ComputationRecord,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CalibrateValidateParam {
    pub model_name: String,
    pub predicted: Vec<f64>,
    pub observed: Vec<f64>,
    pub unit: String,
    pub train_fraction: Option<f64>,
    pub confidence_level: Option<f64>,
    /// Point estimate the prediction interval is attached to. Defaults to the
    /// mean of the test-partition predictions.
    pub point_estimate: Option<f64>,
    pub availability: Option<crate::honesty::DataAvailability>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SwmmCouplingParam {
    pub inp_path: String,
    pub dem: Vec<Vec<f64>>,
    pub dx_m: f64,
    pub manning_n: f64,
    pub duration_s: f64,
    pub dt_max_s: f64,
    pub node_mapping: Vec<crate::coupling::NodeGridMapping>,
    pub timeout_secs: Option<u64>,
    pub mass_tolerance_pct: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SolidWasteParam {
    pub population: u64,
    pub generation_rate_kg: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProperParam {
    pub has_izin: bool,
    pub compliance_pct: f64,
    pub beyond_compliance: bool,
    pub community_dev: bool,
    pub circular_economy: bool,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IklhParam {
    pub ika: f64,
    pub iku: f64,
    pub iktl: f64,
}

// Fase 1+2 Calculator Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WastewaterParam {
    pub q_m3d: f64,
    pub bod_influent: f64,
    pub bod_target: f64,
    pub temp_c: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PeatlandParam {
    pub water_table_depth_cm: f64,
    pub area_ha: f64,
    pub years: u32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MangroveNdmiParam {
    pub nir_b8a: f64,
    pub swir_b11: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TsunamiParam {
    pub depth_m: f64,
    pub distance_km: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HeatIndexParam {
    pub temp_c: f64,
    pub rh_pct: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EutrophicationParam {
    pub secchi_depth_m: Option<f64>,
    pub chlorophyll_ugl: Option<f64>,
    pub total_phosphorus_ugl: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SoilTextureParam {
    pub sand_pct: f64,
    pub silt_pct: f64,
    pub clay_pct: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EflowParam {
    pub maf_m3s: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdfParam {
    pub r24_mm: f64,
    pub duration_hours: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RainwaterParam {
    pub roof_area_m2: f64,
    pub rainfall_mm: f64,
    pub runoff_coeff: f64,
    pub demand_liters_day: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FootprintParam {
    pub electricity_kwh: f64,
    pub vehicle_km: f64,
    pub meat_kg_week: f64,
    pub waste_kg_day: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LcaParam {
    pub material: String,
    pub mass_kg: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UvParam {
    pub solar_zenith_deg: f64,
    pub altitude_m: f64,
    pub ozone_du: f64,
    pub cloud_cover_pct: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanAcidParam {
    pub ph: f64,
    pub pco2_uatm: f64,
    pub temp_c: f64,
    pub salinity_psu: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SubsidenceParam {
    pub clay_thickness_m: f64,
    pub delta_stress_kpa: f64,
    pub cc: f64,
    pub e0: f64,
    pub sigma0_kpa: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ThermalParam {
    pub q_river_m3s: f64,
    pub t_river_c: f64,
    pub q_discharge_m3s: f64,
    pub t_discharge_c: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SlrParam {
    pub elevation_m: f64,
    pub slr_m: f64,
    pub storm_surge_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WteParam {
    pub waste_ton_day: f64,
    pub moisture_pct: f64,
    pub organic_pct: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AcidRainParam {
    pub so2_ugm3: f64,
    pub nox_ugm3: f64,
    pub rainfall_mm_yr: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MicroplasticParam {
    pub water_type: String,
    pub particles_per_liter: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MangroveCarbonParam {
    pub dbh_cm: f64,
    pub wood_density: f64,
    pub trees_per_ha: f64,
}

// Processing Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PdfReportParam {
    pub title: String,
    pub sections_json: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GeotiffCropParam {
    pub input_path: String,
    pub output_path: String,
    pub bbox: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WatershedParam {
    pub dem_path: String,
    pub pour_x: f64,
    pub pour_y: f64,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdwParam {
    pub points: Vec<Vec<f64>>,
    pub target_x: f64,
    pub target_y: f64,
    pub power: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Terrain3dParam {
    pub dem_path: String,
    pub output_path: String,
    pub title: String,
    pub exaggeration: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Timeseries4dParam {
    pub values: String,
    pub output_path: String,
    pub title: String,
    pub labels: Option<String>,
    pub ylabel: Option<String>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Flood3dParam {
    pub dem_path: String,
    pub output_path: String,
    pub water_level_m: f64,
    pub title: String,
    pub exaggeration: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Flood4dParam {
    pub dem_path: String,
    pub output_path: String,
    pub water_start_m: f64,
    pub water_end_m: f64,
    pub steps: Option<u32>,
    pub title: String,
    pub exaggeration: Option<f64>,
}

// Air Quality Dispersion Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StabilityParam {
    pub wind_speed_ms: f64,
    pub solar_radiation: String,
    pub cloud_cover_eighths: u32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PlumeRiseParam {
    pub stack_height_m: f64,
    pub stack_diameter_m: f64,
    pub exit_velocity_ms: f64,
    pub exit_temp_k: f64,
    pub ambient_temp_k: f64,
    pub wind_speed_ms: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Dispersion2dParam {
    pub sources_json: String,
    pub wind_speed: f64,
    pub wind_dir: f64,
    pub stability: String,
    pub output_path: String,
    pub title: String,
    pub grid_size: Option<u32>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Dispersion4dParam {
    pub sources_json: String,
    pub wind_speeds: String,
    pub wind_dirs: String,
    pub stability: String,
    pub output_path: String,
    pub title: String,
    pub grid_size: Option<u32>,
}

// Coral & MPA Spatial Query Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CoralReefParam {
    #[schemars(description = "Latitude (opsional — jika diisi, tampilkan reef terdekat)")]
    pub lat: Option<f64>,
    #[schemars(description = "Longitude (opsional)")]
    pub lon: Option<f64>,
    #[schemars(description = "Jumlah reef terdekat yang ditampilkan (default 5)")]
    pub n: Option<usize>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MpaParam {
    #[schemars(description = "Latitude (opsional — jika diisi, tampilkan MPA terdekat)")]
    pub lat: Option<f64>,
    #[schemars(description = "Longitude (opsional)")]
    pub lon: Option<f64>,
    #[schemars(description = "Jumlah MPA terdekat (default 5)")]
    pub n: Option<usize>,
}

// Ocean Modeling Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanBathyParam {
    pub lat: f64,
    pub lon: f64,
    pub output_path: String,
    pub title: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanCurrentParam {
    pub lat: f64,
    pub lon: f64,
    pub wind_speed: f64,
    pub wind_dir: f64,
    pub output_path: String,
    pub title: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanThermalParam {
    pub discharge_temp: f64,
    pub ambient_temp: f64,
    pub output_path: String,
    pub title: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OceanPollutionParam {
    pub current_speeds: String,
    pub current_dirs: String,
    pub output_path: String,
    pub title: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaveParam {
    pub wind_speed_ms: f64,
    pub fetch_m: f64,
    pub depth_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CoralDhwParam {
    pub sst_weekly: String,
    pub sst_max_monthly_mean: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SedimentParam {
    pub hs_m: f64,
    pub wave_angle_deg: f64,
    pub beach_slope_deg: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OilSpillParam {
    pub volume_m3: f64,
    pub oil_type: String,
    pub wind_speed: f64,
    pub wind_dir: f64,
    pub current_speed: f64,
    pub current_dir: f64,
    pub hours: u32,
    pub output_path: String,
}

// Advanced Physics Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FluxDivergenceParam {
    pub grid_data_json: String,
    pub u_wind: f64,
    pub v_wind: f64,
    pub dx_meters: f64,
    pub dy_meters: f64,
    pub lifetime_hours: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GroundwaterPdeParam {
    pub h_initial_json: String,
    pub diffusivity_d: f64,
    pub dx_meters: f64,
    pub dy_meters: f64,
    pub time_steps: u32,
    pub dt_seconds: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BayesianSensorParam {
    pub prior_particles_json: String,
    pub sensor_reading: f64,
    pub sensor_noise_std: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UhiParam {
    pub albedo_urban: f64,
    pub sky_view_factor: f64,
    pub solar_insolation_w: f64,
    pub ambient_temp_c: f64,
}

// ====== GOD TIER: Previously Unregistered Tool Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BiodiversityCalcParam {
    #[schemars(description = "JSON array jumlah individu per spesies, e.g. [45, 23, 12, 8, 5]")]
    pub species_counts_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CompostingParam {
    #[schemars(
        description = "JSON array [[name, mass_kg, c_pct, n_pct], ...], e.g. [[\"Serbuk Gergaji\", 100, 50, 0.1], [\"Kotoran Ayam\", 50, 30, 3.0]]"
    )]
    pub materials_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FloodFreqParam {
    #[schemars(
        description = "JSON array data debit puncak tahunan (minimal 10 tahun), e.g. [120, 145, 98, ...]"
    )]
    pub data_json: String,
    #[schemars(description = "Return period (tahun), e.g. 25, 50, 100")]
    pub return_period: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AmdCalcParam {
    #[schemars(description = "Total Sulfur (%)")]
    pub sulfur_pct: f64,
    #[schemars(description = "Acid Neutralizing Capacity (kg H2SO4/ton)")]
    pub anc_kg_h2so4_t: f64,
    #[schemars(description = "NAG pH (optional)")]
    pub nag_ph: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TransportEmParam {
    #[schemars(description = "Tipe BBM: bensin/solar/avtur")]
    pub fuel_type: String,
    #[schemars(description = "Volume (Liter)")]
    pub liters: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IpParam {
    #[schemars(
        description = "JSON array: [{\"name\":\"BOD\",\"ci\":4.0,\"lij\":2.0,\"is_do\":false}, ...]"
    )]
    pub data_json: String,
    #[schemars(description = "Suhu air (°C) untuk koreksi DO saturasi")]
    pub temp_c: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StoretParam {
    #[schemars(
        description = "JSON array: [{\"name\":\"BOD\",\"type\":\"kimia\",\"samples\":[{\"value\":4.0,\"limit\":2.0}]}, ...]"
    )]
    pub data_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SpplParam {
    #[schemars(description = "Jenis kegiatan usaha")]
    pub kegiatan: String,
    #[schemars(description = "Apakah wajib AMDAL?")]
    pub is_wajib_amdal: bool,
    #[schemars(description = "Apakah wajib UKL-UPL?")]
    pub is_wajib_uklupl: bool,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuLautParam {
    #[schemars(
        description = "Parameter: pH/DO/BOD5/ammonia/fosfat/nitrat/sulfida/minyak_lemak/surfaktan/fenol/sianida/Hg/Cr6/As/Cd/Cu/Pb/Zn/Ni/coliform/suhu_delta"
    )]
    pub parameter: String,
    #[schemars(
        description = "Nilai terukur (mg/L, MPN/100mL untuk coliform, °C untuk suhu_delta)"
    )]
    pub concentration: f64,
    #[schemars(description = "Peruntukan: wisata/biota/pelabuhan")]
    pub peruntukan: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TimelapseParam {
    pub lat: f64,
    pub lon: f64,
    #[schemars(description = "Buffer radius (km)")]
    pub buffer_km: f64,
    pub start_year: u32,
    pub end_year: u32,
    #[schemars(description = "Sensor: optik_s2 atau radar_s1")]
    pub sensor: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HyperspectralParam {
    pub lat: f64,
    pub lon: f64,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidatorV3Param {
    #[schemars(description = "Gas: NO2, CH4, SO2, CO")]
    pub gas_type: String,
    #[schemars(description = "Konsentrasi (µg/m³)")]
    pub concentration: f64,
    #[schemars(description = "Waktu: day/night/siang/malam")]
    pub time_of_day: String,
    #[schemars(description = "Tipe fluida: water/mud/debris")]
    pub fluid_type: String,
    #[schemars(description = "Sudut lereng (derajat)")]
    pub slope_angle_deg: f64,
    #[schemars(description = "Kedalaman material (m)")]
    pub depth_m: f64,
}

// ====== GOD TIER: New Compliance Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuUdaraParam {
    #[schemars(description = "Parameter: SO2/CO/NO2/O3/PM10/PM2.5/Pb/TSP/HC")]
    pub parameter: String,
    #[schemars(description = "Konsentrasi terukur (µg/m³)")]
    pub concentration: f64,
    #[schemars(description = "Waktu pengukuran: 1_hour/8_hour/24_hour/annual")]
    pub averaging_time: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuEmisiParam {
    #[schemars(
        description = "Industri: pltu_batubara/semen/smelter/kimia/pembangkit_gas/incinerator"
    )]
    pub industry: String,
    #[schemars(description = "Parameter: TSP/SO2/NO2/CO/opacity")]
    pub parameter: String,
    #[schemars(description = "Konsentrasi terukur (mg/Nm³ atau % untuk opacity)")]
    pub concentration: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuAirLimbahParam {
    #[schemars(
        description = "Industri: tekstil/sawit/karet/tapioka/gula/pulp_kertas/farmasi/electroplating/rumah_sakit/hotel/peternakan"
    )]
    pub industry: String,
    #[schemars(description = "Parameter: BOD/COD/TSS/pH/oil_grease/phenol/Cr6/NH3N")]
    pub parameter: String,
    #[schemars(description = "Konsentrasi terukur (mg/L)")]
    pub concentration: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuDomestikParam {
    #[schemars(description = "Parameter: pH/BOD/COD/TSS/oil_grease/ammonia/total_coliform")]
    pub parameter: String,
    #[schemars(description = "Konsentrasi terukur (mg/L, atau per 100mL untuk coliform)")]
    pub concentration: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuKebisinganParam {
    #[schemars(
        description = "Zona: perumahan/perdagangan/perkantoran/industri/rumah_sakit/sekolah/ibadah/ruang_terbuka_hijau"
    )]
    pub zone: String,
    #[schemars(description = "Tingkat kebisingan terukur (dB(A))")]
    pub measured_db: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuGetaranParam {
    #[schemars(description = "Zona: pemukiman/kantor/industri/rumah_sakit")]
    pub zone: String,
    #[schemars(description = "Kecepatan getaran (mm/s)")]
    pub vibration_mm_s: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuKebauanParam {
    #[schemars(description = "Kimia: H2S/NH3/methyl_mercaptan/styrene")]
    pub chemical: String,
    #[schemars(description = "Konsentrasi (ppm)")]
    pub concentration_ppm: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IspuParam {
    #[schemars(description = "PM10 (µg/m³)")]
    pub pm10: Option<f64>,
    #[schemars(description = "PM2.5 (µg/m³)")]
    pub pm25: Option<f64>,
    #[schemars(description = "SO2 (µg/m³)")]
    pub so2: Option<f64>,
    #[schemars(description = "CO (µg/m³)")]
    pub co: Option<f64>,
    #[schemars(description = "O3 (µg/m³)")]
    pub o3: Option<f64>,
    #[schemars(description = "NO2 (µg/m³)")]
    pub no2: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RiskClassParam {
    #[schemars(
        description = "Sektor: pertambangan/industri/energi/pertanian/kehutanan/transportasi/pariwisata"
    )]
    pub sector: String,
    #[schemars(description = "Deskripsi skala kegiatan (misal: 'luas 200 ha')")]
    pub scale_description: String,
    #[schemars(description = "Apakah menghasilkan limbah B3?")]
    pub has_hazardous_waste: bool,
    #[schemars(description = "Apakah dekat kawasan lindung?")]
    pub near_protected_area: bool,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DayaDukungParam {
    #[schemars(description = "Pendekatan: population/water/food")]
    pub approach: String,
    #[schemars(description = "Luas wilayah (ha)")]
    pub area_ha: f64,
    #[schemars(description = "Jumlah penduduk")]
    pub population: f64,
    pub water_supply_m3_yr: Option<f64>,
    pub water_demand_m3_yr: Option<f64>,
    pub food_production_ton_yr: Option<f64>,
    pub food_demand_ton_yr: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DayaTampungParam {
    #[schemars(description = "Debit sungai (m³/s)")]
    pub q_river_m3s: f64,
    #[schemars(description = "Konsentrasi hulu (mg/L)")]
    pub c_upstream_mgl: f64,
    #[schemars(description = "Baku mutu (mg/L)")]
    pub c_standard_mgl: f64,
    #[schemars(description = "Debit limbah (m³/s)")]
    pub q_waste_m3s: f64,
    #[schemars(description = "Konsentrasi limbah (mg/L)")]
    pub c_waste_mgl: f64,
    #[schemars(description = "Nama parameter (BOD/COD/TSS/dll)")]
    pub parameter: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GhgInventoryParam {
    #[schemars(description = "Sektor: energy/ippu/afolu/waste")]
    pub sector: String,
    #[schemars(
        description = "Aktivitas: electricity_kwh/diesel_liter/gasoline_liter/lpg_kg/cement_ton/deforestation_ha/rice_paddy_ha/landfill_ton"
    )]
    pub activity: String,
    #[schemars(description = "Jumlah (sesuai unit aktivitas)")]
    pub amount: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IklhSubParam {
    #[schemars(description = "Tipe: ika/iku/iktl/ikal")]
    pub sub_type: String,
    #[schemars(
        description = "JSON data: array angka IP/ISPU, atau {\"forest_cover_pct\":X,\"target_pct\":Y}, atau JSON params laut"
    )]
    pub data_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AmdalScreeningParam {
    #[schemars(
        description = "Sektor: pertambangan/kehutanan/industri/energi/transportasi/pariwisata/pertanian/perikanan/permukiman"
    )]
    pub sector: String,
    #[schemars(description = "Jenis kegiatan (misal: eksploitasi mineral logam)")]
    pub activity: String,
    #[schemars(description = "Nilai skala (angka)")]
    pub scale_value: f64,
    #[schemars(description = "Unit skala: ha/MW/km/ton_hari/kamar/unit")]
    pub scale_unit: String,
}

// ====== GOD TIER: AMDAL Generator Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct KaAndalParam {
    #[schemars(description = "Nama proyek")]
    pub project_name: String,
    #[schemars(description = "Lokasi proyek")]
    pub location: String,
    #[schemars(description = "Jenis proyek (pertambangan/industri/infrastruktur/energi)")]
    pub project_type: String,
    #[schemars(
        description = "JSON rona lingkungan awal: {\"topografi\":\"...\",\"iklim\":\"...\",\"flora_fauna\":\"...\"}"
    )]
    pub rona_json: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AndalParam {
    pub project_name: String,
    pub location: String,
    #[schemars(
        description = "JSON dampak: [{\"component\":\"...\",\"impact\":\"...\",\"magnitude\":-7,\"importance\":8,\"duration\":\"permanen\"}]"
    )]
    pub impacts_json: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RklRplParam {
    pub project_name: String,
    pub location: String,
    #[schemars(
        description = "JSON rencana: [{\"impact\":\"...\",\"management\":\"...\",\"monitoring\":\"...\",\"institution\":\"...\",\"location\":\"...\",\"period\":\"...\"}]"
    )]
    pub management_json: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UklUplParam {
    pub project_name: String,
    pub location: String,
    #[schemars(description = "JSON dampak dan pengelolaan")]
    pub impacts_json: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct KlhsParam {
    #[schemars(description = "Nama kebijakan/rencana/program")]
    pub policy_name: String,
    #[schemars(description = "JSON data daya dukung")]
    pub daya_dukung_json: String,
    pub output_path: String,
}

// ====== GOD TIER: Noise Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Noise2dParam {
    #[schemars(
        description = "JSON sumber: [{\"x_m\":0,\"y_m\":0,\"power_db\":95,\"type\":\"point\"}]"
    )]
    pub sources_json: String,
    pub output_path: String,
    pub title: String,
    #[schemars(description = "Ukuran grid (m), default 500")]
    pub grid_size: Option<u32>,
    #[schemars(
        description = "JSON barrier: [{\"x1\":100,\"y1\":-50,\"x2\":100,\"y2\":50,\"height_m\":3,\"il_db\":10}] atau \"[]\""
    )]
    pub barrier_json: Option<String>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Noise3dParam {
    pub sources_json: String,
    pub output_path: String,
    pub title: String,
    pub grid_size: Option<u32>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NoiseComplianceParam {
    #[schemars(description = "Zona: perumahan/industri/rumah_sakit/sekolah")]
    pub zone: String,
    #[schemars(description = "Kebisingan terukur (dB)")]
    pub measured_db: f64,
    #[schemars(description = "Jarak dari sumber (m)")]
    pub distance_m: f64,
    #[schemars(description = "Level daya sumber (dB)")]
    pub source_db: f64,
}

// ====== GOD TIER: Biodiversity & Social Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IucnCheckParam {
    pub lat: f64,
    pub lon: f64,
    #[schemars(description = "Radius pencarian (km)")]
    pub radius_km: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProtectedSpeciesParam {
    #[schemars(description = "Nama spesies (scientific atau Indonesia)")]
    pub species_name: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProtectedByProvinceParam {
    #[schemars(description = "Nama provinsi: NTB, Jawa Timur, Kalimantan Timur, dll")]
    pub province: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SocialImpactParam {
    #[schemars(
        description = "JSON dampak sosial: [{\"component\":\"ekonomi\",\"impact\":\"kehilangan lahan\",\"magnitude\":-7,\"importance\":8}]"
    )]
    pub impacts_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HealthImpactParam {
    pub population: u64,
    #[schemars(description = "Polutan: PM2.5/NO2/SO2/CO/benzene")]
    pub pollutant: String,
    #[schemars(description = "Konsentrasi (µg/m³)")]
    pub concentration: f64,
    #[schemars(description = "Durasi paparan (jam/hari)")]
    pub exposure_hours: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValuationParam {
    #[schemars(
        description = "Metode: replacement_cost/travel_cost/hedonic/damage_cost/benefit_transfer"
    )]
    pub method: String,
    #[schemars(description = "JSON parameter sesuai metode")]
    pub params_json: String,
}

// ====== GOD TIER: Data Source Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IspuKlhkParam {
    #[schemars(
        description = "Nama kota: Jakarta/Surabaya/Bandung/Semarang/Medan/Makassar/Denpasar/Mataram/dll"
    )]
    pub kota: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SipongiParam {
    #[schemars(description = "Provinsi: Riau/Kalimantan Barat/Sumatera Selatan/dll")]
    pub province: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BmkgOpenParam {
    #[schemars(description = "ID Stasiun BMKG")]
    pub station_id: String,
    #[schemars(description = "Parameter: rainfall/temperature/humidity/wind/sunshine")]
    pub parameter: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OsmPoiParam {
    pub lat: f64,
    pub lon: f64,
    #[schemars(description = "Radius pencarian (m)")]
    pub radius_m: f64,
    #[schemars(description = "Tipe POI: hospital/school/residential/worship/market/river/forest")]
    pub poi_type: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ElevationParam {
    pub lat1: f64,
    pub lon1: f64,
    pub lat2: f64,
    pub lon2: f64,
    #[schemars(description = "Jumlah titik interpolasi (default 20)")]
    pub num_points: Option<u32>,
}

// ====== GOD TIER: SAR Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SarFloodParam {
    pub lat: f64,
    pub lon: f64,
    #[schemars(description = "Buffer (km)")]
    pub buffer_km: f64,
    #[schemars(description = "Tanggal sebelum banjir (YYYY-MM-DD)")]
    pub pre_date: String,
    #[schemars(description = "Tanggal setelah banjir (YYYY-MM-DD)")]
    pub post_date: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SarDeforestParam {
    pub lat: f64,
    pub lon: f64,
    pub buffer_km: f64,
    pub start_date: String,
    pub end_date: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SarLocalParam {
    #[schemars(description = "Path file Sentinel-1 (.zip atau .SAFE)")]
    pub input_path: String,
    pub output_path: String,
    #[schemars(description = "Tipe analisis: coherence/backscatter/interferogram")]
    pub analysis_type: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SarSubsidenceParam {
    pub lat: f64,
    pub lon: f64,
    pub buffer_km: f64,
    pub start_date: String,
    pub end_date: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BurnedAreaParam {
    pub lat: f64,
    pub lon: f64,
    pub buffer_km: f64,
    #[schemars(description = "Tanggal kebakaran (YYYY-MM-DD)")]
    pub fire_date: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MangroveExtentParam {
    pub lat: f64,
    pub lon: f64,
    pub buffer_km: f64,
    pub output_path: String,
}

// ====== GOD TIER PHASE 2: Water Engineering Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DisinfectantType {
    Chlorine,
    Ozone,
    Uv,
    Chloramine,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CtDisinfectionParam {
    #[schemars(description = "Disinfektan: chlorine/ozone/uv/chloramine")]
    pub disinfectant: DisinfectantType,
    #[schemars(description = "Konsentrasi (mg/L) atau dosis UV (mJ/cm²)")]
    pub concentration_mgl: f64,
    #[schemars(description = "Waktu kontak (menit)")]
    pub contact_time_min: f64,
    #[schemars(description = "Patogen target: giardia/virus/crypto")]
    pub target_pathogen: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DarcyParam {
    #[schemars(description = "Konduktivitas hidraulik K (m/s)")]
    pub k_ms: f64,
    #[schemars(description = "Gradien hidraulik (i = Δh/L)")]
    pub gradient: f64,
    #[schemars(description = "Luas penampang (m²)")]
    pub area_m2: f64,
    #[schemars(description = "Porositas (0-1)")]
    pub porosity: f64,
    #[schemars(description = "Jarak transport (m)")]
    pub distance_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TheisParam {
    #[schemars(description = "Debit pompa (m³/s)")]
    pub q_m3s: f64,
    #[schemars(description = "Transmisivitas (m²/s)")]
    pub transmissivity_m2s: f64,
    #[schemars(description = "Storativity (dimensionless)")]
    pub storativity: f64,
    #[schemars(description = "Jarak dari sumur (m)")]
    pub r_m: f64,
    #[schemars(description = "Waktu pemompaan (detik)")]
    pub t_s: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HazenWilliamsParam {
    #[schemars(description = "Debit (m³/s)")]
    pub q_m3s: f64,
    #[schemars(description = "Panjang pipa (m)")]
    pub length_m: f64,
    #[schemars(description = "Diameter pipa (m)")]
    pub diameter_m: f64,
    #[schemars(
        description = "Koefisien C: PVC(150)/PE(140)/steel_new(120)/cast_iron(100)/concrete(110)"
    )]
    pub c_coeff: f64,
    #[schemars(description = "Sertakan minor losses (10%)")]
    pub include_minor_losses: bool,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PumpSizingParam {
    #[schemars(description = "Debit (m³/s)")]
    pub q_m3s: f64,
    pub static_lift_m: f64,
    pub friction_loss_m: f64,
    pub velocity_head_m: f64,
    pub pressure_head_m: f64,
    #[schemars(description = "Efisiensi pompa (0-1, typical 0.6-0.85)")]
    pub efficiency: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SedimentationParam {
    #[schemars(description = "Debit desain (m³/hari)")]
    pub q_m3d: f64,
    #[schemars(description = "Tipe: primary/secondary")]
    pub tank_type: String,
    #[schemars(description = "Bentuk: rectangular/circular")]
    pub tank_shape: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UasbParam {
    pub q_m3d: f64,
    #[schemars(description = "COD influent (mg/L)")]
    pub cod_in_mgl: f64,
    #[schemars(description = "Target COD effluent (mg/L)")]
    pub cod_eff_target: f64,
    pub temperature_c: f64,
    #[schemars(description = "Tipe limbah: pome/tapioka/karet/domestik")]
    pub waste_type: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TricklingFilterParam {
    pub q_m3d: f64,
    #[schemars(description = "BOD influent (mg/L)")]
    pub bod_in: f64,
    #[schemars(description = "BOD target (mg/L)")]
    pub bod_target: f64,
    #[schemars(description = "Kedalaman media (m), typical 1.5-3.0")]
    pub media_depth_m: f64,
    #[schemars(description = "Rasio resirkulasi (0-3)")]
    pub recirculation_ratio: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ConstructedWetlandParam {
    pub q_m3d: f64,
    #[schemars(description = "Parameter: BOD/TSS/NH4N")]
    pub parameter: String,
    #[schemars(description = "Konsentrasi influent (mg/L)")]
    pub ci_mgl: f64,
    #[schemars(description = "Target effluent (mg/L)")]
    pub ce_target: f64,
    pub temp_c: f64,
    #[schemars(description = "Tipe: FWS (free water surface) / HSSF (horizontal subsurface flow)")]
    pub wetland_type: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AnaerobicDigestionParam {
    pub q_m3d: f64,
    #[schemars(description = "Konsentrasi VS (kg/m³)")]
    pub vs_concentration_kgm3: f64,
    #[schemars(description = "% destruksi VS (50-80%)")]
    pub vs_destruction_pct: f64,
    pub temperature_c: f64,
    #[schemars(description = "Substrat: sapi/babi/ayam/pome")]
    pub substrate: String,
}
// ====== Chemistry Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FirstOrderParam {
    pub c0: f64,
    pub k: f64,
    pub t: f64,
    #[schemars(description = "Unit: s/min/hr/day")]
    pub time_unit: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IsothermParam {
    #[schemars(description = "Model: freundlich/langmuir")]
    pub model: String,
    #[schemars(description = "Konsentrasi kesetimbangan Ce (mg/L)")]
    pub ce: f64,
    pub kf: f64,
    pub n_exp: f64,
    pub qmax: f64,
    pub kl: f64,
    #[schemars(description = "Volume larutan (L)")]
    pub volume_l: f64,
    #[schemars(description = "Konsentrasi awal (mg/L)")]
    pub c0: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HenrysLawParam {
    #[schemars(description = "Senyawa: benzene/toluene/TCE/PCE/chloroform/methane/CO2/O2/NH3")]
    pub compound: String,
    pub concentration_mgl: f64,
    pub temperature_c: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NernstParam {
    #[schemars(description = "Setengah reaksi: O2_H2O/Fe3_Fe2/MnO4_Mn2/Cr2O7_Cr3/NO3_N2")]
    pub half_reaction: String,
    pub temperature_c: f64,
    pub log_q: f64,
    pub n_electrons: u32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PartitionParam {
    #[schemars(description = "Senyawa: benzene/toluene/naphthalene/phenol/atrazine/DDT/PCB")]
    pub compound: String,
    #[schemars(description = "Fraksi karbon organik tanah")]
    pub foc: f64,
    pub bulk_density_kgm3: f64,
    pub porosity: f64,
}
// ====== Hydrology Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RationalParam {
    #[schemars(description = "Koefisien limpasan C (0-1), atau isi 0 dan gunakan land_use")]
    pub c_coeff: f64,
    #[schemars(description = "Intensitas hujan (mm/jam)")]
    pub i_mm_hr: f64,
    #[schemars(description = "Luas DAS (ha)")]
    pub a_ha: f64,
    #[schemars(
        description = "Tipe lahan: hutan/sawah/perkebunan/permukiman_jarang/permukiman_padat/komersial/industri/jalan_aspal"
    )]
    pub land_use: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UnitHydrographParam {
    pub a_km2: f64,
    pub tc_hours: f64,
    pub d_hours: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MuskingumParam {
    #[schemars(description = "JSON: [[t1,Q1],[t2,Q2],...] inflow hydrograph")]
    pub inflow_json: String,
    pub k_hours: f64,
    #[schemars(description = "Weighting factor x (0-0.5)")]
    pub x: f64,
    pub dt_hours: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TocParam {
    #[schemars(description = "Metode: kirpich/bransby_williams/scs_lag")]
    pub method: String,
    #[schemars(description = "Panjang saluran (m)")]
    pub l_m: f64,
    #[schemars(description = "Kemiringan (m/m)")]
    pub s_slope: f64,
    pub a_km2: f64,
    pub cn: f64,
}
// ====== Waste Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandfillLinerParam {
    #[schemars(description = "Tipe: single_clay/composite/double_liner")]
    pub liner_type: String,
    pub area_m2: f64,
    pub head_on_liner_m: f64,
    pub k_clay: f64,
    pub clay_thickness_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LeachateParam {
    pub area_m2: f64,
    #[schemars(description = "JSON 12 nilai curah hujan bulanan (mm)")]
    pub monthly_rainfall_json: String,
    #[schemars(description = "JSON 12 nilai ET bulanan (mm)")]
    pub monthly_et_json: String,
    pub soil_storage_mm: f64,
    pub runoff_coeff: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandfillStabilityParam {
    pub slope_angle_deg: f64,
    pub height_m: f64,
    pub unit_weight_kn_m3: f64,
    pub cohesion_kpa: f64,
    pub friction_deg: f64,
    pub pore_pressure_ratio: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TclpParam {
    #[schemars(description = "JSON: [{\"name\":\"Pb\",\"concentration_mgl\":4.5}, ...]")]
    pub parameters_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WasteCompatParam {
    pub waste_a: String,
    pub waste_b: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct B3StorageParam {
    #[schemars(description = "Tipe: padat/cair/lumpur")]
    pub waste_type: String,
    pub volume_m3_per_month: f64,
    pub density_kg_m3: f64,
}
// ====== Radiation Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InverseSquareParam {
    pub dose_rate_at_d1: f64,
    pub d1_m: f64,
    pub d2_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ShieldingParam {
    pub initial_intensity: f64,
    #[schemars(description = "Material: lead/concrete/water/steel/earth")]
    pub material: String,
    pub thickness_cm: f64,
    #[schemars(description = "Sumber: Cs137/Co60/I131/Sr90/Ra226")]
    pub source: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DecayParam {
    #[schemars(description = "Isotop: Cs137/Co60/I131/Sr90/Ra226/C14/H3/Tc99m/U238")]
    pub isotope: String,
    pub initial_activity_bq: f64,
    pub time_elapsed: f64,
    #[schemars(description = "Unit: seconds/minutes/hours/days/years")]
    pub time_unit: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RadonParam {
    pub soil_radon_bq_m3: f64,
    pub floor_area_m2: f64,
    pub room_height_m: f64,
    pub ventilation_rate_ach: f64,
    #[schemars(description = "Tipe lantai: concrete_slab/basement/tanah/elevated")]
    pub floor_type: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NormParam {
    #[schemars(description = "Material: tin_slag/monazite/zircon/coal_ash/phosphogypsum/bauxite")]
    pub material: String,
    pub activity_bq_g: f64,
}
// ====== Health & Monitoring Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HhraParam {
    #[schemars(description = "Jalur: inhalation/ingestion/dermal")]
    pub exposure_route: String,
    pub concentration: f64,
    pub intake_rate: f64,
    pub exposure_freq_days: f64,
    pub exposure_dur_years: f64,
    pub body_weight_kg: f64,
    pub avg_time_years: f64,
    pub csf: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HqParam {
    #[schemars(
        description = "Kontaminan: arsenic/chromium_vi/cadmium/mercury/benzene/toluene/xylene/phenol/formaldehyde/ammonia"
    )]
    pub contaminant: String,
    #[schemars(description = "Jalur: oral/inhalation")]
    pub route: String,
    pub concentration: f64,
    pub intake_rate: f64,
    pub exposure_freq_days: f64,
    pub exposure_dur_years: f64,
    pub body_weight_kg: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ArklParam {
    #[schemars(
        description = "Kontaminan: arsenic/chromium_vi/cadmium/benzene/toluene/ammonia/dll"
    )]
    pub contaminant: String,
    #[schemars(description = "Jalur: oral/inhalation")]
    pub route: String,
    #[schemars(description = "Konsentrasi terukur (mg/kg/day untuk oral, mg/m³ untuk inhalasi)")]
    pub concentration: f64,
    #[schemars(description = "Tipe populasi: dewasa/anak")]
    pub population_type: String,
    #[schemars(description = "Skenario: residensial/okupasional/sekolah")]
    pub exposure_scenario: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SamplingParam {
    pub confidence_pct: f64,
    pub margin_error_pct: f64,
    pub std_deviation: f64,
    pub population_size: Option<u64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MannKendallParam {
    #[schemars(description = "JSON array data time-series (urut waktu)")]
    pub data_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QaqcParam {
    #[schemars(
        description = "JSON: [{\"sample\":\"S1\",\"value\":5.2,\"duplicate\":5.0,\"spike\":47.5,\"spike_amount\":50.0,\"blank\":0.02}]"
    )]
    pub data_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ColiformParam {
    pub initial_count_per_100ml: f64,
    pub temperature_c: f64,
    pub time_hours: f64,
    #[schemars(description = "Tipe air: freshwater/seawater/tropical")]
    pub water_type: String,
}
// ====== Ecological/Coastal Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BruunParam {
    pub sea_level_rise_m: f64,
    pub profile_length_m: f64,
    pub berm_height_m: f64,
    pub closure_depth_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CviParam {
    pub geomorphology: u32,
    pub shoreline_change_m_yr: f64,
    pub coastal_slope_pct: f64,
    pub slr_mm_yr: f64,
    pub mean_wave_height_m: f64,
    pub mean_tidal_range_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TrafficNoiseParam {
    pub vehicles_per_hour: f64,
    pub speed_kmh: f64,
    pub distance_m: f64,
    pub heavy_vehicle_pct: f64,
    pub gradient_pct: f64,
    #[schemars(description = "Tipe tanah: hard/soft")]
    pub ground_type: String,
    pub barrier_height_m: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BioretentionParam {
    pub q_design_m3s: f64,
    pub ksat_m_hr: f64,
    pub ponding_depth_m: f64,
    pub media_depth_m: f64,
    pub drain_time_hr: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaterFootprintParam {
    #[schemars(
        description = "Produk: rice/palm_oil/rubber/coffee/beef/chicken/cotton/paper/steel/cement"
    )]
    pub product: String,
    pub quantity: f64,
    #[schemars(description = "Unit: kg/ton/L")]
    pub unit: String,
}
// ====== Economics Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CbaParam {
    #[schemars(
        description = "JSON: [{\"year\":0,\"amount\":1e9,\"description\":\"Konstruksi\",\"recurring\":false}]. Set \"recurring\":true for annual items repeated from year to end of period."
    )]
    pub costs_json: String,
    #[schemars(
        description = "JSON: [{\"year\":1,\"amount\":2e8,\"description\":\"Revenue\",\"recurring\":true}]. Set \"recurring\":true for annual items repeated from year to end of period."
    )]
    pub benefits_json: String,
    pub discount_rate: f64,
    pub years: u32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MfaParam {
    pub inputs_json: String,
    pub outputs_json: String,
    pub stock_change: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Scope123Param {
    pub scope1_json: String,
    pub scope2_json: String,
    pub scope3_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CircularParam {
    pub mass_product_kg: f64,
    pub virgin_feedstock_pct: f64,
    pub recycled_input_pct: f64,
    pub reused_input_pct: f64,
    pub recycled_output_pct: f64,
    pub reused_output_pct: f64,
    pub product_lifetime_years: f64,
    pub industry_avg_lifetime: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExternalityParam {
    pub pollutant: String,
    pub amount: f64,
    #[schemars(description = "Unit: ton/kg")]
    pub unit: String,
    #[schemars(description = "Lokasi: urban/suburban/rural")]
    pub location_type: String,
}

// ====== GIS/RS Tool Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RasterBandMathParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Buffer radius (km)")]
    pub buffer_km: f64,
    #[schemars(description = "Index type: ndvi/ndwi/savi/evi/mndwi/ndbi/bsi")]
    pub index_type: String,
    #[schemars(description = "Start date YYYY-MM-DD")]
    pub start_date: String,
    #[schemars(description = "End date YYYY-MM-DD")]
    pub end_date: String,
    #[schemars(description = "Output GeoTIFF path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RasterBandMathLocalParam {
    #[schemars(description = "Input GeoTIFF path")]
    pub input_path: String,
    #[schemars(description = "Band math expression (e.g. '(b1-b2)/(b1+b2)')")]
    pub expression: String,
    #[schemars(description = "Output GeoTIFF path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DemGeeParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Buffer radius (km)")]
    pub buffer_km: f64,
    #[schemars(description = "Output path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ZonalStatsGeeParam {
    #[schemars(description = "GEE Image ID (e.g. USGS/SRTMGL1_003)")]
    pub image_id: String,
    #[schemars(description = "Band name (e.g. elevation)")]
    pub band: String,
    #[schemars(description = "GeoJSON polygon string (optional, use lat/lon/buffer if empty)")]
    pub geojson: Option<String>,
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Buffer radius (km)")]
    pub buffer_km: f64,
    #[schemars(description = "Output JSON path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ZonalStatsLocalParam {
    #[schemars(description = "Input raster path")]
    pub raster_path: String,
    #[schemars(description = "Input vector path (GeoJSON/Shapefile)")]
    pub vector_path: String,
    #[schemars(description = "Stats: comma-separated (min,max,mean,std,sum,count)")]
    pub stats: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandCoverClassifyParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Buffer radius (km)")]
    pub buffer_km: f64,
    #[schemars(description = "Start date YYYY-MM-DD")]
    pub start_date: String,
    #[schemars(description = "End date YYYY-MM-DD")]
    pub end_date: String,
    #[schemars(description = "Output path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandUseChangeParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Buffer radius (km)")]
    pub buffer_km: f64,
    #[schemars(description = "Period 1 start date YYYY-MM-DD")]
    pub d1_start: String,
    #[schemars(description = "Period 1 end date YYYY-MM-DD")]
    pub d1_end: String,
    #[schemars(description = "Period 2 start date YYYY-MM-DD")]
    pub d2_start: String,
    #[schemars(description = "Period 2 end date YYYY-MM-DD")]
    pub d2_end: String,
    #[schemars(description = "Output path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AccuracyAssessmentParam {
    #[schemars(
        description = "JSON array of predicted class labels, e.g. [\"forest\",\"water\",\"urban\"]"
    )]
    pub predicted_json: String,
    #[schemars(description = "JSON array of actual (ground truth) class labels")]
    pub actual_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BufferAnalysisParam {
    #[schemars(description = "GeoJSON string")]
    pub geojson: String,
    #[schemars(description = "Buffer distance (meters)")]
    pub distance_m: f64,
    #[schemars(description = "Output GeoJSON path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OverlayAnalysisParam {
    #[schemars(description = "GeoJSON A string")]
    pub geojson_a: String,
    #[schemars(description = "GeoJSON B string")]
    pub geojson_b: String,
    #[schemars(description = "Operation: intersection/union/difference/symmetric_difference")]
    pub operation: String,
    #[schemars(description = "Output GeoJSON path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SuitabilityAnalysisParam {
    #[schemars(description = "JSON criteria for suitability analysis")]
    pub criteria_json: String,
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Buffer radius (km)")]
    pub buffer_km: f64,
    #[schemars(description = "Output path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ViewshedAnalysisParam {
    #[schemars(description = "DEM GeoTIFF path")]
    pub dem_path: String,
    #[schemars(description = "Observer latitude")]
    pub observer_lat: f64,
    #[schemars(description = "Observer longitude")]
    pub observer_lon: f64,
    #[schemars(description = "Observer height above ground (m)")]
    pub observer_height_m: f64,
    #[schemars(description = "Max viewshed distance (m)")]
    pub max_distance_m: f64,
    #[schemars(description = "Output path")]
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CoordTransformV2Param {
    #[schemars(description = "X coordinate / Easting / Longitude")]
    pub x: f64,
    #[schemars(description = "Y coordinate / Northing / Latitude")]
    pub y: f64,
    #[schemars(description = "Source CRS EPSG code (e.g. 4326)")]
    pub from_epsg: u32,
    #[schemars(description = "Target CRS EPSG code (e.g. 32750)")]
    pub to_epsg: u32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Wgs84ToUtmParam {
    #[schemars(description = "Latitude (WGS84)")]
    pub lat: f64,
    #[schemars(description = "Longitude (WGS84)")]
    pub lon: f64,
}

// ====== RESEARCH-GRADE GIS/RS PARAMS ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OlofssonParam {
    #[schemars(
        description = "JSON array: mapped area per class (ha atau pixel count), e.g. [50000, 30000, 20000]"
    )]
    pub mapped_areas_json: String,
    #[schemars(
        description = "JSON 2D array: confusion matrix dari stratified random sampling, e.g. [[45,3,2],[1,38,1],[2,1,47]]"
    )]
    pub confusion_matrix_json: String,
    #[schemars(
        description = "JSON array: nama kelas, e.g. [\"Hutan\",\"Pertanian\",\"Permukiman\"]"
    )]
    pub class_names_json: String,
    #[schemars(description = "Z-score for CI (default 1.96 = 95%)")]
    pub z_score: Option<f64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SupervisedRfParam {
    pub lat: f64,
    pub lon: f64,
    #[schemars(description = "Buffer radius (km)")]
    pub buffer_km: f64,
    #[schemars(
        description = "GeoJSON FeatureCollection training polygons dengan property 'class' (integer 0,1,2,...)"
    )]
    pub training_geojson: String,
    pub start_date: String,
    pub end_date: String,
    #[schemars(description = "Jumlah decision trees (default 100)")]
    pub n_trees: Option<u32>,
    pub output_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TopoCorrectParam {
    pub lat: f64,
    pub lon: f64,
    pub buffer_km: f64,
    pub start_date: String,
    pub end_date: String,
    pub output_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NdviTimeseriesParam {
    pub lat: f64,
    pub lon: f64,
    pub buffer_km: f64,
    pub start_year: u32,
    pub end_year: u32,
    pub output_path: String,
}

// Custom Thematic Map Generator Param
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CustomThematicMapParam {
    #[schemars(
        description = "Path lokal GeoJSON batas wilayah (diambil dari output BIG/Admin tool)"
    )]
    pub geojson_path: String,
    #[schemars(description = "Path lokal tempat Peta final (.png) akan disimpan")]
    pub output_path: String,
    #[schemars(description = "Judul utama peta SNI")]
    pub title: String,
    #[schemars(description = "Path lokal ke file GeoTIFF hasil analisis raster sebelumnya")]
    pub overlay_raster_path: String,
    #[schemars(
        description = "Tipe analisis legenda: 'continuous' (untuk colorbar gradien) atau 'discrete' (untuk kotak warna klasifikasi)"
    )]
    pub analysis_type: String,
    #[schemars(
        description = "Pilihan Colormap Matplotlib (contoh: 'RdYlGn', 'turbo', 'viridis', 'tab10')"
    )]
    pub cmap: String,
    #[schemars(
        description = "JSON String dari label diskrit. Contoh: {\"#ff0000\": \"Hutan (100 Ha)\"} (opsional, hanya untuk discrete)"
    )]
    pub discrete_labels_json: Option<String>,
    #[schemars(
        description = "Label di atas colorbar (contoh: 'Konsentrasi TSS (mg/L)') (opsional, hanya untuk continuous)"
    )]
    pub colorbar_label: Option<String>,
    #[schemars(
        description = "Kesimpulan naratif singkat untuk Kotak Peringatan/Kesimpulan Kuning (opsional)"
    )]
    pub conclusion_text: Option<String>,
    #[schemars(
        description = "JSON String berisikan statistik untuk Tabel Metadata SNI. Contoh: {\"Algoritma\": \"CCDC\", \"Resolusi\": \"10m\"} (opsional)"
    )]
    pub stats_json: Option<String>,
}

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("env-indonesia-mcp/1.0.0")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
});

#[derive(Debug, Clone)]
pub struct EnvIndonesiaServer {
    tool_router: ToolRouter<Self>,
}

impl EnvIndonesiaServer {
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
    #[schemars(
        description = "Bounding box: south,west,north,east. Default: Indonesia (-11,95,6,141)"
    )]
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
pub struct StacSearchParam {
    #[schemars(description = "STAC API: 'mpc' (Microsoft Planetary Computer, 135 collections) or 'earth-search' (Element 84, 9 collections)")]
    pub api: Option<String>,
    #[schemars(description = "Collection ID, e.g. sentinel-2-l2a, landsat-c2-l2, sentinel-1-grd, sentinel-5p-l2-netcdf, modis-13A1-061, gpm-imerg-hhr, esa-worldcover, planet-nicfi-visual, cop-dem-glo-30")]
    pub collection: String,
    #[schemars(description = "Bounding box: south,west,north,east. Default: Indonesia (-11.5,95.0,6.0,141.0)")]
    pub bbox: Option<String>,
    #[schemars(description = "Datetime range ISO8601: '2025-01-01T00:00:00Z/2026-12-31T23:59:59Z'. Default: 2024-2026")]
    pub datetime: Option<String>,
    #[schemars(description = "Max results (default 10, max 100)")]
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StacCollectionParam {
    #[schemars(description = "STAC API: 'mpc' or 'earth-search'. Default: mpc")]
    pub api: Option<String>,
    #[schemars(description = "Collection ID to describe, e.g. sentinel-2-l2a, sentinel-5p-l2-netcdf, planet-nicfi-visual")]
    pub collection: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StacListParam {
    #[schemars(description = "STAC API: 'mpc' (135 collections) or 'earth-search' (9 collections). Default: mpc")]
    pub api: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StacAssetParam {
    #[schemars(description = "STAC API: 'mpc' or 'earth-search'. Default: mpc")]
    pub api: Option<String>,
    #[schemars(description = "Collection ID (e.g., sentinel-2-l2a)")]
    pub collection: String,
    #[schemars(description = "Item/Scene ID from stac_search results")]
    pub item_id: String,
    #[schemars(description = "Asset key (e.g., visual, red, nir)")]
    pub asset_key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StacDownloadParam {
    #[schemars(description = "STAC API: 'mpc' or 'earth-search'. Default: mpc")]
    pub api: Option<String>,
    pub collection: String,
    pub item_id: String,
    pub asset_key: String,
    pub output_dir: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LatLonParam {
    #[schemars(description = "Latitude (Indonesia: -11.5 to 6.0)")]
    pub lat: Option<f64>,
    #[schemars(description = "Longitude (Indonesia: 95.0 to 141.5)")]
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
pub struct FloodSarParam {
    #[schemars(description = "Latitude (Indonesia: -11.5 to 6.0)")]
    pub lat: f64,
    #[schemars(description = "Longitude (Indonesia: 95.0 to 141.5)")]
    pub lon: f64,
    #[schemars(description = "Buffer radius in km (default: 10)")]
    pub buffer_km: Option<f64>,
    #[schemars(description = "Flood event date (YYYY-MM-DD). When the flooding occurred.")]
    pub flood_date: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct KarhutlaParam {
    #[schemars(description = "Latitude (Indonesia: -11.5 to 6.0)")]
    pub lat: f64,
    #[schemars(description = "Longitude (Indonesia: 95.0 to 141.5)")]
    pub lon: f64,
    #[schemars(description = "Buffer radius in km (default: 10)")]
    pub buffer_km: Option<f64>,
    #[schemars(description = "Fire event date (YYYY-MM-DD). When the fire occurred.")]
    pub fire_date: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CoralAlertParam {
    #[schemars(description = "Latitude of reef site (Indonesia: -11.5 to 6.0)")]
    pub lat: f64,
    #[schemars(description = "Longitude of reef site (Indonesia: 95.0 to 141.5)")]
    pub lon: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClimateProjParam {
    #[schemars(description = "Latitude (Indonesia: -11.5 to 6.0)")]
    pub lat: f64,
    #[schemars(description = "Longitude (Indonesia: 95.0 to 141.5)")]
    pub lon: f64,
    #[schemars(description = "Scenario: 'ssp245' (moderate ~3°C) or 'ssp585' (worst ~4.5°C). Default: ssp585")]
    pub scenario: Option<String>,
    #[schemars(description = "Period: '2030', '2050', or '2080'. Default: 2050")]
    pub period: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PeatCo2Param {
    #[schemars(description = "Burned area (hectares)")]
    pub burned_area_ha: f64,
    #[schemars(description = "Peat depth (meters). Indonesia: 0.5-12m typical")]
    pub peat_depth_m: f64,
    #[schemars(description = "Severity: 'low', 'moderate', or 'high'")]
    pub severity: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HeavyMetalParam {
    #[schemars(description = "Lead (Pb) concentration mg/L")]
    pub pb: f64,
    #[schemars(description = "Cadmium (Cd) concentration mg/L")]
    pub cd: f64,
    #[schemars(description = "Mercury (Hg) concentration mg/L")]
    pub hg: f64,
    #[schemars(description = "Arsenic (As) concentration mg/L")]
    pub as_: f64,
    #[schemars(description = "Chromium (Cr) concentration mg/L")]
    pub cr: f64,
    #[schemars(description = "Body weight kg (default 70)")]
    pub body_weight_kg: Option<f64>,
    #[schemars(description = "Water intake L/day (default 2)")]
    pub intake_l_per_day: Option<f64>,
    #[schemars(description = "Exposure years (default 30)")]
    pub exposure_years: Option<f64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PollutionIndexParam {
    #[schemars(description = "BOD concentration mg/L")]
    pub bod: f64,
    #[schemars(description = "COD concentration mg/L")]
    pub cod: f64,
    #[schemars(description = "Dissolved Oxygen mg/L")]
    pub do_: f64,
    #[schemars(description = "Total Suspended Solids mg/L")]
    pub tss: f64,
    #[schemars(description = "Total Coliform MPN/100mL (optional)")]
    pub total_coliform: Option<f64>,
    #[schemars(description = "Water class 1-4 (PP 22/2021). Default 2")]
    pub class: Option<u8>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AsgmMercuryParam {
    #[schemars(description = "Hg concentration in water mg/L")]
    pub hg_conc_water: f64,
    #[schemars(description = "Hg concentration in sediment mg/kg")]
    pub hg_conc_sediment: f64,
    #[schemars(description = "Gold production kg/year")]
    pub gold_production_kg_yr: f64,
    #[schemars(description = "Population exposed")]
    pub population_exposed: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HealthImpactAssessmentParam {
    #[schemars(description = "Pollutant: PM2.5, NO2, SO2, O3")]
    pub pollutant: String,
    #[schemars(description = "Concentration ug/m3 (annual or 24h avg)")]
    pub concentration_ug_m3: f64,
    #[schemars(description = "Population exposed")]
    pub population_exposed: f64,
    #[schemars(description = "Background concentration ug/m3 (use WHO guideline if unknown: PM2.5=5)")]
    pub background_conc_ug_m3: f64,
    #[schemars(description = "Exposure duration years")]
    pub exposure_years: f64,
    #[schemars(description = "Value of one DALY in USD (WHO range 50000-150000)")]
    pub valuation_usd_per_daly: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RestorationCostParam {
    #[schemars(description = "Restoration type: mangrove, peatland, river, mine, coral")]
    pub restoration_type: String,
    #[schemars(description = "Area in hectares (for river: pass km value; for coral: pass m2 value)")]
    pub area_ha: f64,
    #[schemars(description = "Degradation level: light, moderate, severe")]
    pub degradation_level: String,
    #[schemars(description = "Years since degradation")]
    pub years_since_degradation: f64,
    #[schemars(description = "Monitoring period years")]
    pub monitoring_years: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProblemSolutionImpactParam {
    #[schemars(description = "Problem type: flood, fire, pollution_river, pollution_air, coastal_erosion, mining_impact")]
    pub problem_type: String,
    #[schemars(description = "Location name")]
    pub location_name: String,
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Affected/study area hectares")]
    pub area_ha: f64,
    #[schemars(description = "Severity: low, moderate, high")]
    pub severity: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClimateVulnerabilityParam {
    #[schemars(description = "Temperature change °C (from climate_projection)")]
    pub temp_change_c: f64,
    #[schemars(description = "Precipitation change %")]
    pub precip_change_pct: f64,
    #[schemars(description = "Extreme events per year")]
    pub extreme_event_freq: u32,
    #[schemars(description = "Elevation meters")]
    pub elevation_m: f64,
    #[schemars(description = "Population density per km²")]
    pub population_density: f64,
    #[schemars(description = "Poverty rate %")]
    pub poverty_rate: f64,
    #[schemars(description = "GDP per capita USD")]
    pub gdp_per_capita_usd: f64,
    #[schemars(description = "Literacy rate %")]
    pub literacy_rate: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MineImpactParam {
    #[schemars(description = "Mine type: nickel, coal, gold, tin")]
    pub mine_type: String,
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Mine area hectares")]
    pub area_ha: f64,
    #[schemars(description = "Deforested area hectares")]
    pub deforestation_ha: f64,
    #[schemars(description = "Water pollution: good, light, moderate, heavy")]
    pub water_pollution_level: String,
    #[schemars(description = "Tailings present? true/false")]
    pub has_tailings: bool,
    #[schemars(description = "Acid Mine Drainage present? true/false")]
    pub has_amd: bool,
    #[schemars(description = "Social displacement (people)")]
    pub social_displacement: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TidalFloodParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "SLR scenario: 'ssp245' or 'ssp585'")]
    pub slr_scenario: Option<String>,
    #[schemars(description = "Subsidence rate mm/yr (from InSAR)")]
    pub subsidence_rate_mm_yr: f64,
    #[schemars(description = "Projection year: 2050 or 2100")]
    pub projection_year: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandslideParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Buffer km (default 10)")]
    pub buffer_km: Option<f64>,
    #[schemars(description = "24h rainfall mm")]
    pub rainfall_mm: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GpmImergParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Date YYYY-MM-DD")]
    pub date: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CycloneParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PlasticLeakageParam {
    #[schemars(description = "Population")]
    pub population: u64,
    #[schemars(description = "Waste generation kg/cap/day (Indonesia ~0.7)")]
    pub waste_generation_kg_cap_day: f64,
    #[schemars(description = "Plastic fraction % (default 10)")]
    pub plastic_fraction_pct: f64,
    #[schemars(description = "Mismanaged waste % (Indonesia ~50)")]
    pub mismanaged_waste_pct: f64,
    #[schemars(description = "Coastal population % (Indonesia ~60)")]
    pub coastal_population_pct: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ViirsFishingParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Date YYYY-MM-DD")]
    pub date: String,
}

// ═══ GOD TIER v3: 9 Advanced Modeling Tools ═══

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EnhancedLeopoldParam {
    #[schemars(description = "JSON: [[\"kegiatan\",\"komponen\",magnitude(-10..10),importance(1..10)],...]")]
    pub impacts_json: String,
    #[schemars(description = "JSON: [[\"criterion\",weight],...] (e.g. [[\"Ekologi\",0.4],[\"Sosial\",0.25],...]). Used as criteria names if pairwise provided.")]
    pub criteria_weights_json: String,
    #[schemars(description = "JSON: [[\"Alt A\",[score1,score2,...]],...] for TOPSIS ranking")]
    pub alternatives_json: String,
    #[schemars(description = "Optional: n×n AHP pairwise comparison matrix (Saaty 1-9 scale). If provided, computes true λ_max, CI, CR via power iteration. e.g. [[1,3,5],[0.333,1,3],[0.2,0.333,1]]")]
    pub pairwise_matrix_json: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LcaEnhancedParam {
    #[schemars(description = "JSON: [[\"material\",mass_kg],...] (e.g. [[\"semen\",500],[\"baja\",100]])")]
    pub materials_json: String,
    #[schemars(description = "Transport in kg·km (e.g. 50000 for 500kg × 100km)")]
    pub transport_kg_km: f64,
    #[schemars(description = "Energy consumption in kWh (Indonesia grid)")]
    pub energy_kwh: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmpParam {
    #[schemars(description = "JSON: [[\"dampak\",\"komponen\",significance],...] (from Leopold)")]
    pub impacts_json: String,
    #[schemars(description = "Project type (e.g. tambang, jalan, PLTU)")]
    pub project_type: String,
    #[schemars(description = "Location name")]
    pub location: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Iso14001GapParam {
    #[schemars(description = "JSON: [[\"clause_id\",\"sub_req\",level(1-5),\"evidence\"],...]. Level: 1=not implemented, 3=partial, 5=fully. Empty = template.")]
    pub compliance_json: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TrigrsParam {
    #[schemars(description = "Rainfall intensity mm/hr")]
    pub rainfall_mm_hr: f64,
    #[schemars(description = "Duration hours")]
    pub duration_hr: f64,
    #[schemars(description = "Saturated hydraulic conductivity m/s (e.g. 1e-6 for clay, 1e-4 for sand)")]
    pub ks_m_s: f64,
    #[schemars(description = "Diffusivity m²/s (e.g. 1e-5)")]
    pub d2_m: f64,
    #[schemars(description = "Effective cohesion kPa (e.g. 5-20)")]
    pub cohesion_kpa: f64,
    #[schemars(description = "Friction angle degrees (e.g. 25-35)")]
    pub friction_angle_deg: f64,
    #[schemars(description = "Slope angle degrees")]
    pub slope_deg: f64,
    #[schemars(description = "Soil depth meters")]
    pub depth_m: f64,
    #[schemars(description = "Porosity (0-1, e.g. 0.3-0.5)")]
    pub porosity: f64,
    #[schemars(description = "Saturated unit weight kN/m³ (e.g. 18-20)")]
    pub unit_weight_kn_m3: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ModflowParam {
    #[schemars(description = "Number of layers")]
    pub grid_nlay: u32,
    #[schemars(description = "Number of rows")]
    pub grid_nrow: u32,
    #[schemars(description = "Number of columns")]
    pub grid_ncol: u32,
    #[schemars(description = "Cell size meters")]
    pub cell_size_m: f64,
    #[schemars(description = "Horizontal hydraulic conductivity m/s")]
    pub hk_m_s: f64,
    #[schemars(description = "Vertical hydraulic conductivity m/s")]
    pub vk_m_s: f64,
    #[schemars(description = "Specific yield (0.05-0.3)")]
    pub sy: f64,
    #[schemars(description = "Specific storage 1/m (e.g. 1e-5)")]
    pub ss_per_m: f64,
    #[schemars(description = "Pumping rate m³/day")]
    pub pumping_m3_day: f64,
    #[schemars(description = "Pumping well X (column)")]
    pub pumping_x: u32,
    #[schemars(description = "Pumping well Y (row)")]
    pub pumping_y: u32,
    #[schemars(description = "Pumping well layer")]
    pub pumping_layer: u32,
    #[schemars(description = "Recharge mm/year")]
    pub recharge_mm_yr: f64,
    #[schemars(description = "Constant head boundary m")]
    pub chb_head_m: f64,
    #[schemars(description = "Simulation type: 'steady' or 'transient'")]
    pub sim_type: String,
    #[schemars(description = "Duration days (for transient)")]
    pub duration_days: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MintpyInsarParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Start date YYYY-MM-DD")]
    pub date_start: String,
    #[schemars(description = "End date YYYY-MM-DD")]
    pub date_end: String,
    #[schemars(description = "BBox size km (default 10)")]
    pub bbox_km: Option<f64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EnkfParam {
    #[schemars(description = "JSON: [[x1,x2,...],...] ensemble of state vectors")]
    pub model_states_json: String,
    #[schemars(description = "JSON: [y1, y2, ...] observation vector")]
    pub observations_json: String,
    #[schemars(description = "Ensemble size (default 50)")]
    pub ensemble_size: Option<u32>,
    #[schemars(description = "Observation noise std dev")]
    pub noise_std: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FireDangerRatingParam {
    pub kbdi_yesterday: f64,
    pub max_temp_c: f64,
    pub mean_annual_precip_mm: f64,
    pub daily_precip_mm: f64,
    pub is_peatland: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SourceApportionmentParam {
    pub pm25_total_ug_m3: f64,
    pub so4_ug_m3: f64,
    pub no3_ug_m3: f64,
    pub ec_ug_m3: f64,
    pub oc_ug_m3: f64,
    pub crustal_ug_m3: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DstpPlumeParam {
    pub discharge_depth_m: f64,
    pub tailings_volume_m3_day: f64,
    pub solid_fraction_pct: f64,
    pub ocean_current_speed_m_s: f64,
    pub settling_velocity_mm_s: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AermodGeneratorParam {
    pub project_name: String,
    pub source_lat: f64,
    pub source_lon: f64,
    pub stack_height_m: f64,
    pub stack_diameter_m: f64,
    pub exit_velocity_m_s: f64,
    pub exit_temp_k: f64,
    pub emission_rate_g_s: f64,
    pub pollutant_id: String,
    pub is_rural: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PhreeqcLeachingParam {
    pub waste_type: String,
    pub solid_mass_g: f64,
    pub water_volume_l: f64,
    pub target_ph: f64,
    pub initial_metals_mg_kg: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FireSpreadParam {
    #[schemars(description = "Anderson fuel model 1-13 (1=short grass, 4=chaparral, 10=timber)")]
    pub fuel_model: u8,
    #[schemars(description = "Wind speed m/s")]
    pub wind_speed_ms: f64,
    #[schemars(description = "Wind direction degrees (0=N, 90=E)")]
    pub wind_dir_deg: f64,
    #[schemars(description = "Slope degrees")]
    pub slope_deg: f64,
    #[schemars(description = "Fuel moisture % (5=dry, 30=wet)")]
    pub moisture_pct: f64,
    #[schemars(description = "Ignition latitude")]
    pub ignition_lat: f64,
    #[schemars(description = "Ignition longitude")]
    pub ignition_lon: f64,
    #[schemars(description = "Duration hours")]
    pub duration_hr: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FireSuppressionParam {
    #[schemars(description = "Fire area in hectares")]
    pub fire_area_ha: f64,
    #[schemars(description = "Duration hours")]
    pub duration_hr: f64,
    #[schemars(description = "Number of aircraft")]
    pub n_aircraft: u32,
    #[schemars(description = "Aircraft mix: mixed/standard, water_only/helicopter, heavy/vlats")]
    pub aircraft_mix: String,
    #[schemars(description = "Wind speed m/s")]
    pub wind_speed_ms: f64,
    #[schemars(description = "Wind direction degrees")]
    pub wind_dir_deg: f64,
    #[schemars(description = "Anderson fuel model 1-13")]
    pub fuel_model: u8,
    #[schemars(description = "Budget number of drops")]
    pub budget_drops: u32,
}

// ═══ ENVIRONMENTAL ENGINEERING DESIGN TOOLS (11) ═══

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PumpTreatParam {
    #[schemars(description = "Horizontal hydraulic conductivity m/s")]
    pub hk_m_s: f64,
    #[schemars(description = "Aquifer thickness m")]
    pub aquifer_thickness_m: f64,
    #[schemars(description = "Hydraulic gradient (dh/dx, dimensionless)")]
    pub hydraulic_gradient: f64,
    #[schemars(description = "Pumping rate m³/day")]
    pub pumping_rate_m3_day: f64,
    #[schemars(description = "Porosity (0.2-0.4 typical)")]
    pub porosity: f64,
    #[schemars(description = "Contaminant name")]
    pub contaminant: String,
    #[schemars(description = "Initial concentration µg/L")]
    pub initial_conc_ug_l: f64,
    #[schemars(description = "Target cleanup concentration µg/L")]
    pub target_conc_ug_l: f64,
    #[schemars(description = "Cleanup time target years")]
    pub cleanup_time_years: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PrbDesignParam {
    #[schemars(description = "Contaminant (tce, pce, cr6, as, etc.)")]
    pub contaminant: String,
    #[schemars(description = "Inflow concentration µg/L")]
    pub c_inflow_ug_l: f64,
    #[schemars(description = "Target outlet concentration µg/L")]
    pub c_target_ug_l: f64,
    #[schemars(description = "First-order degradation rate hr⁻¹ (0 = auto from contaminant)")]
    pub k_first_order_hr: f64,
    #[schemars(description = "Groundwater seepage velocity m/day")]
    pub gw_velocity_m_day: f64,
    #[schemars(description = "Barrier porosity")]
    pub porosity: f64,
    #[schemars(description = "Barrier width m (perpendicular to flow)")]
    pub barrier_width_m: f64,
    #[schemars(description = "Barrier depth m")]
    pub barrier_depth_m: f64,
    #[schemars(description = "ZVI bulk density kg/m³ (typical 2500)")]
    pub bulk_density_kg_m3: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SveDesignParam {
    #[schemars(description = "Air permeability m² (1e-14 to 1e-11 typical)")]
    pub k_air_m2: f64,
    #[schemars(description = "Well screen length m")]
    pub screen_length_m: f64,
    #[schemars(description = "Applied vacuum kPa (below atmospheric)")]
    pub vacuum_pressure_kpa: f64,
    #[schemars(description = "Contaminant name")]
    pub contaminant: String,
    #[schemars(description = "NAPL mass in soil kg")]
    pub napl_mass_kg: f64,
    #[schemars(description = "Soil porosity")]
    pub soil_porosity: f64,
    #[schemars(description = "Soil temperature °C")]
    pub soil_temp_c: f64,
    #[schemars(description = "Cleanup time target days")]
    pub cleanup_time_target_days: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BioremediationParam {
    #[schemars(description = "Contaminant (benzene, toluene, tce, pah, diesel, etc.)")]
    pub contaminant: String,
    #[schemars(description = "Initial concentration mg/L")]
    pub initial_conc_mg_l: f64,
    #[schemars(description = "Target concentration mg/L")]
    pub target_conc_mg_l: f64,
    #[schemars(description = "First-order decay rate day⁻¹ (0 = auto from contaminant)")]
    pub k_first_order_day: f64,
    #[schemars(description = "Soil volume m³")]
    pub soil_volume_m3: f64,
    #[schemars(description = "Porosity")]
    pub porosity: f64,
    #[schemars(description = "Soil bulk density kg/m³")]
    pub bulk_density_kg_m3: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CycloneSeparatorParam {
    #[schemars(description = "Gas flow m³/s")]
    pub gas_flow_m3_s: f64,
    #[schemars(description = "Particle density kg/m³")]
    pub particle_density_kg_m3: f64,
    #[schemars(description = "Gas viscosity Pa·s (air at 20°C = 1.81e-5)")]
    pub gas_viscosity_pa_s: f64,
    #[schemars(description = "Cyclone diameter m")]
    pub cyclone_diameter_m: f64,
    #[schemars(description = "Target efficiency %")]
    pub target_efficiency_pct: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BaghouseParam {
    #[schemars(description = "Gas flow m³/s")]
    pub gas_flow_m3_s: f64,
    #[schemars(description = "Dust concentration g/m³")]
    pub dust_conc_g_m3: f64,
    #[schemars(description = "Target pressure drop Pa (1000-2500 typical)")]
    pub target_pressure_drop_pa: f64,
    #[schemars(description = "Bag diameter m (0.1-0.3 typical)")]
    pub bag_diameter_m: f64,
    #[schemars(description = "Bag length m (3-10 typical)")]
    pub bag_length_m: f64,
    #[schemars(description = "Fabric type: woven, polyester, felt, ptfe, fiberglass")]
    pub fabric_type: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ScrubberParam {
    #[schemars(description = "Gas flow m³/s")]
    pub gas_flow_m3_s: f64,
    #[schemars(description = "Particle density kg/m³")]
    pub particle_density_kg_m3: f64,
    #[schemars(description = "Target efficiency %")]
    pub target_efficiency_pct: f64,
    #[schemars(description = "Throat velocity m/s (40-100 typical)")]
    pub throat_velocity_ms: f64,
    #[schemars(description = "L/G ratio L/m³ (0.5-10 typical)")]
    pub lg_ratio_l_m3: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ParticleType {
    Dielectric,
    Conductive,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EspParam {
    #[schemars(description = "Gas flow m3/s")]
    pub gas_flow_m3_s: f64,
    #[schemars(description = "Particle density kg/m3")]
    pub particle_density_kg_m3: f64,
    #[schemars(description = "Target efficiency %")]
    pub target_efficiency_pct: f64,
    #[schemars(description = "Field strength kV/cm (3-8 typical)")]
    pub field_strength_kv_cm: f64,
    #[schemars(description = "Particle diameter um")]
    pub particle_diameter_um: f64,
    #[schemars(description = "Particle type: dielectric/conductive")]
    pub particle_type: ParticleType,
    #[schemars(description = "Particle resistivity ohm.cm")]
    pub resistivity_ohm_cm: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RoDesignParam {
    #[schemars(description = "Feed salinity mg/L (as NaCl)")]
    pub feed_salinity_mg_l: f64,
    #[schemars(description = "Target permeate quality mg/L")]
    pub target_permeate_mg_l: f64,
    #[schemars(description = "Feed pressure bar (40-80 typical)")]
    pub feed_pressure_bar: f64,
    #[schemars(description = "Membrane water permeability LMH/bar (1-5 typical)")]
    pub membrane_water_perm_l_m2_h_bar: f64,
    #[schemars(description = "Membrane salt permeability LMH (0.01-0.5 typical)")]
    pub membrane_salt_perm_l_m2_h: f64,
    #[schemars(description = "Feed flow m³/day")]
    pub feed_flow_m3_day: f64,
    #[schemars(description = "Temperature °C")]
    pub temp_c: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GacDesignParam {
    #[schemars(description = "Contaminant name")]
    pub contaminant: String,
    #[schemars(description = "Influent concentration mg/L")]
    pub c_influent_mg_l: f64,
    #[schemars(description = "Target effluent mg/L")]
    pub c_target_mg_l: f64,
    #[schemars(description = "Flow m³/day")]
    pub flow_m3_day: f64,
    #[schemars(description = "Freundlich K (mg/g)(L/mg)^(1/n)")]
    pub freundlich_k: f64,
    #[schemars(description = "Freundlich 1/n (0.1-0.7 typical)")]
    pub freundlich_1_over_n: f64,
    #[schemars(description = "Empty bed contact time minutes (5-30 typical)")]
    pub ebct_min: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ContaminantTransport1DParam {
    #[schemars(description = "Distance from source m")]
    pub distance_m: f64,
    #[schemars(description = "Groundwater velocity m/day")]
    pub velocity_m_day: f64,
    #[schemars(description = "Dispersion coefficient m2/day")]
    pub dispersion_m2_day: f64,
    #[schemars(description = "Time days")]
    pub time_days: f64,
    #[schemars(description = "Retardation factor R (1=no retardation)")]
    pub retardation_factor: f64,
    #[schemars(description = "First-order decay rate day-1")]
    pub decay_rate_day: f64,
    #[schemars(description = "Initial concentration mg/L")]
    pub initial_conc_mg_l: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ContaminantTransport2DParam {
    #[schemars(description = "Distance x (downgradient) m")]
    pub distance_x_m: f64,
    #[schemars(description = "Source width y m")]
    pub source_width_y_m: f64,
    #[schemars(description = "Source depth z m")]
    pub source_depth_z_m: f64,
    #[schemars(description = "Groundwater velocity m/day")]
    pub velocity_m_day: f64,
    #[schemars(description = "Longitudinal dispersion m2/day")]
    pub dispersion_x_m2_day: f64,
    #[schemars(description = "Transverse dispersion y m2/day")]
    pub dispersion_y_m2_day: f64,
    #[schemars(description = "Vertical dispersion z m2/day")]
    pub dispersion_z_m2_day: f64,
    #[schemars(description = "Time days")]
    pub time_days: f64,
    #[schemars(description = "Retardation factor")]
    pub retardation_factor: f64,
    #[schemars(description = "Decay rate day-1")]
    pub decay_rate_day: f64,
    #[schemars(description = "Initial concentration mg/L")]
    pub initial_conc_mg_l: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct VaporIntrusionParam {
    #[schemars(description = "Source concentration ug/m3")]
    pub source_conc_ug_m3: f64,
    #[schemars(description = "Total soil porosity")]
    pub soil_porosity_total: f64,
    #[schemars(description = "Water-filled porosity")]
    pub soil_porosity_water: f64,
    #[schemars(description = "Air-filled porosity")]
    pub soil_porosity_air: f64,
    #[schemars(description = "Stratum thickness (source to building) m")]
    pub stratum_thickness_m: f64,
    #[schemars(description = "Building footprint m2")]
    pub bldg_footprint_m2: f64,
    #[schemars(description = "Building height m")]
    pub bldg_height_m: f64,
    #[schemars(description = "Air exchange rate hr-1 (ACH)")]
    pub air_exchange_rate_hr: f64,
    #[schemars(description = "Foundation crack area m2")]
    pub crack_area_m2: f64,
    #[schemars(description = "Crack depth m")]
    pub crack_depth_m: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RiverQualityParam {
    #[schemars(description = "River length m")]
    pub river_length_m: f64,
    #[schemars(description = "Flow m3/s")]
    pub flow_m3_s: f64,
    #[schemars(description = "Velocity m/s")]
    pub velocity_m_s: f64,
    #[schemars(description = "Initial BOD mg/L")]
    pub initial_bod_mg_l: f64,
    #[schemars(description = "Initial DO mg/L")]
    pub initial_do_mg_l: f64,
    #[schemars(description = "BOD decay rate day-1")]
    pub bod_decay_rate_day: f64,
    #[schemars(description = "Reaeration rate day-1")]
    pub reaeration_rate_day: f64,
    #[schemars(description = "Saturation DO mg/L")]
    pub saturation_do_mg_l: f64,
    #[schemars(description = "Number of reaches")]
    pub n_reaches: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReaerationParam {
    #[schemars(description = "Velocity m/s")]
    pub velocity_m_s: f64,
    #[schemars(description = "Depth m")]
    pub depth_m: f64,
    #[schemars(description = "Temperature C")]
    pub temp_c: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SODParam {
    #[schemars(description = "SOD at 20C g/m2/day")]
    pub sod20_g_m2_day: f64,
    #[schemars(description = "Temperature C")]
    pub temp_c: f64,
    #[schemars(description = "Sediment area m2")]
    pub area_m2: f64,
    #[schemars(description = "River flow m3/s (0=ignore)")]
    pub river_flow_m3_s: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ChlorophyllParam {
    #[schemars(description = "Phosphorus load kg/year")]
    pub phosphorus_load_kg_yr: f64,
    #[schemars(description = "Lake area km2")]
    pub lake_area_km2: f64,
    #[schemars(description = "Lake volume m3")]
    pub lake_volume_m3: f64,
    #[schemars(description = "Outflow m3/s")]
    pub outflow_m3_s: f64,
    #[schemars(description = "Lake type: deep, shallow, or mixed")]
    pub lake_type: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MBRParam {
    #[schemars(description = "Flow m3/day")]
    pub flow_m3_day: f64,
    #[schemars(description = "Influent BOD mg/L")]
    pub influent_bod_mg_l: f64,
    #[schemars(description = "Target effluent BOD mg/L")]
    pub target_effluent_bod_mg_l: f64,
    #[schemars(description = "HRT hours")]
    pub hrt_hours: f64,
    #[schemars(description = "SRT days")]
    pub srt_days: f64,
    #[schemars(description = "MLSS mg/L")]
    pub mlss_mg_l: f64,
    #[schemars(description = "Membrane flux LMH (L/m2/hr)")]
    pub membrane_flux_lmh: f64,
    #[schemars(description = "Temperature C")]
    pub temp_c: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SBRParam {
    #[schemars(description = "Flow m3/day")]
    pub flow_m3_day: f64,
    #[schemars(description = "Influent BOD mg/L")]
    pub influent_bod_mg_l: f64,
    #[schemars(description = "Target BOD mg/L")]
    pub target_bod_mg_l: f64,
    #[schemars(description = "Cycles per day")]
    pub n_cycles_day: u32,
    #[schemars(description = "MLSS mg/L")]
    pub mlss_mg_l: f64,
    #[schemars(description = "Fill fraction (0-1)")]
    pub fill_fraction: f64,
    #[schemars(description = "React time hr")]
    pub react_time_hr: f64,
    #[schemars(description = "Settle time hr")]
    pub settle_time_hr: f64,
    #[schemars(description = "Draw time hr")]
    pub draw_time_hr: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AOPParam {
    #[schemars(description = "Contaminant name")]
    pub contaminant: String,
    #[schemars(description = "Initial concentration mg/L")]
    pub initial_conc_mg_l: f64,
    #[schemars(description = "Target concentration mg/L")]
    pub target_conc_mg_l: f64,
    #[schemars(description = "Process: ozone, uv_h2o2, fenton, uv_ozone")]
    pub process_type: String,
    #[schemars(description = "k_OH rate constant M-1 s-1 (0=auto)")]
    pub k_oh_m: f64,
    #[schemars(description = "OH radical concentration M (0=auto)")]
    pub oh_conc_m: f64,
    #[schemars(description = "Contact time min")]
    pub contact_time_min: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NutrientRemovalParam {
    #[schemars(description = "Influent TKN mg/L")]
    pub influent_tkn_mg_l: f64,
    #[schemars(description = "Influent NO3 mg/L")]
    pub influent_no3_mg_l: f64,
    #[schemars(description = "Target TN mg/L")]
    pub target_tn_mg_l: f64,
    #[schemars(description = "SRT days")]
    pub srt_days: f64,
    #[schemars(description = "Temperature C")]
    pub temp_c: f64,
    #[schemars(description = "DO mg/L")]
    pub do_mg_l: f64,
    #[schemars(description = "MLSS mg/L")]
    pub mlss_mg_l: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StruviteParam {
    #[schemars(description = "Mg concentration mg/L")]
    pub mg_mg_l: f64,
    #[schemars(description = "NH4 concentration mg/L")]
    pub nh4_mg_l: f64,
    #[schemars(description = "PO4 concentration mg/L")]
    pub po4_mg_l: f64,
    #[schemars(description = "pH")]
    pub ph: f64,
    #[schemars(description = "Temperature C")]
    pub temp_c: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ChlorineDemandParam {
    #[schemars(description = "Free chlorine mg/L")]
    pub free_chlorine_mg_l: f64,
    #[schemars(description = "Contact time min")]
    pub contact_time_min: f64,
    #[schemars(description = "Target log removal")]
    pub target_log_removal: f64,
    #[schemars(description = "Contaminant: giardia, virus, bacteria")]
    pub contaminant: String,
    #[schemars(description = "Temperature C")]
    pub temp_c: f64,
    #[schemars(description = "pH")]
    pub ph: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BufferCapacityParam {
    #[schemars(description = "Alkalinity mg/L as CaCO3")]
    pub alkalinity_mg_l_caco3: f64,
    #[schemars(description = "pH")]
    pub ph: f64,
    #[schemars(description = "Temperature C")]
    pub temp_c: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IndoorAirParam {
    #[schemars(description = "Emission rate mg/hr")]
    pub emission_rate_mg_hr: f64,
    #[schemars(description = "Room volume m3")]
    pub room_volume_m3: f64,
    #[schemars(description = "Ventilation rate m3/hr")]
    pub ventilation_m3_hr: f64,
    #[schemars(description = "Outdoor concentration mg/m3")]
    pub outdoor_conc_mg_m3: f64,
    #[schemars(description = "Deposition rate hr-1")]
    pub deposition_rate_hr: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StackHeightParam {
    #[schemars(description = "Building height m")]
    pub building_height_m: f64,
    #[schemars(description = "Building width m")]
    pub building_width_m: f64,
    #[schemars(description = "Building length m")]
    pub building_length_m: f64,
    #[schemars(description = "Wind direction degrees")]
    pub wind_direction_deg: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FugitiveDustParam {
    #[schemars(description = "Road type: paved, unpaved_industrial, unpaved_public")]
    pub road_type: String,
    #[schemars(description = "Silt loading g/m2 (paved)")]
    pub silt_loading_g_m2: f64,
    #[schemars(description = "Silt content % (unpaved)")]
    pub silt_content_pct: f64,
    #[schemars(description = "Average vehicle weight ton")]
    pub avg_vehicle_weight_ton: f64,
    #[schemars(description = "Precipitation days/year")]
    pub precip_days: u32,
    #[schemars(description = "Vehicle count")]
    pub vehicle_count: u32,
    #[schemars(description = "Road length m")]
    pub road_length_m: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct POMEParam {
    #[schemars(description = "FFB processed ton/day")]
    pub ton_ffb_day: f64,
    #[schemars(description = "Has pond system? (true/false)")]
    pub has_pond_system: bool,
    #[schemars(description = "Target BOD mg/L")]
    pub target_bod_mg_l: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MDLParam {
    #[schemars(description = "JSON array of replicate concentrations [c1,c2,...]")]
    pub replicate_concs_json: String,
    #[schemars(description = "Spike level mg/L")]
    pub spike_level_mg_l: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HoldingTimeParam {
    #[schemars(description = "Parameter name (bod, cod, voc, metals, etc.)")]
    pub parameter: String,
    #[schemars(description = "Sample matrix (water, soil)")]
    pub sample_matrix: String,
    #[schemars(description = "Days since sampling")]
    pub days_since_sampling: f64,
    #[schemars(description = "Preserved? (true/false)")]
    pub preserved: bool,
    #[schemars(description = "Storage temperature C")]
    pub temp_c: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CalibrationParam {
    #[schemars(description = "Instrument name")]
    pub instrument: String,
    #[schemars(description = "JSON array of standard concentrations")]
    pub std_concs_json: String,
    #[schemars(description = "JSON array of measured concentrations")]
    pub measured_concs_json: String,
    #[schemars(description = "Calibration range low")]
    pub calibration_range_low: f64,
    #[schemars(description = "Calibration range high")]
    pub calibration_range_high: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuAirPermukaanParam {
    #[schemars(description = "Parameter name (bod, do, tss, ph, pb, hg, total_coliform, etc.)")]
    pub parameter: String,
    #[schemars(description = "Measured value")]
    pub value: f64,
    #[schemars(description = "Water quality class 1-4 (1=drinking, 2=recreation, 3=livestock, 4=irrigation)")]
    pub kelas: u8,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuAirPermukaanMultiParam {
    #[schemars(description = "JSON object of parameter:value pairs")]
    pub params_json: String,
    #[schemars(description = "Water quality class 1-4")]
    pub kelas: u8,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SanksiAdministratifParam {
    #[schemars(description = "Violation type")]
    pub violation_type: String,
    #[schemars(description = "Has Persetujuan Lingkungan?")]
    pub has_persetujuan_lingkungan: bool,
    #[schemars(description = "Has Perizinan Berusaha?")]
    pub has_perizinan_berusaha: bool,
    #[schemars(description = "Investment value in Rupiah")]
    pub nilai_investasi_rp: f64,
    #[schemars(description = "Wastewater discharge m3/day")]
    pub debit_m3_day: f64,
    #[schemars(description = "Pollutant concentration mg/L")]
    pub konsentrasi_pencemar_mg_l: f64,
    #[schemars(description = "Duration of violation in days")]
    pub durasi_hari: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NDCMRVParam {
    #[schemars(description = "Current emissions in Gg CO2e")]
    pub current_emissions_gg_co2e: f64,
    #[schemars(description = "Sector (energi, ippu, pertanian, limbah, folu, kelautan, migas)")]
    pub sector: String,
    #[schemars(description = "Year")]
    pub year: u32,
    #[schemars(description = "Has MRV active?")]
    pub has_mrv: bool,
    #[schemars(description = "NDC scenario: LCCP_L, LCCP_H, CM1, CM2")]
    pub ndc_scenario: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TrafficImpactParam {
    #[schemars(description = "Road type (2/2-TT, 4/2-T, 6/2-T)")]
    pub road_type: String,
    #[schemars(description = "Lane width in meters")]
    pub lane_width_m: f64,
    #[schemars(description = "Traffic volume kend/jam")]
    pub volume_kend_per_jam: f64,
    #[schemars(description = "EMP for mobil penumpang")]
    pub emp_mp: f64,
    #[schemars(description = "EMP for kendaraan sedang")]
    pub emp_ks: f64,
    #[schemars(description = "EMP for sepeda motor")]
    pub emp_sm: f64,
    #[schemars(description = "EMP for bus besar")]
    pub emp_bb: f64,
    #[schemars(description = "Volume mobil penumpang")]
    pub vol_mp: f64,
    #[schemars(description = "Volume kendaraan sedang")]
    pub vol_ks: f64,
    #[schemars(description = "Volume sepeda motor")]
    pub vol_sm: f64,
    #[schemars(description = "Volume bus besar")]
    pub vol_bb: f64,
    #[schemars(description = "Side friction class (sangat rendah/rendah/sedang/tinggi/sangat tinggi)")]
    pub khs: String,
    #[schemars(description = "Shoulder width in meters")]
    pub shoulder_width_m: f64,
    #[schemars(description = "City population in millions")]
    pub city_population_million: f64,
    #[schemars(description = "Direction split percentage (50-70)")]
    pub direction_split: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MineReclamationParam {
    #[schemars(description = "Pit area in hectares")]
    pub pit_area_ha: f64,
    #[schemars(description = "Overburden area in hectares")]
    pub overburden_area_ha: f64,
    #[schemars(description = "Post-mining land use")]
    pub post_mining_land_use: String,
    #[schemars(description = "Revegetation species")]
    pub revegetation_species: String,
    #[schemars(description = "Target canopy cover percentage")]
    pub target_canopy_cover_pct: f64,
    #[schemars(description = "Years since reclamation started")]
    pub years_since_reclamation: u32,
    #[schemars(description = "Reclamation bond in Rupiah")]
    pub bond_rp: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RemediationTargetParam {
    pub contaminant: String,
    pub contaminant_conc_mg_kg: f64,
    pub groundwater_conc_mg_l: f64,
    pub land_use: String,
    pub has_residential_receptor: bool,
    pub depth_to_groundwater_m: f64,
    pub soil_organic_carbon_pct: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OilSpillResponseParam {
    pub spill_volume_ton: f64,
    pub oil_type: String,
    pub wind_speed_ms: f64,
    pub current_speed_ms: f64,
    pub sea_state: u8,
    pub distance_to_coast_km: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AquacultureWasteParam {
    pub fish_type: String,
    pub production_ton_year: f64,
    pub fcr: f64,
    pub feed_protein_pct: f64,
    pub feed_n_pct: f64,
    pub feed_p_pct: f64,
    pub water_body_volume_m3: f64,
    pub outflow_m3_s: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ForestCarbonParam {
    pub forest_type: String,
    pub area_ha: f64,
    pub tree_density_per_ha: f64,
    pub avg_dbh_cm: f64,
    pub avg_height_m: f64,
    pub soil_carbon_ton_ha: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CarbonRegistryParam {
    pub project_type: String,
    pub emission_reduction_ton_co2e: f64,
    pub vintage_year: u32,
    pub buyer: String,
    pub seller: String,
    pub price_rp_per_ton: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PesticideRunoffParam {
    pub pesticide: String,
    pub application_rate_kg_ha: f64,
    pub koc: f64,
    pub half_life_days: f64,
    pub rainfall_mm: f64,
    pub slope_pct: f64,
    pub soil_erodibility: f64,
    pub area_ha: f64,
    pub water_body_distance_m: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TailingsManagementParam {
    pub ore_type: String,
    pub tailings_volume_m3_day: f64,
    pub tailings_solid_pct: f64,
    pub dam_height_m: f64,
    pub dam_volume_m3: f64,
    pub supernatant_ph: f64,
    pub supernatant_metals_json: String,
    pub disposal_method: String,
    pub foundation_type: String,
    pub seismic_zone: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AwdGhgParam {
    pub area_ha: f64,
    pub water_management: String,
    pub rice_season: String,
    pub soil_type: String,
    pub n_fertilizer_kg_ha: f64,
    pub organic_amendment: String,
    pub climate_zone: String,
    pub duration_years: f64,
}


#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PfasTransportParam {
    pub pfas_type: String, pub conc_mg_l: f64, pub distance_m: f64,
    pub velocity_m_day: f64, pub dispersivity_m: f64, pub time_days: f64,
    pub foc_pct: f64, pub koc_l_kg: f64, pub water_saturation: f64,
    pub awi_area_m2_per_m3: f64, pub kaw_m: f64, pub gamma_max_mol_m2: f64,
    pub decay_rate_day: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PfasElectrochemParam {
    pub pfas_type: String, pub conc_mg_l: f64, pub volume_m3: f64,
    pub electrode_type: String, pub current_density_ma_cm2: f64,
    pub electrode_area_cm2: f64, pub target_removal_pct: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PfasScwoParam {
    pub pfas_conc_ppb: f64, pub feed_flow_m3_day: f64, pub cod_g_l: f64,
    pub target_temp_c: f64, pub target_pressure_mpa: f64, pub residence_time_s: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PfasFoamParam {
    pub pfas_type: String, pub conc_ug_l: f64, pub volume_m3: f64,
    pub gas_flow_lpm: f64, pub column_height_m: f64, pub column_diameter_m: f64,
    pub hrt_min: f64, pub n_stages: u32, pub co_surfactant: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PfasScreeningParam {
    pub pfas_type: String, pub conc_ng_l: f64, pub water_source: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PfasElectroNfParam {
    pub pfas_type: String, pub feed_conc_ng_l: f64, pub membrane_type: String,
    pub applied_voltage_v: f64, pub pressure_mpa: f64, pub flow_rate_lmh: f64,
    pub temperature_c: f64, pub treatment_goal_ng_l: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NanoTreatmentParam {
    pub contaminant: String, pub conc_mg_l: f64, pub volume_m3: f64,
    pub nanomaterial: String, pub dose_g: f64, pub contact_time_min: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BlockchainCreditParam {
    pub project_id: String, pub carbon_stock_ton_co2e: f64, pub baseline_ton: f64,
    pub price_rp_per_ton: f64, pub verification_body: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EdnaBiodiversityParam {
    pub sample_type: String, pub n_sites: u32, pub n_samples_per_site: u32,
    pub n_pcr_replicates: u32, pub detections_json: String, pub target_species: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PinnWaterParam {
    pub observations_json: String, pub domain_length_m: f64,
    pub velocity_m_s: f64, pub dispersion_m2_s: f64, pub decay_rate_s: f64, pub n_grid: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HybridPhysicsMlParam {
    pub observations_json: String, pub velocity_m_s: f64,
    pub dispersion_m2_s: f64, pub domain_length_m: f64, pub n_grid: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MlDispersionParam {
    pub emission_g_s: f64, pub wind_speed_m_s: f64, pub wind_dir_deg: f64,
    pub mixing_height_m: f64, pub distance_m: f64, pub land_use: String, pub receptor_height_m: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PmForecastParam {
    pub pm10_history_json: String, pub temp_c: f64,
    pub humidity_pct: f64, pub wind_speed_ms: f64, pub forecast_horizon_hr: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WwtpDigitalTwinParam {
    pub influent_bod_mg_l: f64, pub influent_cod_mg_l: f64, pub flow_m3_day: f64,
    pub mlss_mg_l: f64, pub do_mg_l: f64, pub temp_c: f64,
    pub volume_m3: f64, pub target_bod_mg_l: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MicroplasticDetectParam {
    pub sample_id: String, pub particle_count: u32,
    pub sizes_json: String, pub spectra_match_json: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TropomiEmissionParam {
    pub facility_lat: f64, pub facility_lon: f64, pub pollutant: String,
    pub vcd_molec_cm2: f64, pub background_vcd: f64,
    pub wind_speed_ms: f64, pub area_m2: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BlueCarbonMrvParam {
    pub mangrove_species: String, pub area_ha: f64, pub avg_dbh_cm: f64,
    pub avg_height_m: f64, pub tree_density_ha: f64, pub soil_carbon_ton_ha: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SatelliteComplianceParam {
    pub facility_name: String, pub lat: f64, pub lon: f64,
    pub parameter: String, pub measured_value: f64,
    pub regulatory_limit: f64, pub satellite_source: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WatershedTwinParam {
    pub watershed_area_km2: f64, pub pfas_source_kg_yr: f64,
    pub rainfall_mm_yr: f64, pub soil_kd_l_kg: f64,
    pub foc_pct: f64, pub river_flow_m3_s: f64, pub n_subbasins: u32,
}

// ====== Phase 3 Gap-Filler Params (Indonesia-specific audit) ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HazeTrajectoryParam {
    #[schemars(description = "Fire source latitude (degrees)")]
    pub fire_lat: f64,
    #[schemars(description = "Fire source longitude (degrees)")]
    pub fire_lon: f64,
    #[schemars(description = "Wind speed m/s (>= 0.5)")]
    pub wind_speed_m_s: f64,
    #[schemars(description = "Wind direction degrees (meteorological FROM)")]
    pub wind_dir_deg: f64,
    #[schemars(description = "Duration hours (1-168, max 7 days)")]
    pub duration_hours: f64,
    #[schemars(description = "PM2.5 emission rate g/s")]
    pub pm_emission_rate_g_s: f64,
    #[schemars(description = "Effective stack/plume height m")]
    pub stack_height_m: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct JakartaCoastalRiskParam {
    #[schemars(description = "Latitude")]
    pub lat: f64,
    #[schemars(description = "Longitude")]
    pub lon: f64,
    #[schemars(description = "Subsidence rate mm/yr (InSAR; Jakarta ~-75, Semarang ~-150)")]
    pub subsidence_rate_mm_yr: f64,
    #[schemars(description = "Groundwater extraction m3/day")]
    pub groundwater_extraction_m3_day: f64,
    #[schemars(description = "Distance to coast km")]
    pub distance_to_coast_km: f64,
    #[schemars(description = "Ground elevation m above MSL")]
    pub elevation_m: f64,
    #[schemars(description = "Planning horizon years (1-100)")]
    pub planning_horizon_years: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RiverApportionmentParam {
    #[schemars(description = "River length km")]
    pub river_length_km: f64,
    #[schemars(description = "River flow m3/s")]
    pub flow_m3_s: f64,
    #[schemars(description = "JSON array: [{\"name\":\"IPAL-X\",\"bod_kg_day\":100,\"distance_km\":10,\"type\":\"point\"}]")]
    pub sources_json: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CoastalErosionParam {
    pub shoreline_length_km: f64,
    pub sea_level_rise_m: f64,
    #[schemars(description = "Closure depth h* m (below MSL)")]
    pub closure_depth_m: f64,
    pub beach_width_m: f64,
    #[schemars(description = "Significant wave height Hs m")]
    pub wave_height_m: f64,
    #[schemars(description = "Wave period T s")]
    pub wave_period_s: f64,
    #[schemars(description = "Wave angle to shoreline degrees")]
    pub wave_angle_deg: f64,
    #[schemars(description = "Sand mining volume m3/yr")]
    pub sand_mining_m3_yr: f64,
    #[schemars(description = "Mangrove loss ha over shoreline")]
    pub mangrove_loss_ha: f64,
    #[schemars(description = "Planning horizon years")]
    pub planning_horizon_years: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SanitationImpactParam {
    pub population: u32,
    #[schemars(description = "Open defecation rate % (BABS rate)")]
    pub open_defecation_rate_pct: f64,
    #[schemars(description = "Septic tank coverage %")]
    pub septic_coverage_pct: f64,
    #[schemars(description = "Distance to river receptor m")]
    pub river_distance_m: f64,
    #[schemars(description = "Groundwater depth m (shallow <10m = high risk)")]
    pub groundwater_depth_m: f64,
    #[schemars(description = "River flow m3/s")]
    pub river_flow_m3_s: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct ValidateModelParam {
    #[schemars(description = "Model name (e.g. 'Streeter-Phelps DO model')")]
    pub model_name: String,
    #[schemars(description = "Predicted values from model (comma-separated)")]
    pub predicted: String,
    #[schemars(description = "Observed values from field measurement (comma-separated)")]
    pub observed: String,
    #[schemars(description = "Units (e.g. 'mg/L', 'µg/m³')")]
    pub units: String,
}


#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IonExchangeParam {
    #[schemars(description = "Target ion (ca2+, mg2+, na+, no3-, cl-, so42-, fe3+, cr6+)")]
    pub target_ion: String,
    #[schemars(description = "Influent concentration mg/L")]
    pub c_influent_mg_l: f64,
    #[schemars(description = "Resin exchange capacity eq/L (0.5-2.0 typical)")]
    pub exchange_capacity_eq_l: f64,
    #[schemars(description = "Flow m³/day")]
    pub flow_m3_day: f64,
    #[schemars(description = "Bed volume m³")]
    pub bed_volume_m3: f64,
    #[schemars(description = "Selectivity coefficient K (1-100)")]
    pub selectivity_coeff: f64,
    #[schemars(description = "Regenerant: nacl, hcl, naoh")]
    pub regenerant_type: String,
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
    #[schemars(
        description = "Activity: electricity_kwh, diesel, gasoline, lpg_kg, waste_ton, flight_km, vehicle_km, rice_paddy_ha, deforestation_ha"
    )]
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
    #[schemars(description = "Location in Indonesia (nama kota/kabupaten)")]
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
    #[schemars(description = "Judul Peta")]
    pub title: String,
    #[schemars(
        description = "Jika true, gunakan Sentinel-2 (30 hari terakhir) via GEE sebagai basemap"
    )]
    pub realtime_satellite: Option<bool>,
    #[schemars(description = "Nama pembuat peta (default: Environmental AI Agent)")]
    pub author: Option<String>,
    #[schemars(description = "Tanggal produksi (YYYY-MM-DD, default: hari ini)")]
    pub date: Option<String>,
    #[schemars(description = "Tampilkan batas administrasi (default: true)")]
    pub show_admin: Option<bool>,
}

// ====== Tool implementations ======

use rmcp::handler::server::wrapper::Parameters;

/// Parse "lat,lon" or "lat,lon,days" from query string, default to Indonesia center
fn parse_latlon_query(query: &str) -> (f64, f64, u32) {
    let parts: Vec<&str> = query.split(',').collect();
    let lat: f64 = parts
        .first()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(-8.65);
    let lon: f64 = parts
        .get(1)
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(116.35);
    let days: u32 = parts
        .get(2)
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(30);
    (lat, lon, days)
}

#[tool_router]
impl EnvIndonesiaServer {
    // --- DATA INDONESIA ---
    #[tool(description = "BMKG weather forecast for Indonesian cities")]
    async fn bmkg_weather(&self, Parameters(p): Parameters<LocationParam>) -> String {
        tools::data::bmkg::weather(&HTTP, &p.location).await
    }

    #[tool(description = "Integrated environment study: validate GeoJSON AOI, classify data sufficiency, optionally discover satellite fallback data, and run truthful flood/leachate/AMD baselines. Outputs structured JSON with evidence level, provenance, uncertainty status, and limitations.")]
    async fn integrated_environment_study(&self, Parameters(p): Parameters<tools::integrated_study::IntegratedStudyRequest>) -> String {
        let plan = match tools::integrated_study::plan_study(&p) {
            Ok(plan) => plan,
            Err(error) => return serde_json::json!({"status":"invalid_request","error":error}).to_string(),
        };
        let mut report = tools::integrated_study::run_baselines(&p, &plan);
        if p.satellite_fallback {
            report.satellite_discovery = Some(tools::integrated_study::discover_satellite_sources(&HTTP, &plan).await);
        }
        serde_json::to_string_pretty(&report).unwrap_or_else(|error| serde_json::json!({"status":"serialization_error","error":error.to_string()}).to_string())
    }

    #[tool(description = "Assess data maturity against the honesty ladder (insufficient_data, screening, conceptual, calibrated, validated). Returns allowed level, whether the request is blocked, and missing data requirements. Synthetic field data is capped and never reaches calibrated/validated.")]
    async fn assess_data_maturity(&self, Parameters(p): Parameters<MaturityParam>) -> String {
        let decision = crate::honesty::gate(crate::honesty::parse_level(&p.requested_level), &p.availability);
        serde_json::to_string(&decision).unwrap_or_default()
    }

    #[tool(description = "Record an external software computation (QGIS, SWMM, EPANET, GDAL, etc.) into the tamper-evident audit chain. Returns the audit event with its SHA-256 chain hash. External software output is untrusted execution: record it before treating any result as evidence.")]
    async fn record_computation(&self, Parameters(p): Parameters<ComputationParam>) -> String {
        crate::computation::record_json(&p.record)
    }

    #[tool(description = "Assess multi-source evidence: deduplicate claims by semantics, require at least two INDEPENDENT reporting lineages before corroborating, flag contradictions for human review, and abstain rather than concluding. A tier-1 official finding is sufficient alone. Never emits a legal or regulatory conclusion.")]
    async fn evidence_assess(&self, Parameters(p): Parameters<crate::evidence::EvidenceAssessmentRequest>) -> String {
        crate::evidence::assess_request(&p)
    }

    #[tool(description = "Simulate time-dependent PYRITE OXIDATION (acid mine drainage generation rate) with PHREEQC KINETICS using the Williamson & Rimstidt (1994) rate law. Answers how fast acid appears, which static ABA screening (MPA/NAPP) and equilibrium speciation cannot. Returns a pH / Fe / sulfate time series plus four guards: oxygen limitation (a sealed system stalls and its flat pH is an artifact), pyrite depletion (the physically real reason a curve flattens), sulfate-to-iron stoichiometry (FeS2 gives 2 S per Fe; iron precipitation breaks the link between dissolved Fe and oxidation extent), and an always-reported note that the rate is laboratory-derived. Capped at screening_only.")]
    async fn pyrite_oxidation_kinetics(&self, Parameters(p): Parameters<crate::pyrite_kinetics::PyriteKineticsRequest>) -> String {
        use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};

        let started_at = chrono::Utc::now().to_rfc3339();
        let run = match crate::pyrite_kinetics::run_pyrite_kinetics(&p).await {
            Ok(run) => run,
            Err(error) => {
                return ScientificResult::new("pyrite_kinetics_error", 0.0, "dimensionless")
                    .with_status(ResultStatus::ValidationFailed)
                    .with_claim(Claim::new("pyrite_kinetics_error", &error))
                    .with_limitation("pyrite oxidation kinetics did not execute; no acid generation rate was computed")
                    .emit_validated();
            }
        };
        let finished_at = chrono::Utc::now().to_rfc3339();

        let audit_event = crate::computation::record_json(&crate::computation::ComputationRecord {
            run_id: format!("pyrite-{}", &run.database_sha256[..16]),
            software: "phreeqc".to_string(),
            software_version: format!("kinetics/{}", run.database),
            tool_name: "pyrite_oxidation_kinetics".to_string(),
            arguments: serde_json::to_value(&p).unwrap_or(serde_json::Value::Null),
            input_sha256s: vec![run.database_sha256.clone()],
            output_sha256s: vec![],
            exit_code: 0,
            started_at,
            finished_at,
        });

        let guards = &run.guards;
        let mut result = ScientificResult::new("pyrite_oxidation_final_ph", guards.final_ph, "pH")
            .with_status(crate::honesty::to_result_status(
                crate::honesty::MaturityLevel::Screening,
            ))
            .with_provenance(Provenance::new(
                "model",
                &format!("phreeqc_kinetics/{}", run.database),
                &chrono::Utc::now().to_rfc3339(),
            ))
            .with_claim(Claim::new("rate_law", &run.rate_law))
            .with_claim(Claim::new("database_sha256", &run.database_sha256))
            .with_claim(Claim::new("audit_event", &audit_event))
            .with_claim(Claim::new(
                "simulated_days",
                &format!("{:.1}", guards.simulated_days),
            ))
            .with_claim(Claim::new(
                "ph_trajectory",
                &format!("{:.3} -> {:.3}", guards.initial_ph, guards.final_ph),
            ))
            .with_claim(Claim::new(
                "time_series",
                &serde_json::to_string(&run.series).unwrap_or_default(),
            ))
            .with_claim(Claim::new(
                "pyrite_consumed_fraction",
                &format!("{:?}", guards.pyrite_consumed_fraction),
            ))
            .with_limitation(
                "Laboratory-derived rate constant: field pyrite oxidation is commonly one to two orders of magnitude slower, so the absolute timescale is not calibrated",
            )
            .with_limitation(
                "Single well-mixed batch: no gas diffusion through waste rock, no unsaturated flow, no bacterial catalysis (Acidithiobacillus)",
            );

        if guards.oxygen_limited {
            result = result.with_limitation(&format!(
                "reaction stalled from oxygen exhaustion in a closed system (late pH change {:.5}); the flat pH is an artifact of the sealed box, not a stable long-term outcome",
                guards.late_ph_change
            ));
        }
        if guards.pyrite_depleted {
            result = result.with_claim(Claim::new(
                "pyrite_depleted",
                "all reactive pyrite was consumed within the simulated window",
            ));
        }
        if !guards.stoichiometry_consistent {
            result = result.with_limitation(&format!(
                "sulfate-to-iron ratio {:?} departs from the FeS2 value of 2, so iron was removed by secondary precipitation and dissolved Fe understates how much pyrite oxidised",
                guards.sulfate_to_iron_ratio
            ));
        }
        if !crate::pyrite_kinetics::trajectory_is_interpretable(&run) {
            result = result.with_limitation(
                "one or more kinetic guards failed; this trajectory describes the model setup rather than the waste rock and must not be read as an acid generation forecast",
            );
        }
        result.emit_validated()
    }

    #[tool(description = "Execute 1D advective-dispersive reactive transport through a mineral column using real PHREEQC TRANSPORT. Reports outlet pH/metals by pore volume and catches four failure modes: numerical dispersion dominating because grid Peclet > 2, an influent front that never traversed the column, reactive-buffer exhaustion, and the explicit assumption of full equilibrium at every cell (no kinetic limitation or preferential flow). Capped at screening_only.")]
    async fn reactive_transport(&self, Parameters(p): Parameters<crate::reactive_transport::ReactiveTransportRequest>) -> String {
        use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};

        let started_at = chrono::Utc::now().to_rfc3339();
        let run = match crate::reactive_transport::run_reactive_transport(&p).await {
            Ok(run) => run,
            Err(error) => {
                return ScientificResult::new("reactive_transport_error", 0.0, "dimensionless")
                    .with_status(ResultStatus::ValidationFailed)
                    .with_claim(Claim::new("reactive_transport_error", &error))
                    .with_limitation("reactive transport did not execute; no column result was computed")
                    .emit_validated();
            }
        };
        let finished_at = chrono::Utc::now().to_rfc3339();
        let audit_event = crate::computation::record_json(&crate::computation::ComputationRecord {
            run_id: format!("transport-{}", &run.database_sha256[..16]),
            software: "phreeqc".to_string(),
            software_version: "TRANSPORT".to_string(),
            tool_name: "reactive_transport".to_string(),
            arguments: serde_json::to_value(&p).unwrap_or(serde_json::Value::Null),
            input_sha256s: vec![run.database_sha256.clone()],
            output_sha256s: vec![],
            exit_code: 0,
            started_at,
            finished_at,
        });

        let final_ph = run.outlet_series.last().map(|step| step.ph).unwrap_or(0.0);
        let mut result = ScientificResult::new("reactive_transport_outlet_ph", final_ph, "pH")
            .with_status(crate::honesty::to_result_status(crate::honesty::MaturityLevel::Screening))
            .with_provenance(Provenance::new("model", &format!("phreeqc_transport/{}", run.database), &chrono::Utc::now().to_rfc3339()))
            .with_claim(Claim::new("database_sha256", &run.database_sha256))
            .with_claim(Claim::new("audit_event", &audit_event))
            .with_claim(Claim::new("outlet_series", &serde_json::to_string(&run.outlet_series).unwrap_or_default()))
            .with_claim(Claim::new("pore_volumes_flushed", &format!("{:.3}", run.guards.pore_volumes_flushed)))
            .with_limitation("1D column model: no 3D groundwater-flow coupling, preferential flow, or kinetic limitation")
            .with_limitation("Each cell is assumed to reach full thermodynamic equilibrium at every advective shift");

        if run.guards.numerical_dispersion_dominates {
            result = result.with_limitation(&format!(
                "numerical dispersion dominates: grid Peclet {:?} exceeds limit {:.1}; refine cells or increase physical dispersivity",
                run.guards.grid_peclet, run.guards.grid_peclet_limit
            ));
        }
        if !run.guards.front_traversed_column {
            result = result.with_limitation("influent front did not traverse the column; a clean outlet means the simulation was too short, not that the barrier works");
        }
        if run.guards.buffer_exhausted {
            result = result
                .with_claim(Claim::new("buffer_exhausted", &run.guards.exhausted_phases.join(", ")))
                .with_limitation("reactive buffer exhausted at the outlet; barrier breakthrough is a real result");
        }
        result.emit_validated()
    }

    #[tool(description = "Execute a REAL MODFLOW 6 groundwater flow model via FloPy (aquifer drawdown, wellfield sustainability, landfill/tailings groundwater). Units are fixed: metres and days; hydraulic conductivity is m/day. Reports heads, the volumetric budget, and four gates: convergence, MODFLOW's own percent discrepancy, dry-cell count (sentinel heads are excluded, not averaged), and whether wells were silently curtailed because their cell went dry. There is NO analytical fallback: a failed model is an error, never a substituted Theis estimate. Capped at screening_only.")]
    async fn modflow_groundwater(&self, Parameters(p): Parameters<crate::modflow_runner::ModflowRequest>) -> String {
        use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};

        let started_at = chrono::Utc::now().to_rfc3339();
        let run = match crate::modflow_runner::run_modflow(&p).await {
            Ok(run) => run,
            Err(error) => {
                return ScientificResult::new("modflow_groundwater_error", 0.0, "dimensionless")
                    .with_status(ResultStatus::ValidationFailed)
                    .with_claim(Claim::new("modflow_error", &error))
                    .with_limitation("MODFLOW did not execute; no groundwater head was computed")
                    .emit_validated();
            }
        };
        let finished_at = chrono::Utc::now().to_rfc3339();

        let arguments = serde_json::to_value(&p).unwrap_or(serde_json::Value::Null);
        let mut argument_hasher = <sha2::Sha256 as sha2::Digest>::new();
        sha2::Digest::update(&mut argument_hasher, arguments.to_string().as_bytes());
        let argument_sha256 = format!("{:x}", sha2::Digest::finalize(argument_hasher));

        let audit_event = crate::computation::record_json(&crate::computation::ComputationRecord {
            run_id: format!("modflow-{}", &argument_sha256[..16]),
            software: "modflow6".to_string(),
            software_version: run.mf6_version.clone(),
            tool_name: "modflow_groundwater".to_string(),
            arguments,
            input_sha256s: vec![argument_sha256],
            output_sha256s: vec![],
            exit_code: 0,
            started_at,
            finished_at,
        });

        // Report the largest drawdown across wells; fall back to 0 when every
        // well cell dried out (in which case the gate has already failed).
        let drawdown = run
            .wells
            .iter()
            .filter_map(|well| well.drawdown_m)
            .fold(f64::NEG_INFINITY, f64::max);
        let reported = if drawdown.is_finite() { drawdown } else { 0.0 };

        let mut result = ScientificResult::new("modflow_max_drawdown", reported, "m")
            .with_status(crate::honesty::to_result_status(
                crate::honesty::MaturityLevel::Screening,
            ))
            .with_provenance(Provenance::new(
                "model",
                &format!("modflow6/{}", run.mf6_version),
                &chrono::Utc::now().to_rfc3339(),
            ))
            .with_claim(Claim::new("converged", &run.converged.to_string()))
            .with_claim(Claim::new("units", &format!("{} / {}", run.units.length, run.units.time)))
            .with_claim(Claim::new("audit_event", &audit_event))
            .with_claim(Claim::new(
                "budget_gate",
                &format!(
                    "percent_discrepancy = {:?} (tolerance {:.3}%), gate_passed = {}",
                    run.gate.percent_discrepancy, run.gate.tolerance_pct, run.gate.gate_passed
                ),
            ))
            .with_claim(Claim::new(
                "dry_cells",
                &format!(
                    "{} dry / {} active",
                    run.heads.dry_cell_count, run.heads.active_cell_count
                ),
            ))
            .with_claim(Claim::new(
                "wells",
                &serde_json::to_string(&run.wells).unwrap_or_default(),
            ))
            .with_limitation(
                "Uncalibrated groundwater model: hydraulic conductivity, storage, and boundary heads were supplied, not fitted to observed heads",
            );

        if !run.converged {
            result = result.with_limitation(
                "MODFLOW did NOT converge; the reported heads are numerically meaningless and must not be used",
            );
        }
        if run.gate.wells_curtailed == Some(true) {
            result = result.with_limitation(&format!(
                "wells were curtailed because their cell went dry: {:?} m3 requested vs {:?} m3 delivered, so the requested extraction scenario never ran",
                run.gate.requested_extraction_m3, run.gate.delivered_extraction_m3
            ));
        }
        if run.heads.dry_cell_count > 0 {
            result = result.with_limitation(&format!(
                "{} cells went dry; head statistics exclude their sentinel values",
                run.heads.dry_cell_count
            ));
        }
        if run.gate.boundary_controlled == Some(true) {
            result = result.with_limitation(
                "most inflow came from the constant-head boundary, so drawdown reflects the chosen boundary location more than the aquifer",
            );
        }
        if let Some(discrepancy) = run.gate.percent_discrepancy {
            if discrepancy.abs() > run.gate.tolerance_pct {
                result = result.with_limitation(&format!(
                    "volumetric budget discrepancy {:.3}% exceeds the {:.3}% tolerance",
                    discrepancy, run.gate.tolerance_pct
                ));
            }
        }
        if !crate::modflow_runner::result_is_interpretable(&run) {
            result = result.with_limitation(
                "one or more MODFLOW gates failed; this result must not be interpreted as a groundwater prediction",
            );
        }
        result.emit_validated()
    }

    #[tool(description = "Execute a REAL PHREEQC geochemical speciation and optional lime-neutralisation titration (acid mine drainage, landfill leachate, tailings leachate). Reports dissolved metals before and after treatment plus saturation indices. Three honesty guards travel with the result: elements with no master species in the database are listed as unsupported instead of silently reported as 0 mg/L; specific conductance is null when the database cannot compute it; and phases that are supersaturated but were not equilibrated are flagged, marking the concentrations as upper bounds. Capped at screening_only.")]
    async fn phreeqc_speciation(&self, Parameters(p): Parameters<crate::phreeqc_runner::PhreeqcRequest>) -> String {
        use crate::result_contract::{Claim, Provenance, ResultStatus, ScientificResult};

        let started_at = chrono::Utc::now().to_rfc3339();
        let run = match crate::phreeqc_runner::run_phreeqc(&p).await {
            Ok(run) => run,
            Err(error) => {
                return ScientificResult::new("phreeqc_speciation_error", 0.0, "dimensionless")
                    .with_status(ResultStatus::ValidationFailed)
                    .with_claim(Claim::new("phreeqc_error", &error))
                    .with_limitation("PHREEQC did not execute; no geochemistry was computed")
                    .emit_validated();
            }
        };
        let finished_at = chrono::Utc::now().to_rfc3339();

        // Record the external PHREEQC invocation in the tamper-evident audit chain.
        let audit_event = crate::computation::record_json(&crate::computation::ComputationRecord {
            run_id: format!("phreeqc-{}", &run.database_sha256[..16]),
            software: "phreeqc".to_string(),
            software_version: format!("phreeqpython/{}", run.database),
            tool_name: "phreeqc_speciation".to_string(),
            arguments: serde_json::to_value(&p).unwrap_or(serde_json::Value::Null),
            input_sha256s: vec![run.database_sha256.clone()],
            output_sha256s: vec![],
            exit_code: 0,
            started_at,
            finished_at,
        });

        // The reported value is the treated pH when a titration ran, else the raw pH.
        let (reported_ph, stage) = match &run.treated {
            Some(treated) => (treated.ph, "after_lime_treatment"),
            None => (run.raw.ph, "raw_solution"),
        };
        let state = run.treated.as_ref().unwrap_or(&run.raw);

        let mut result = ScientificResult::new("phreeqc_ph", reported_ph, "pH")
            .with_status(crate::honesty::to_result_status(
                crate::honesty::MaturityLevel::Screening,
            ))
            .with_provenance(Provenance::new(
                "model",
                &format!("phreeqc/{}", run.database),
                &chrono::Utc::now().to_rfc3339(),
            ))
            .with_claim(Claim::new("stage", stage))
            .with_claim(Claim::new("database_sha256", &run.database_sha256))
            .with_claim(Claim::new("audit_event", &audit_event))
            .with_claim(Claim::new(
                "dissolved_metals_mg_l",
                &serde_json::to_string(&state.elements_mg_l).unwrap_or_default(),
            ))
            .with_claim(Claim::new(
                "saturation_indices",
                &serde_json::to_string(&state.saturation_indices).unwrap_or_default(),
            ))
            .with_claim(Claim::new(
                "lime_added_mmol",
                &format!("{:.3}", run.lime_added_mmol),
            ))
            .with_limitation(
                "Equilibrium thermodynamics only: no reaction kinetics, no reactive transport, no field validation",
            );

        if !run.unsupported_elements.is_empty() {
            result = result
                .with_claim(Claim::new(
                    "unsupported_elements",
                    &run.unsupported_elements.join(", "),
                ))
                .with_limitation(&format!(
                    "these elements have no master species in {} and were NOT modelled (reported 0 mg/L means not computed, not immobile): {}",
                    run.database,
                    run.unsupported_elements.join(", ")
                ));
        }
        if state.concentrations_are_upper_bounds {
            let phases: Vec<String> = state
                .supersaturated_but_unmodelled
                .iter()
                .map(|entry| format!("{} (SI {:.2})", entry.phase, entry.si))
                .collect();
            result = result
                .with_claim(Claim::new("supersaturated_but_unmodelled", &phases.join(", ")))
                .with_limitation(
                    "dissolved concentrations are UPPER BOUNDS: supersaturated phases were not equilibrated, so real precipitation would lower them",
                );
        }
        if let Some(note) = &state.sc_note {
            result = result.with_limitation(note);
        }
        if let Some(treated) = &run.treated {
            if treated.reached_target == Some(false) {
                result = result.with_limitation(
                    "lime titration did not reach the target pH within the step budget",
                );
            }
        }
        result.emit_validated()
    }

    #[tool(description = "Earned split-sample validation: contiguously split paired predicted/observed series (Klemes 1986), compute Moriasi et al. 2007 metrics on train and test partitions separately, and derive the maturity level the model actually EARNS (validated requires test NSE > 0.5, |PBIAS| < 25%, and at least 5 test points). Attaches a prediction interval from test residuals. A declared availability can only lower the result, never raise it.")]
    async fn calibrate_and_validate(&self, Parameters(p): Parameters<CalibrateValidateParam>) -> String {
        use crate::result_contract::{
            Claim, Provenance, ResultStatus, ScientificResult, Uncertainty, UncertaintyType,
        };

        let train_fraction = p.train_fraction.unwrap_or(0.7);
        let confidence_level = p.confidence_level.unwrap_or(0.95);
        let evidence = match crate::calibration::validate_split_sample(
            &p.predicted,
            &p.observed,
            train_fraction,
            confidence_level,
        ) {
            Ok(evidence) => evidence,
            Err(error) => {
                return ScientificResult::new("validation_error", 0.0, &p.unit)
                    .with_status(ResultStatus::ValidationFailed)
                    .with_claim(Claim::new("validation_error", &error))
                    .with_limitation("split-sample validation did not execute")
                    .emit_validated();
            }
        };

        let earned = crate::calibration::earned_level(&evidence);
        let level = match &p.availability {
            Some(availability) => {
                crate::honesty::assess_level_with_evidence(availability, Some(earned))
            }
            None => earned,
        };

        let point_estimate = p.point_estimate.unwrap_or_else(|| {
            let train_n = (p.predicted.len() as f64 * train_fraction).floor() as usize;
            let test = &p.predicted[train_n..];
            if test.is_empty() { 0.0 } else { test.iter().sum::<f64>() / test.len() as f64 }
        });
        let (lower, upper) = crate::calibration::prediction_interval(point_estimate, &evidence);

        let mut result = ScientificResult::new("validated_prediction", point_estimate, &p.unit)
            .with_status(crate::honesty::to_result_status(level))
            .with_uncertainty(Uncertainty {
                uncertainty_type: UncertaintyType::PredictionInterval,
                lower,
                upper,
                method: evidence.split_method.clone(),
                confidence_level: Some(confidence_level),
                seed: None,
            })
            .with_provenance(Provenance::new(
                "model_validation",
                &p.model_name,
                &chrono::Utc::now().to_rfc3339(),
            ))
            .with_claim(Claim::new("earned_level", &format!("{:?}", earned).to_lowercase()))
            .with_claim(Claim::new(
                "test_partition",
                &format!(
                    "n={} NSE={:.4} PBIAS={:.2}% RMSE={:.4} KGE={:.4}",
                    evidence.test.n, evidence.test.nse, evidence.test.pbias, evidence.test.rmse, evidence.test.kge
                ),
            ))
            .with_claim(Claim::new(
                "train_partition",
                &format!(
                    "n={} NSE={:.4} PBIAS={:.2}%",
                    evidence.train.n, evidence.train.nse, evidence.train.pbias
                ),
            ))
            .with_claim(Claim::new("thresholds", &evidence.thresholds))
            .with_limitation("Metrics assume paired observations at matching time and location")
            .with_limitation("No temporal or spatial autocorrelation diagnostic was performed");

        if earned != crate::honesty::MaturityLevel::Validated {
            result = result.with_limitation(
                "test partition did not clear the Moriasi satisfactory bar; result is not independently validated",
            );
        }
        result.emit_validated()
    }

    #[tool(description = "1D->2D flood coupling: run a real EPA SWMM model, convert each flooding node's surcharge volume into an equivalent steady 2D point inflow, solve the 2D shallow-water equations, then apply a mass-balance gate comparing 1D surcharge volume against 2D injected volume. Result is capped at screening_only (never valid) because the overland extent is not validated against observed flood extent. A failed gate is reported with an explicit do-not-use limitation.")]
    async fn swmm_1d2d_coupling(&self, Parameters(p): Parameters<SwmmCouplingParam>) -> String {
        // DEM orientation matches integrated_study::run_flood: outer index is y (rows), inner is x.
        let ny = p.dem.len();
        let nx = p.dem.first().map_or(0, Vec::len);
        if nx < 3 || ny < 3 || p.dem.iter().any(|row| row.len() != nx) {
            return crate::coupling::coupling_failure(
                "DEM must be a rectangular grid with at least 3x3 cells",
            )
            .emit_validated();
        }

        let started_at = chrono::Utc::now().to_rfc3339();
        let run = match crate::swmm_runner::run_swmm(&p.inp_path, p.timeout_secs.unwrap_or(120)).await {
            Ok(run) => run,
            Err(error) => return crate::coupling::coupling_failure(&error).emit_validated(),
        };
        let finished_at = chrono::Utc::now().to_rfc3339();

        // Record the external SWMM invocation in the tamper-evident audit chain so
        // the provenance claim on the coupled result is backed by a real event.
        let audit_event = crate::computation::record_json(&crate::computation::ComputationRecord {
            run_id: format!("swmm-{}", &run.inp_sha256[..16]),
            software: "epa_swmm".to_string(),
            software_version: format!("pyswmm {}", run.pyswmm_version),
            tool_name: "swmm_1d2d_coupling".to_string(),
            arguments: serde_json::json!({
                "inp_path": p.inp_path,
                "duration_s": p.duration_s,
                "dx_m": p.dx_m,
                "node_mapping_count": p.node_mapping.len(),
            }),
            input_sha256s: vec![run.inp_sha256.clone()],
            output_sha256s: vec![],
            exit_code: 0,
            started_at,
            finished_at,
        });

        let sources = match crate::coupling::build_sources(&run, &p.node_mapping, p.duration_s) {
            Ok(sources) => sources,
            Err(error) => return crate::coupling::coupling_failure(&error).emit_validated(),
        };

        let params = tools::advanced_physics::swe_solver::SweParams {
            nx,
            ny,
            dx: p.dx_m,
            manning_n: p.manning_n,
            duration_s: p.duration_s,
            dt_max: p.dt_max_s,
            second_order: false,
        };
        // duty_fraction = 1.0: the equivalent discharge was derived over the full
        // window, so injecting for the whole window reproduces the 1D volume.
        let swe = tools::advanced_physics::swe_solver::solve_multi_source(&p.dem, &params, &sources, 1.0);

        let tolerance = p
            .mass_tolerance_pct
            .unwrap_or(crate::coupling::DEFAULT_MASS_TOLERANCE_PCT);
        let gate = crate::coupling::check_mass_balance(
            run.routing.flooding_m3,
            swe.total_volume_m3,
            tolerance,
        );

        crate::coupling::coupling_result(&gate, swe.max_depth, swe.flooded_cells)
            .with_claim(crate::result_contract::Claim::new("audit_event", &audit_event))
            .with_claim(crate::result_contract::Claim::new(
                "routing_continuity",
                &format!("swmm routing_error_pct = {:.4}", run.routing.routing_error_pct),
            ))
            .emit_validated()
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
        tools::satellite::copernicus::search(&HTTP, &p.collection, p.limit.unwrap_or(5), p.bbox)
            .await
    }

    #[tool(description = "STAC Search — Microsoft Planetary Computer (135 collections, NO API key) or Element 84 Earth Search (9 collections). Free satellite data: Sentinel-1/2/3/5P, Landsat, MODIS, GPM IMERG, ESA WorldCover, Planet NICFI 4.77m, DEM. Covers Indonesia. Direct HTTPS download URLs included.")]
    async fn stac_search(&self, Parameters(p): Parameters<StacSearchParam>) -> String {
        let api = p.api.as_deref().unwrap_or("mpc");
        let limit = p.limit.unwrap_or(10).min(100);
        tools::satellite::stac::search(&HTTP, api, &p.collection, &p.bbox, &p.datetime, limit).await
    }

    #[tool(description = "STAC List Collections — list all available satellite collections. MPC: 135 collections (Sentinel, Landsat, MODIS, GPM, ERA5, ESA WorldCover, Planet NICFI, DEM, etc). Earth Search: 9 collections. No API key required.")]
    async fn stac_collections(&self, Parameters(p): Parameters<StacListParam>) -> String {
        let api = p.api.as_deref().unwrap_or("mpc");
        tools::satellite::stac::list_collections(&HTTP, api).await
    }

    #[tool(description = "STAC Describe Collection — get details: spatial/temporal extent, assets, license, providers, resolution. Input: collection ID (e.g. sentinel-2-l2a, sentinel-5p-l2-netcdf, planet-nicfi-visual, gpm-imerg-hhr, esa-worldcover)")]
    async fn stac_describe(&self, Parameters(p): Parameters<StacCollectionParam>) -> String {
        let api = p.api.as_deref().unwrap_or("mpc");
        tools::satellite::stac::describe_collection(&HTTP, api, &p.collection).await
    }

    #[tool(description = "STAC Get Asset URL — get direct download URL for a satellite scene asset. No API key. MPC assets are signed HTTPS URLs (valid 1 hour). Use after stac_search to download specific bands/images.")]
    async fn stac_asset_url(&self, Parameters(p): Parameters<StacAssetParam>) -> String {
        let api = p.api.as_deref().unwrap_or("mpc");
        tools::satellite::stac::get_asset_url(&HTTP, api, &p.collection, &p.item_id, &p.asset_key).await
    }

    #[tool(description = "Download and hash one STAC raster asset. Retrieval is validated; scientific interpretation is not performed.")]
    async fn stac_download_asset(&self, Parameters(p): Parameters<StacDownloadParam>) -> String {
        tools::satellite::stac::download_asset(&HTTP, p.api.as_deref().unwrap_or("mpc"), &p.collection, &p.item_id, &p.asset_key, &p.output_dir).await.unwrap_or_else(|e| e)
    }



    #[tool(description = "Flood SAR Mapping — Sentinel-1 VV change detection. Cloud-penetrating radar, 10m, 6-day revisit. Searches pre and post flood scenes, provides VV band download URLs + flood analysis protocol. Ref: Twele et al. 2016; Cian et al. 2018. Bypasses cloud cover limitation.")]
    async fn flood_sar_mapping(&self, Parameters(p): Parameters<FloodSarParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::satellite::flood_sar::search_flood_scenes(
            &HTTP, p.lat, p.lon, p.buffer_km.unwrap_or(10.0), &p.flood_date
        ).await
    }

    #[tool(description = "Karhutla Assessment — Sentinel-2 dNBR burned area severity + peat fire ID. Searches pre/post fire S2 scenes (NIR B08 + SWIR2 B12). Severity: Key & Benson 2006 (USGS). Peat proxy: DEMNAS elev<50m + slope<2° + FIRMS sustained FRP. Ref: Hooijer et al. 2012; Page et al. 2002.")]
    async fn karhutla_assessment(&self, Parameters(p): Parameters<KarhutlaParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::satellite::karhutla::assess_karhutla(
            &HTTP, p.lat, p.lon, p.buffer_km.unwrap_or(10.0), &p.fire_date
        ).await
    }

    #[tool(description = "Coral Bleaching Alert — NOAA Coral Reef Watch DHW (Degree Heating Weeks) real-time query. 5km resolution, 12-week cumulative heat stress. DHW>4=bleaching, >8=mortality. 2024=fourth global mass bleaching. Indonesia=Coral Triangle (76% world species). Ref: Goreau & Hayes 2024; Lachs et al. 2024; Festo et al. 2026.")]
    async fn coral_dhw_alert(&self, Parameters(p): Parameters<CoralAlertParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::ocean::coral_dhw::query_dhw(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "Climate Projection — NEX-GDDP-CMIP6 NASA downscaled (25km, bias-corrected). Scenarios: SSP2-4.5 (moderate ~3°C) or SSP5-8.5 (worst ~4.5°C). Period: 2030/2050/2080. Variables: tasmax, tasmin, pr. For AMDAL climate chapter, infrastructure planning, adaptation. Ref: Thrasher et al. 2022; Eyring et al. 2016.")]
    async fn climate_projection(&self, Parameters(p): Parameters<ClimateProjParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::data::climate_projection::search_climate_projection(
            &HTTP, p.lat, p.lon,
            p.scenario.as_deref().unwrap_or("ssp585"),
            p.period.as_deref().unwrap_or("2050")
         ).await
     }

    #[tool(description = "Peat Fire CO2 Emission — IPCC 2013 Wetlands Supplement. Input: burned area (ha), peat depth (m), severity. Peat fire = 10x more CO2 than mineral soil. Ref: Hooijer et al. 2012; Page et al. 2002.")]
    fn peat_co2_emission(&self, Parameters(p): Parameters<PeatCo2Param>) -> String {
        tools::calculators::peat_co2::calculate(p.burned_area_ha, p.peat_depth_m, &p.severity)
    }

    #[tool(description = "Heavy Metal Risk Assessment — HPI (Mohsen 1989) + US EPA RAGS health risk. Input: Pb, Cd, Hg, As, Cr concentrations. Baku mutu PP 22/2021. HQ + ILCR carcinogenic risk. Ref: Diansyah et al. 2025; Ramadhani et al. 2026.")]
    fn heavy_metal_risk(&self, Parameters(p): Parameters<HeavyMetalParam>) -> String {
        tools::calculators::heavy_metal_risk::assess(
            p.pb, p.cd, p.hg, p.as_, p.cr,
            p.body_weight_kg.unwrap_or(70.0),
            p.intake_l_per_day.unwrap_or(2.0),
            p.exposure_years.unwrap_or(30.0),
        )
    }

    #[tool(description = "Water Pollution Index — KepMen LH 115/2003. Input: BOD, COD, DO, TSS, coliform, class (1-4). PI = sqrt(max(Ci/Lij) × avg(Ci/Lij)). Also STORET. Baku mutu PP 22/2021. Ref: Marselina et al. 2025; Hidayati et al. 2025.")]
    fn water_pollution_index(&self, Parameters(p): Parameters<PollutionIndexParam>) -> String {
        tools::water::pollution_index::calculate(
            p.bod, p.cod, p.do_, p.tss, p.total_coliform, p.class.unwrap_or(2)
        )
    }

    #[tool(description = "ASGM Mercury Assessment — Hg mass balance + health risk. Input: Hg in water/sediment, gold production, population. UNEP 2013 method. Baku mutu PP 22/2021. Minamata Convention. Ref: Agustiani et al. 2025; Desmaiani et al. 2026.")]
    fn asgm_mercury_assessment(&self, Parameters(p): Parameters<AsgmMercuryParam>) -> String {
        tools::calculators::asgm_mercury::assess(
            p.hg_conc_water, p.hg_conc_sediment,
            p.gold_production_kg_yr, p.population_exposed
        )
    }

    #[tool(description = "Climate Vulnerability Index — IPCC AR5/AR6. V = Exposure × Sensitivity × (1-Adaptive Capacity). Input: climate change, elevation, population, poverty, GDP, literacy. Ref: Onat et al. 2025; Padaliya et al. 2025; Kumar et al. 2025.")]
    fn climate_vulnerability_index(&self, Parameters(p): Parameters<ClimateVulnerabilityParam>) -> String {
        tools::calculators::climate_vulnerability::calculate(
            p.temp_change_c, p.precip_change_pct, p.extreme_event_freq,
            p.elevation_m, p.population_density, p.poverty_rate,
            p.gdp_per_capita_usd, p.literacy_rate
        )
    }

    #[tool(description = "Mining Impact Assessment — Screening tool for nickel/coal/gold/tin. Leopold-style impact matrix + mine-specific profiles. Input: mine type, area, deforestation, water pollution, tailings, AMD, social. Ref: Pambudi 2025; Rosada 2025; Nasution 2024; Manurung 2025.")]
    fn mine_impact_assessment(&self, Parameters(p): Parameters<MineImpactParam>) -> String {
        tools::calculators::mine_impact::assess(
            &p.mine_type, p.lat, p.lon, p.area_ha, p.deforestation_ha,
            &p.water_pollution_level, p.has_tailings, p.has_amd, p.social_displacement
        )
    }

    #[tool(description = "Tidal Flood Compound — SLR + Subsidence + Tide (bathtub model). IPCC AR6. Open-Meteo Marine tide + Copernicus DEM 30m. Compound flood = max(SLR + tide - ground_elev + subsidence, 0). Ref: Shan et al. 2025 (Nature); Momin et al. 2026; Chrysanti et al. 2024.")]
    async fn tidal_flood_compound(&self, Parameters(p): Parameters<TidalFloodParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: {}", e);
        }
        tools::advanced_physics::tidal_flood::assess(
            &HTTP, p.lat, p.lon,
            p.slr_scenario.as_deref().unwrap_or("ssp585"),
            p.subsidence_rate_mm_yr,
            p.projection_year.unwrap_or(2050)
        ).await
    }

    #[tool(description = "Landslide Susceptibility — Frequency Ratio (FR) method. DEM slope + curvature + rainfall threshold. AUC 0.80-0.90 globally. Ref: Tirsyayu 2025 (Sulsel); Gnagne 2025 (Ivory Coast); Akhil 2025 (Wayanad AUC=0.896); Akbar 2025 (Japan).")]
    async fn landslide_susceptibility(&self, Parameters(p): Parameters<LandslideParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: {}", e);
        }
        tools::gis::landslide::assess(
            &HTTP, p.lat, p.lon, p.buffer_km.unwrap_or(10.0), p.rainfall_mm
        ).await
    }

    #[tool(description = "GPM IMERG Rainfall — 30-min precipitation (0.1°, ~10km). ⚠️ Tropical bias -41% (Watters 2025 NASA GPM). Valid for monthly/seasonal, NOT hourly flood. Ref: Setiyowati 2025; Lufira 2026 (bias correction).")]
    async fn gpm_imerg_rainfall(&self, Parameters(p): Parameters<GpmImergParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: {}", e);
        }
        tools::satellite::gpm_imerg::query(&HTTP, p.lat, p.lon, &p.date).await
    }

    #[tool(description = "Tropical Cyclone Track — ECMWF trajectory forecast (type=tf). Track error 100-350km at 24h. GRIB2 data. CC-BY-4.0. Indonesia rarely affected (NTT/Maluku). BMKG authoritative. Ref: Yang et al. 2025/2026; DeMaria et al. 2025.")]
    async fn tropical_cyclone_track(&self, Parameters(p): Parameters<CycloneParam>) -> String {
        tools::data::cyclone::search(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "Plastic Leakage Estimate — Jambeck et al. 2015 (Science). Population × waste × plastic × mismanaged → tons/year to sea. Indonesia #2 globally. Target 70% reduction by 2025. Ref: Nursyahputra 2026; Anuar 2025; Adnan 2025.")]
    fn plastic_leakage_estimate(&self, Parameters(p): Parameters<PlasticLeakageParam>) -> String {
        tools::ocean::plastic_leakage::estimate(
            p.population, p.waste_generation_kg_cap_day,
            p.plastic_fraction_pct, p.mismanaged_waste_pct, p.coastal_population_pct
        )
    }

    #[tool(description = "VIIRS Fishing Detection — Night light fishing boat detection. VIIRS DNB ~750m. VBD algorithm (Elvidge). Overlay MPA → illegal fishing. Cloud blocks detection. Ref: Elvidge et al. 2024 (SE Asia); Wang et al. 2025; Li et al. 2024.")]
    async fn viirs_fishing_detection(&self, Parameters(p): Parameters<ViirsFishingParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: {}", e);
        }
        tools::satellite::viirs_fishing::search(&HTTP, p.lat, p.lon, &p.date).await
    }

    #[tool(description = "Air quality: current AQI and concentrations of PM2.5, PM10, NO2, O3, SO2, CO for a location (Open-Meteo CAMS, free, no key). Concentrations in µg/m³; AQI 0-500. Baku mutu PP 22/2021: PM2.5 24h 65 µg/m³, PM10 24h 150 µg/m³. Limitation: regional reanalysis, not station-grade.")]
    async fn air_pollution(&self, Parameters(p): Parameters<LatLonParam>) -> String {
        let lat = p.lat.unwrap_or(-6.2);
        let lon = p.lon.unwrap_or(106.85);
        if let Err(e) = crate::indonesia::validate_coords(lat, lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::data::openweather::air_pollution(&HTTP, lat, lon).await
    }

    #[tool(description = "Weather 7-day forecast for a location: temperature (°C), precipitation (mm), wind speed/direction (m/s). Source Open-Meteo, free, no API key. Limitation: coarse grid; use BMKG for legal/AMDAL wind rose or AERMOD met input.")]
    async fn open_meteo_weather(&self, Parameters(p): Parameters<LatLonParam>) -> String {
        let lat = p.lat.unwrap_or(-6.2);
        let lon = p.lon.unwrap_or(106.85);
        if let Err(e) = crate::indonesia::validate_coords(lat, lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::data::open_meteo::weather(&HTTP, lat, lon).await
    }

    #[tool(description = "NASA POWER solar irradiance for a location: GHI, DNI, DHI (kWh/m²/day or W/m²) monthly/daily. Use for solar PV energy potential & renewable feasibility. Free, no key. Limitation: satellite-derived; validate with ground pyranometer for bankable reports.")]
    async fn nasa_power_solar(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::data::nasa_power::solar(&HTTP, p.lat, p.lon, None, None).await
    }

    #[tool(description = "Search Satu Data Indonesia (data.go.id) for official environmental datasets (climate, forest, pollution, water). Returns dataset title, publisher, URL. Use as authoritative BPS/KLHK/BMKG source for AMDAL & regulatory reporting.")]
    async fn satu_data_search(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::data::satu_data::search(&HTTP, &p.query, 5).await
    }

    #[tool(description = "Climate TRACE GHG emissions for Indonesia by sector (energy, industry, agriculture, forestry, transport) in tCO2e. Use for Scope 1/3 benchmarking & NDC gap analysis. Ref: Climate TRACE 2024 inventory.")]
    async fn climate_trace_emissions(&self, Parameters(p): Parameters<SectorParam>) -> String {
        tools::data::climate_trace::emissions(&HTTP, p.sector).await
    }

    // --- GIS ANALYSIS ---
    #[tool(description = "NDVI vegetation index from Sentinel-2 NIR (B8) and Red (B4) bands. NDVI=(NIR-Red)/(NIR+Red), valid range -1.0 to 1.0. Interpretation: <0 water/cloud, 0-0.2 bare, 0.2-0.5 sparse, >0.5 dense vegetation. Ref: Rouse et al. 1974.")]
    fn ndvi_compute(&self, Parameters(p): Parameters<NdviParam>) -> String {
        tools::gis::ndvi::compute(p.nir, p.red)
    }

    #[tool(description = "Water quality indices from Sentinel-2 bands: NDWI (water detection), turbidity proxy (Red/Green), chlorophyll-a proxy (eutrophication). For danau/sungai/pesisir Indonesia. Limitation: empirical proxies need in-situ calibration (SNI 6989) for legal reporting.")]
    fn water_quality(&self, Parameters(p): Parameters<WaterQualityParam>) -> String {
        tools::gis::water::quality(p.green, p.red, p.nir, None)
    }

    #[tool(description = "Drought index SPI (Standardized Precipitation Index) from precipitation time series. SPI: -2 extreme drought, -1.5 severe, -1 moderate, 0 normal, +2 wet. Ref: McKee et al. 1993. Use for kemarau/El Niño early warning & AMDAL hydrology.")]
    fn drought_index(&self, Parameters(p): Parameters<DroughtParam>) -> String {
        tools::gis::drought::index(p.precipitation_mm, p.avg_mm, p.std_mm)
    }

    #[tool(description = "Analyze GeoJSON: report geometry type, feature count, coordinate range, CRS. Use to validate GIS layers before raster/vector processing (e.g. AMDAL map inputs, watershed boundaries).")]
    fn geojson_analyze(&self, Parameters(p): Parameters<GeoJsonParam>) -> String {
        tools::gis::geojson_ops::analyze(&p.geojson)
    }

    #[tool(
        description = "[LEGACY → use coordinate_transform_v2 or wgs84_to_utm] Coordinate transform. direction: wgs84_to_utm, utm_to_wgs84, or EPSG code"
    )]
    fn coordinate_transform(&self, Parameters(p): Parameters<CoordParam>) -> String {
        match p.direction.as_str() {
            "wgs84_to_utm" => tools::gis::coords::wgs84_to_utm_auto(p.y, p.x),
            "utm_to_wgs84" => tools::gis::coords::utm_to_wgs84(p.x, p.y, "EPSG:32750"),
            _ => tools::gis::coords::transform(p.x, p.y, "EPSG:4326", &p.direction),
        }
    }

    // --- ESG ANALYTICS ---
    #[tool(
        description = "Carbon footprint calculator with Indonesia emission factors (IPCC + Perpres 98/2021)"
    )]
    fn carbon_calculator(&self, Parameters(p): Parameters<CarbonParam>) -> String {
        tools::esg::carbon::calculate(&p.activity, p.amount)
    }

    #[tool(description = "Map an activity/project to relevant UN SDGs (17 goals). Returns matched goals with rationale. Use for ESG sustainability reporting & AMDAL social chapter. Ref: UN Agenda 2030.")]
    fn sdg_mapper(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::esg::sdg::map_activity(&p.query)
    }

    #[tool(
        description = "OJK POJK 51/2017 ESG compliance checker for Indonesian financial institutions"
    )]
    fn ojk_compliance(&self, Parameters(p): Parameters<OjkParam>) -> String {
        tools::esg::ojk::check_compliance(&p.entity_type, &p.disclosures)
    }

    #[tool(description = "TCFD climate risk assessment for Indonesian sectors")]
    fn climate_risk_tcfd(&self, Parameters(p): Parameters<TcfdParam>) -> String {
        tools::esg::tcfd::risk_assessment(&p.sector, &p.location)
    }

    // --- OCEAN & MARINE ---
    #[tool(
        description = "Coral reef health Indonesia: 15 reef sites, 590 coral species. Opsional: lat/lon untuk cari reef terdekat."
    )]
    fn coral_reef_health(&self, Parameters(p): Parameters<CoralReefParam>) -> String {
        tools::ocean::coral::reef_health(p.lat, p.lon, p.n)
    }

    #[tool(
        description = "Marine protected areas Indonesia: 16+ KKP, 28.4 juta ha. Opsional: lat/lon untuk cari MPA terdekat."
    )]
    fn marine_protected_areas(&self, Parameters(p): Parameters<MpaParam>) -> String {
        tools::ocean::mpa::protected_areas(p.lat, p.lon, p.n)
    }

    // --- WRAPPERS (Existing Projects) ---
    #[tool(description = "ESG Audit pipeline (GeoESG-Final service, Port 8000). Computes Environmental/Social/Governance score from company activity data. Use for corporate ESG disclosure & OJK POJK 51 compliance. Limitation: requires the GeoESG service to be running.")]
    async fn wrapper_esg_audit(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::wrappers::trigger_esg_audit(&HTTP, &p.query).await
    }

    #[tool(description = "Flood prediction (geo-flood-ai service, Port 8001). Returns flood risk/zone for a location from ML model. Use for rapid screening only — verify with hydrologic model (SCS-CN/SWE) for legal flood maps. Limitation: requires geo-flood-ai service running.")]
    async fn wrapper_flood_predict(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::wrappers::predict_flood(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "Methane plume data (service, Port 8002). Retrieves CH4 concentration/plume detection for a location. Use for fugitive methane (TPA, gas facilities, peatland) screening. Limitation: requires the methane service running.")]
    async fn wrapper_methane_plumes(&self) -> String {
        tools::wrappers::get_methane_plumes(&HTTP).await
    }

    #[tool(description = "Groundwater monitoring status (service, Port 8003). Returns water-table depth/level for a location. Use for peatland TMAT (PP 71/2014 threshold 40cm) & land subsidence screening. Limitation: requires the groundwater service running.")]
    async fn wrapper_groundwater(&self) -> String {
        tools::wrappers::get_groundwater_status(&HTTP).await
    }

    #[tool(description = "Air quality monitoring health data (service, Port 8004). Returns station AQI/health status for a location. Use for ISPU & public health screening. Limitation: requires the air quality service running.")]
    async fn wrapper_air_quality(&self) -> String {
        tools::wrappers::get_air_quality(&HTTP).await
    }

    // --- SATELLITE TOOLS ---
    #[tool(description = "Status Gunung Api Indonesia dari MAGMA Indonesia")]
    async fn magma_volcano(&self) -> String {
        tools::data::magma::status(&HTTP).await
    }

    #[tool(
        description = "BPS Environmental Statistics Indonesia. keyword: hutan/sampah/air/ekonomi"
    )]
    async fn bps_environment(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::data::bps::statistics(&HTTP, &p.query).await
    }

    #[tool(description = "InaRISK BNPB Disaster Risk Assessment for any Indonesian location")]
    async fn inarisk_hazard(&self, Parameters(p): Parameters<LocationParam>) -> String {
        tools::data::inarisk::disaster_risk(&HTTP, &p.location).await
    }

    #[tool(description = "USGS Landsat STAC search by lat/lon. Query: lat,lon or location name.")]
    async fn satellite_landsat(&self, Parameters(p): Parameters<QueryParam>) -> String {
        // Parse lat,lon from query or default to NTB center
        let (lat, lon, days) = parse_latlon_query(&p.query);
        tools::satellite::landsat::search(&HTTP, lat, lon, days).await
    }

    #[tool(description = "NASA MODIS products information for environmental monitoring.")]
    async fn satellite_modis(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR: {}", e);
        }
        tools::satellite::modis::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "NASA VIIRS products information (Nighttime lights, active fires).")]
    async fn satellite_viirs(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR: {}", e);
        }
        tools::satellite::viirs::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "SRTM 30m Digital Elevation Model for Indonesia")]
    async fn satellite_srtm(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR: {}", e);
        }
        tools::satellite::srtm::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "CHIRPS Rainfall data (real HTTP query). Query: year,month (e.g. 2024,6)")]
    async fn satellite_chirps(&self, Parameters(p): Parameters<QueryParam>) -> String {
        let parts: Vec<&str> = p.query.split(',').collect();
        let year: u32 = parts
            .first()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(2024);
        let month: u32 = parts
            .get(1)
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(1);
        tools::satellite::chirps::query(&HTTP, year, month).await
    }

    #[tool(description = "NASA GRACE / GRACE-FO Groundwater Storage anomaly information.")]
    async fn satellite_grace(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR: {}", e);
        }
        tools::satellite::grace::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "Google Dynamic World 10m near real-time land cover info.")]
    async fn satellite_dynamic_world(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR: {}", e);
        }
        tools::satellite::dynamic_world::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "ECMWF ERA5 Climate Reanalysis information for long-term trends.")]
    async fn satellite_era5(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR: {}", e);
        }
        tools::satellite::era5::query(&HTTP, p.lat, p.lon).await
    }

    // --- ADVANCED GIS & ESG ---
    #[tool(description = "Parse Sustainability Report (PDF) for ESG Analytics.")]
    async fn esg_report_parser(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::esg::report_parser::parse_esg_report(&p.query).await
    }

    #[tool(
        description = "[LEGACY → use dem_slope_gee] DEM Slope via GEE SRTM. Query: lat,lon,buffer_km,output_path"
    )]
    fn gis_dem_slope(&self, Parameters(p): Parameters<QueryParam>) -> String {
        let parts: Vec<&str> = p.query.split(',').collect();
        let lat: f64 = parts
            .first()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(-8.65);
        let lon: f64 = parts
            .get(1)
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(116.35);
        let buffer_km: f64 = parts
            .get(2)
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(10.0);
        let output_path = parts.get(3).map(|s| s.trim()).unwrap_or("/tmp/slope.tif");
        tools::gis::advanced::dem_slope(lat, lon, buffer_km, output_path)
    }

    #[tool(
        description = "[LEGACY → use zonal_statistics_gee] Zonal Raster Statistics via GEE. Input: GeoJSON polygon."
    )]
    fn gis_raster_stats(&self, Parameters(p): Parameters<GeoJsonParam>) -> String {
        tools::gis::advanced::raster_stats(
            "USGS/SRTMGL1_003",
            "elevation",
            &p.geojson,
            -8.65,
            116.35,
            10.0,
            "/tmp/zonal_stats.json",
        )
    }

    #[tool(
        description = "[LEGACY → use land_cover_classify] Land Cover Classifier via GEE Sentinel-2."
    )]
    fn gis_land_cover_classifier(&self) -> String {
        match tools::gis::landcover::classify(
            -8.65,
            116.35,
            10.0,
            "2023-01-01",
            "2023-12-31",
            "/tmp/landcover.tif",
        ) {
            Ok(res) => serde_json::to_string_pretty(&res).unwrap_or_else(|e| format!("JSON Error: {}", e)),
            Err(e) => e,
        }
    }

    #[tool(
        description = "Generate peta layout SNI 6502:2010 compliant. 13 elemen kartografi: judul, skala grafis+numerik, legenda, arah utara, grid koordinat (lat/lon), peta inset Indonesia, CRS (UTM auto), sumber data, tanggal, pembuat, batas administrasi, bingkai peta. Ref: SNI 6502:2010, PermenLH 16/2012."
    )]
    async fn generate_map_sni(&self, Parameters(p): Parameters<MapGenParam>) -> String {
        tools::gis::cartography::generate_map(
            &p.geojson,
            &p.output_path,
            &p.title,
            p.realtime_satellite.unwrap_or(false),
            p.author.as_deref(),
            p.date.as_deref(),
            p.show_admin.unwrap_or(true),
        )
    }

    #[tool(
        description = "VALIDATOR FISIKA EKUATORIAL: Wajib dipanggil oleh AI sebelum mengonfirmasi angka analisis untuk banjir, polusi udara, atau vegetasi (NDVI) guna memastikan tidak ada hukum alam yang dilanggar."
    )]
    async fn physics_check(&self, Parameters(p): Parameters<ValidatorParam>) -> String {
        crate::tools::physics_validator::validate(p)
    }

    // =======================================
    // CALCULATORS — Deterministik, akurat 99%
    // =======================================

    #[tool(
        description = "RUSLE Soil Loss Equation. A = R x K x LS x C x P. Includes Lenvain's empirical formula for Indonesia."
    )]
    fn rusle_erosion(&self, Parameters(p): Parameters<RusleParam>) -> String {
        tools::calculators::rusle::calculate(p.r_input, p.rain_mm_yr, p.k, p.ls, p.c, p.p)
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
        tools::calculators::streeter_phelps::calculate(
            p.k1,
            p.k2,
            p.l0,
            p.d0,
            p.velocity_ms,
            p.distance_km,
            p.temp_c,
        )
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
        tools::calculators::gaussian_plume::calculate(
            p.emission_gs,
            p.wind_ms,
            p.stack_height_m,
            p.distance_m,
            &p.stability_class,
        )
    }

    #[tool(description = "Noise dB Attenuation. Kebisingan vs jarak. Ref: ISO 9613.")]
    fn noise_attenuation(&self, Parameters(p): Parameters<NoiseParam>) -> String {
        tools::calculators::noise_db::attenuation_distance(p.source_db, p.distance_m)
    }

    #[tool(description = "Landfill Gas CH4 Estimator. Emisi metana TPA. Ref: EPA LandGEM.")]
    fn landfill_gas(&self, Parameters(p): Parameters<LandfillParam>) -> String {
        tools::calculators::landfill_gas::calculate(
            p.waste_ton,
            p.years_open,
            p.k_decay,
            p.l0_potential,
        )
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
        tools::compliance::proper::score(
            p.has_izin,
            p.compliance_pct,
            p.beyond_compliance,
            p.community_dev,
            p.circular_economy,
        )
    }

    #[tool(
        description = "IKLH: Indeks Kualitas Lingkungan Hidup = (IKA×30%)+(IKU×30%)+(IKTL×40%). Ref: PermenLHK P.27/2021."
    )]
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

    #[tool(
        description = "Desain IPAL Activated Sludge (Monod kinetics). Ref: Metcalf & Eddy 2003."
    )]
    fn wastewater_design(&self, Parameters(p): Parameters<WastewaterParam>) -> String {
        tools::calculators::wastewater::design(p.q_m3d, p.bod_influent, p.bod_target, p.temp_c)
    }

    // (Peatland Subsidence moved to the advanced_physics implementation below)

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

    #[tool(
        description = "Carlson TSI Eutrophication Index. Hanya valid untuk DANAU. Ref: Carlson 1977."
    )]
    fn eutrophication_tsi(&self, Parameters(p): Parameters<EutrophicationParam>) -> String {
        tools::calculators::eutrophication::calculate(
            p.secchi_depth_m,
            p.chlorophyll_ugl,
            p.total_phosphorus_ugl,
        )
    }

    #[tool(
        description = "Soil Texture Classification (USDA triangle). Input: sand%, silt%, clay%."
    )]
    fn soil_texture(&self, Parameters(p): Parameters<SoilTextureParam>) -> String {
        tools::calculators::soil_quality::classify_texture(p.sand_pct, p.silt_pct, p.clay_pct)
    }

    #[tool(description = "Environmental Flow Tennant Method. ⚠️ Screening awal saja.")]
    fn environmental_flow(&self, Parameters(p): Parameters<EflowParam>) -> String {
        tools::calculators::eflow::calculate(p.maf_m3s)
    }

    #[tool(
        description = "IDF Curve Mononobe. Intensitas hujan dari R24 & durasi. Ref: standar Indonesia."
    )]
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
        tools::calculators::rainwater::calculate(
            p.roof_area_m2,
            p.rainfall_mm,
            p.runoff_coeff,
            p.demand_liters_day,
        )
    }

    #[tool(description = "Ecological Footprint (gha). Jejak ekologis personal.")]
    fn ecological_footprint(&self, Parameters(p): Parameters<FootprintParam>) -> String {
        tools::calculators::ecological_footprint::calculate(
            p.electricity_kwh,
            p.vehicle_km,
            p.meat_kg_week,
            p.waste_kg_day,
        )
    }

    #[tool(
        description = "Simplified LCA. Cradle-to-gate emission. Materials: baja/semen/plastik/aluminium/kayu/kertas/beton/kaca/bata."
    )]
    fn lca_simplified(&self, Parameters(p): Parameters<LcaParam>) -> String {
        tools::calculators::lca::calculate(&p.material, p.mass_kg)
    }

    #[tool(description = "UV Index dari solar zenith, altitude, ozone, cloud. Ref: WHO/WMO.")]
    fn uv_index(&self, Parameters(p): Parameters<UvParam>) -> String {
        tools::calculators::uv_index::calculate(
            p.solar_zenith_deg,
            p.altitude_m,
            p.ozone_du,
            p.cloud_cover_pct,
        )
    }

    #[tool(
        description = "Ocean Acidification: Ω aragonite dari pH, pCO2, suhu, salinitas. Ref: Zeebe 2001."
    )]
    fn ocean_acidification(&self, Parameters(p): Parameters<OceanAcidParam>) -> String {
        tools::calculators::ocean_acidification::calculate(
            p.ph,
            p.pco2_uatm,
            p.temp_c,
            p.salinity_psu,
        )
    }

    #[tool(description = "Land Subsidence Terzaghi 1D Consolidation. Jakarta/Semarang/Pekalongan.")]
    fn land_subsidence(&self, Parameters(p): Parameters<SubsidenceParam>) -> String {
        tools::calculators::land_subsidence::calculate(
            p.clay_thickness_m,
            p.delta_stress_kpa,
            p.cc,
            p.e0,
            p.sigma0_kpa,
        )
    }

    #[tool(
        description = "Time-dependent 1D consolidation settlement (Terzaghi 1943 + Biot α poroelasticity). Computes ultimate settlement Sc, degree of consolidation U(t) via the Terzaghi series, time factor Tv, settlement at time t, and time to U=50%/90%. Handles single/double drainage and Biot coefficient α. Ref: Terzaghi 1943; Biot 1941. Use for Jakarta/Semarang/Pekalongan land subsidence timelines from groundwater extraction."
    )]
    fn land_subsidence_time(&self, Parameters(p): Parameters<ConsolidationParam>) -> String {
        tools::calculators::land_subsidence::calculate_consolidation(&p)
    }

    #[tool(
        description = "Thermal Pollution mixing zone. Suhu campuran sungai + buangan PLTU. Baku mutu: ΔT maks 3°C."
    )]
    fn thermal_pollution(&self, Parameters(p): Parameters<ThermalParam>) -> String {
        tools::calculators::thermal_pollution::calculate(
            p.q_river_m3s,
            p.t_river_c,
            p.q_discharge_m3s,
            p.t_discharge_c,
        )
    }

    #[tool(description = "Sea Level Rise Inundation (bathtub model). Skenario IPCC AR6.")]
    fn sea_level_rise(&self, Parameters(p): Parameters<SlrParam>) -> String {
        tools::calculators::sea_level_rise::calculate(p.elevation_m, p.slr_m, p.storm_surge_m)
    }

    #[tool(description = "Waste to Energy Calculator. Nilai kalori sampah → listrik.")]
    fn waste_to_energy(&self, Parameters(p): Parameters<WteParam>) -> String {
        tools::calculators::waste_to_energy::calculate(
            p.waste_ton_day,
            p.moisture_pct,
            p.organic_pct,
        )
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

    #[tool(
        description = "Generate laporan PDF formal (AMDAL/ESG/Environmental Report). sections: JSON array [[title,body],...]"
    )]
    fn generate_pdf_report(&self, Parameters(p): Parameters<PdfReportParam>) -> String {
        tools::processing::pdf_report::generate(&p.title, &p.sections_json, &p.output_path)
    }

    #[tool(
        description = "GeoTIFF info via GDAL. Metadata citra satelit (CRS, resolusi, band, extent)."
    )]
    fn geotiff_info(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::processing::geotiff::info(&p.query)
    }

    #[tool(description = "Crop/clip GeoTIFF ke bounding box. bbox: 'minlon minlat maxlon maxlat'")]
    fn geotiff_crop(&self, Parameters(p): Parameters<GeotiffCropParam>) -> String {
        tools::processing::geotiff::crop(&p.input_path, &p.output_path, &p.bbox)
    }

    #[tool(
        description = "Watershed/DAS delineation dari DEM (pysheds D8). Input: DEM .tif + pour point (x,y)."
    )]
    fn watershed_delineation(&self, Parameters(p): Parameters<WatershedParam>) -> String {
        tools::processing::watershed::delineate(&p.dem_path, p.pour_x, p.pour_y, &p.output_path)
    }

    #[tool(description = "IDW Spatial Interpolation. Interpolasi data titik ke lokasi target.")]
    fn spatial_interpolation_idw(&self, Parameters(p): Parameters<IdwParam>) -> String {
        let points: Vec<(f64, f64, f64)> =
            p.points.iter().map(|pt| (pt[0], pt[1], pt[2])).collect();
        tools::processing::interpolation::idw(
            &points,
            p.target_x,
            p.target_y,
            p.power.unwrap_or(2.0),
        )
    }

    #[tool(
        description = "3D Terrain Visualization dari DEM GeoTIFF. Render surface 3D dengan color map elevasi."
    )]
    fn terrain_3d(&self, Parameters(p): Parameters<Terrain3dParam>) -> String {
        tools::processing::terrain3d::render(
            &p.dem_path,
            &p.output_path,
            &p.title,
            p.exaggeration.unwrap_or(2.0),
        )
    }

    #[tool(
        description = "4D Terrain Rotation Animation (GIF). Rotasi 360° dari terrain 3D — simulasi perspektif temporal."
    )]
    fn terrain_4d_rotation(&self, Parameters(p): Parameters<Terrain3dParam>) -> String {
        tools::processing::viz4d::terrain_rotation(
            &p.dem_path,
            &p.output_path,
            &p.title,
            p.exaggeration.unwrap_or(2.0),
            36,
        )
    }

    #[tool(
        description = "4D Time Series Animation (GIF). Animasi data lingkungan berkembang seiring waktu. values: comma-separated, labels: comma-separated."
    )]
    fn timeseries_4d(&self, Parameters(p): Parameters<Timeseries4dParam>) -> String {
        tools::processing::viz4d::timeseries_animation(
            &p.values,
            &p.labels.clone().unwrap_or_default(),
            &p.output_path,
            &p.title,
            &p.ylabel.clone().unwrap_or("Value".into()),
        )
    }

    #[tool(
        description = "3D Flood Simulation: terrain + genangan air pada level tertentu. Menghitung area genangan & kedalaman."
    )]
    fn flood_3d(&self, Parameters(p): Parameters<Flood3dParam>) -> String {
        tools::processing::flood_sim::flood_3d(
            &p.dem_path,
            &p.output_path,
            p.water_level_m,
            &p.title,
            p.exaggeration.unwrap_or(2.0),
        )
    }

    #[tool(
        description = "4D Flood Animation (GIF): simulasi kenaikan level air dari start ke end. Temporal flood inundation model."
    )]
    fn flood_4d(&self, Parameters(p): Parameters<Flood4dParam>) -> String {
        tools::processing::flood_sim::flood_4d(
            &p.dem_path,
            &p.output_path,
            p.water_start_m,
            p.water_end_m,
            p.steps.unwrap_or(15),
            &p.title,
            p.exaggeration.unwrap_or(2.0),
        )
    }

    // =======================================
    // AIR QUALITY DISPERSION MODELING
    // =======================================

    #[tool(
        description = "Stability Class (Turner 1970). Estimasi kelas Pasquill-Gifford dari data met. solar_radiation: strong/moderate/slight/night"
    )]
    fn stability_class(&self, Parameters(p): Parameters<StabilityParam>) -> String {
        tools::airquality::stability::estimate(
            p.wind_speed_ms,
            &p.solar_radiation,
            p.cloud_cover_eighths,
        )
    }

    #[tool(
        description = "Monin-Obukhov Similarity Theory. Computes the Obukhov length L, stability parameter ζ=z/L, Businger-Dyer similarity functions φm/φh, and continuous eddy diffusivities Km/Kh from friction velocity u*, surface heat flux, and temperature. Ref: Monin & Obukhov 1954; Dyer 1974. Use for continuous (non-discrete) atmospheric stability and AERMOD-grade eddy diffusivity parameterization."
    )]
    fn monin_obukhov(&self, Parameters(p): Parameters<MoninObukhovParam>) -> String {
        tools::airquality::stability::monin_obukhov(&p)
    }

    #[tool(
        description = "Briggs Plume Rise. Hitung effective stack height. Ref: Briggs (1969-1975), AERMOD."
    )]
    fn plume_rise(&self, Parameters(p): Parameters<PlumeRiseParam>) -> String {
        tools::airquality::plume_rise::calculate(
            p.stack_height_m,
            p.stack_diameter_m,
            p.exit_velocity_ms,
            p.exit_temp_k,
            p.ambient_temp_k,
            p.wind_speed_ms,
        )
    }

    #[tool(
        description = "2D Air Dispersion Contour Map (PNG). Multi-source Gaussian plume grid. sources: JSON [{Q_gs,H_m,x_m,y_m}]"
    )]
    fn dispersion_2d(&self, Parameters(p): Parameters<Dispersion2dParam>) -> String {
        tools::airquality::dispersion::render_2d(
            &p.sources_json,
            p.wind_speed,
            p.wind_dir,
            &p.stability,
            &p.output_path,
            &p.title,
            p.grid_size.unwrap_or(5000),
        )
    }

    #[tool(
        description = "3D Air Dispersion Plume Visualization (PNG). 3D surface plot konsentrasi polutan."
    )]
    fn dispersion_3d(&self, Parameters(p): Parameters<Dispersion2dParam>) -> String {
        tools::airquality::dispersion::render_3d(
            &p.sources_json,
            p.wind_speed,
            p.wind_dir,
            &p.stability,
            &p.output_path,
            &p.title,
            p.grid_size.unwrap_or(5000),
        )
    }

    #[tool(
        description = "4D Air Dispersion Animation (GIF). Simulasi perubahan arah/kecepatan angin temporal. wind_speeds & wind_dirs: comma-separated."
    )]
    fn dispersion_4d(&self, Parameters(p): Parameters<Dispersion4dParam>) -> String {
        tools::airquality::dispersion::render_4d(
            &p.sources_json,
            &p.wind_speeds,
            &p.wind_dirs,
            &p.stability,
            &p.output_path,
            &p.title,
            p.grid_size.unwrap_or(5000),
        )
    }

    // =======================================
    // OCEAN MODELING 2D/3D/4D
    // =======================================

    #[tool(
        description = "3D Bathymetry: Visualisasi relief dasar laut. Input: lat, lon pusat area."
    )]
    fn ocean_bathymetry_3d(&self, Parameters(p): Parameters<OceanBathyParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::ocean_modeling::ocean_viz::bathymetry_3d(p.lat, p.lon, &p.output_path, &p.title)
    }

    #[tool(
        description = "2D Ocean Current: Peta vector field arus laut berbasis angin (Ekman). Input: lat, lon, wind."
    )]
    fn ocean_current_2d(&self, Parameters(p): Parameters<OceanCurrentParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::ocean_modeling::ocean_viz::current_2d(
            p.lat,
            p.lon,
            p.wind_speed,
            p.wind_dir,
            &p.output_path,
            &p.title,
        )
    }

    #[tool(
        description = "3D Thermal Mixing: Visualisasi mixing zone polusi termal PLTU di laut. Baku mutu: DeltaT max 3C."
    )]
    fn ocean_thermal_3d(&self, Parameters(p): Parameters<OceanThermalParam>) -> String {
        tools::ocean_modeling::ocean_viz::thermal_3d(
            p.discharge_temp,
            p.ambient_temp,
            &p.output_path,
            &p.title,
        )
    }

    #[tool(
        description = "4D Marine Pollution: Animasi GIF Lagrangian particle tracking polutan di laut. current_speeds & current_dirs: comma-separated."
    )]
    fn ocean_pollution_4d(&self, Parameters(p): Parameters<OceanPollutionParam>) -> String {
        tools::ocean_modeling::ocean_viz::pollution_4d(
            &p.current_speeds,
            &p.current_dirs,
            &p.output_path,
            &p.title,
        )
    }

    #[tool(
        description = "JONSWAP Wave Height: Hitung Hs dari angin, fetch, dan kedalaman. Ref: Hasselmann 1973."
    )]
    fn wave_jonswap(&self, Parameters(p): Parameters<WaveParam>) -> String {
        tools::ocean_modeling::wave::jonswap(p.wind_speed_ms, p.fetch_m, p.depth_m)
    }

    #[tool(
        description = "Coral Bleaching DHW: Degree Heating Weeks dari data SST mingguan. Ref: NOAA Coral Reef Watch."
    )]
    fn coral_bleaching_dhw(&self, Parameters(p): Parameters<CoralDhwParam>) -> String {
        let sst: Vec<f64> = p
            .sst_weekly
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        tools::ocean_modeling::wave::coral_bleaching_dhw(&sst, p.sst_max_monthly_mean)
    }

    #[tool(
        description = "CERC Sediment Transport: Longshore transport rate. Ref: SPM 1984, USACE."
    )]
    fn sediment_transport_cerc(&self, Parameters(p): Parameters<SedimentParam>) -> String {
        tools::ocean_modeling::sediment::cerc_transport(p.hs_m, p.wave_angle_deg, p.beach_slope_deg)
    }

    #[tool(
        description = "Oil Spill Trajectory & Fate: Drift (3% wind + current) + evaporasi + spreading. oil_type: crude/diesel/gasoline/bunker."
    )]
    fn oil_spill_model(&self, Parameters(p): Parameters<OilSpillParam>) -> String {
        tools::ocean_modeling::oil_spill::simulate_4d(
            p.volume_m3,
            &p.oil_type,
            p.wind_speed,
            p.wind_dir,
            p.current_speed,
            p.current_dir,
            p.hours,
            &p.output_path,
        )
    }

    // =======================================
    // ADVANCED PHYSICS (FRONTIER 2026)
    // =======================================

    #[tool(
        description = "Flux Divergence Emission: Deteksi emisi gas misterius dari citra Satelit via Central Difference (Beirle et al., 2019)."
    )]
    fn satellite_flux_divergence(&self, Parameters(p): Parameters<FluxDivergenceParam>) -> String {
        tools::advanced_physics::flux_divergence::calculate_emissions(
            &p.grid_data_json,
            p.u_wind,
            p.v_wind,
            p.dx_meters,
            p.dy_meters,
            p.lifetime_hours,
        )
    }

    #[tool(
        description = "Groundwater Advection-Diffusion: Solusi eksplisit Finite Difference dengan jaminan stabilitas CFL."
    )]
    fn groundwater_advection_diffusion(
        &self,
        Parameters(p): Parameters<GroundwaterPdeParam>,
    ) -> String {
        tools::advanced_physics::groundwater_pde::solve_pde(
            &p.h_initial_json,
            p.diffusivity_d,
            p.dx_meters,
            p.dy_meters,
            p.time_steps,
            p.dt_seconds,
        )
    }

    #[tool(
        description = "Richards Equation 1D infiltration solver (van Genuchten–Mualem). Solves the mixed-form Richards PDE with the modified-Picard scheme (Celia et al. 1990) for variably-saturated flow in the unsaturated zone. Input: θr, θs, α(1/m), n, Ks(m/s), column depth, pressure heads. Returns water-content profile, wetting-front depth, mass balance, and top flux. Ref: van Genuchten 1980; Mualem 1976. Use for infiltration, recharge, slope-stability antecedent moisture, and peatland TMAT."
    )]
    fn richards_infiltration(&self, Parameters(p): Parameters<RichardsParam>) -> String {
        tools::advanced_physics::groundwater_pde::solve_richards_1d(&p)
    }

    #[tool(
        description = "Bayesian Sensor Assimilation: Particle Filter Systematic Resampling untuk membersihkan noise sensor IoT lapangan."
    )]
    fn bayesian_sensor_assimilation(
        &self,
        Parameters(p): Parameters<BayesianSensorParam>,
    ) -> String {
        tools::advanced_physics::bayesian_assimilation::assimilate_sensor_data(
            &p.prior_particles_json,
            p.sensor_reading,
            p.sensor_noise_std,
        )
    }

    #[tool(
        description = "UHI Radiative Transfer: Hitung lonjakan suhu mikro perkotaan akibat geometri gedung (Sky View Factor) & Albedo."
    )]
    fn uhi_radiative_transfer(&self, Parameters(p): Parameters<UhiParam>) -> String {
        tools::advanced_physics::uhi_radiative::calculate_uhi(
            p.albedo_urban,
            p.sky_view_factor,
            p.solar_insolation_w,
            p.ambient_temp_c,
        )
    }

    // =====================================================
    // GOD TIER: 13 PREVIOUSLY UNREGISTERED TOOLS
    // =====================================================

    #[tool(
        description = "Biodiversity Index: Shannon-Wiener H' & Simpson 1-D. Ref: Shannon 1949. Input: JSON array jumlah individu per spesies."
    )]
    fn biodiversity_index(&self, Parameters(p): Parameters<BiodiversityCalcParam>) -> String {
        let counts: Result<Vec<u64>, _> = serde_json::from_str(&p.species_counts_json);
        match counts {
            Ok(c) => tools::calculators::biodiversity::calculate(&c),
            Err(e) => format!("ERROR [E103]: JSON parsing: {}", e),
        }
    }

    #[tool(
        description = "Composting C/N Ratio Optimizer. Ref: USDA/SNI. Input: JSON array [[name, mass_kg, c_pct, n_pct], ...]"
    )]
    fn composting_cn(&self, Parameters(p): Parameters<CompostingParam>) -> String {
        let mats: Result<Vec<(String, f64, f64, f64)>, _> = serde_json::from_str(&p.materials_json);
        match mats {
            Ok(m) => tools::calculators::composting::calculate(&m),
            Err(e) => format!("ERROR [E103]: JSON parsing: {}", e),
        }
    }

    #[tool(
        description = "Flood Frequency Gumbel Distribution. Min 10 tahun data. Ref: Chow 1951, USGS Bulletin 17C."
    )]
    fn flood_frequency_gumbel(&self, Parameters(p): Parameters<FloodFreqParam>) -> String {
        let data: Result<Vec<f64>, _> = serde_json::from_str(&p.data_json);
        match data {
            Ok(d) => tools::calculators::flood_frequency::gumbel(&d, p.return_period),
            Err(e) => format!("ERROR [E103]: JSON parsing: {}", e),
        }
    }

    #[tool(
        description = "Log-Pearson Type III Flood Frequency. Ref: USGS Bulletin 17C, SNI 2415:2016. Wilson-Hilferty KT approximation."
    )]
    fn log_pearson_iii(&self, Parameters(p): Parameters<FloodFreqParam>) -> String {
        let data: Result<Vec<f64>, _> = serde_json::from_str(&p.data_json);
        match data {
            Ok(d) => tools::calculators::flood_frequency::log_pearson_iii(&d, p.return_period),
            Err(e) => format!("ERROR [E103]: JSON parsing: {}", e),
        }
    }

    #[tool(
        description = "Acid Mine Drainage (AMD/ABA). Ref: PermenLH 113/2003. Klasifikasi: PAF/NAF/Uncertain."
    )]
    fn acid_mine_drainage(&self, Parameters(p): Parameters<AmdCalcParam>) -> String {
        tools::calculators::acid_mine_drainage::calculate(p.sulfur_pct, p.anc_kg_h2so4_t, p.nag_ph)
    }

    #[tool(
        description = "Transport Emission IPCC Volume BBM. Ref: IPCC 2006. Input: tipe BBM + liter."
    )]
    fn transport_emission(&self, Parameters(p): Parameters<TransportEmParam>) -> String {
        tools::calculators::transport_emission::calculate(&p.fuel_type, p.liters)
    }

    #[tool(
        description = "Indeks Pencemaran (IP) Air. Ref: KepmenLH 115/2003. Normalisasi log untuk ratio >1."
    )]
    fn indeks_pencemaran(&self, Parameters(p): Parameters<IpParam>) -> String {
        tools::compliance::indeks_pencemaran::calculate(&p.data_json, p.temp_c)
    }

    #[tool(
        description = "Metode STORET Kualitas Air. Ref: KepmenLH 115/2003. Skor negatif: Kelas A-D."
    )]
    fn storet_water(&self, Parameters(p): Parameters<StoretParam>) -> String {
        tools::compliance::storet::calculate(&p.data_json)
    }

    #[tool(
        description = "SPPL Checker. Ref: PP 22/2021. Cek apakah kegiatan cukup SPPL atau wajib UKL-UPL/AMDAL."
    )]
    fn sppl_checker(&self, Parameters(p): Parameters<SpplParam>) -> String {
        tools::compliance::sppl::check(&p.kegiatan, p.is_wajib_amdal, p.is_wajib_uklupl)
    }

    #[tool(
        description = "Baku Mutu Air Laut (30+ parameter). Ref: KepMen LH 51/2004. pH/DO/BOD/logam berat/nutrient/coliform. Peruntukan: wisata/biota/pelabuhan."
    )]
    fn baku_mutu_laut(&self, Parameters(p): Parameters<BakuMutuLautParam>) -> String {
        tools::compliance::baku_mutu_laut::check(&p.parameter, p.concentration, &p.peruntukan)
    }

    #[tool(
        description = "WAQI Ground Sensor Air Quality. Source: waqi.info. Data stasiun fisik PM2.5/NO2/SO2/O3/CO."
    )]
    async fn waqi_air_quality(&self, Parameters(p): Parameters<LatLonParam>) -> String {
        let lat = p.lat.unwrap_or(-6.2);
        let lon = p.lon.unwrap_or(106.85);
        if let Err(e) = crate::indonesia::validate_coords(lat, lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::data::waqi::get_air_quality(&HTTP, lat, lon).await
    }

    #[tool(
        description = "4D Satellite Timelapse GIF via GEE. Cloud-free compositing tahunan Sentinel-2/Sentinel-1."
    )]
    fn satellite_timelapse(&self, Parameters(p): Parameters<TimelapseParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::satellite::timelapse::generate_4d_timelapse(
            p.lat,
            p.lon,
            p.buffer_km,
            p.start_year,
            p.end_year,
            &p.sensor,
            &p.output_path,
            "monthly",
            10,
            None,
            None,
        )
    }

    #[tool(
        description = "NASA EMIT Hyperspectral 285-band. Ekstraksi spectral signature mineral via GEE. Output: PNG + data."
    )]
    fn satellite_hyperspectral(&self, Parameters(p): Parameters<HyperspectralParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::satellite::hyperspectral::extract_signature(p.lat, p.lon, &p.output_path)
    }

    #[tool(
        description = "Advanced Physics Validator V3: TROPOMI UQ, PBL Inversion, Bingham Rheology, Gas Kinetics Tropis."
    )]
    fn physics_validator_v3(&self, Parameters(p): Parameters<ValidatorV3Param>) -> String {
        tools::advanced_physics::validator_v3::validate_advanced_physics(
            &p.gas_type,
            p.concentration,
            &p.time_of_day,
            &p.fluid_type,
            p.slope_angle_deg,
            p.depth_m,
        )
    }

    // =====================================================
    // GOD TIER: 15 NEW COMPLIANCE/REGULATION TOOLS
    // =====================================================

    #[tool(
        description = "Baku Mutu Udara Ambien. Ref: PP 41/1999. Cek konsentrasi polutan vs standar nasional."
    )]
    fn baku_mutu_udara(&self, Parameters(p): Parameters<BakuMutuUdaraParam>) -> String {
        tools::compliance::baku_mutu_udara::check(&p.parameter, p.concentration, &p.averaging_time)
    }

    #[tool(
        description = "Baku Mutu Emisi Sumber Tidak Bergerak. Ref: PermenLHK 15/2019. Per jenis industri."
    )]
    fn baku_mutu_emisi(&self, Parameters(p): Parameters<BakuMutuEmisiParam>) -> String {
        tools::compliance::baku_mutu_emisi::check(&p.industry, &p.parameter, p.concentration)
    }

    #[tool(
        description = "Baku Mutu Air Limbah Industri. Ref: PermenLH 5/2014. 15+ jenis industri."
    )]
    fn baku_mutu_air_limbah(&self, Parameters(p): Parameters<BakuMutuAirLimbahParam>) -> String {
        tools::compliance::baku_mutu_air_limbah::check(&p.industry, &p.parameter, p.concentration)
    }

    #[tool(
        description = "Baku Mutu Air Limbah Domestik. Ref: PermenLHK 11/2025. pH/BOD/COD/TSS/oil/ammonia/coliform/detergen."
    )]
    fn baku_mutu_domestik(&self, Parameters(p): Parameters<BakuMutuDomestikParam>) -> String {
        tools::compliance::baku_mutu_domestik::check(&p.parameter, p.concentration)
    }

    #[tool(
        description = "Baku Mutu Kebisingan. Ref: KepmenLH 48/1996. 10 zona: perumahan/industri/RS/sekolah/ibadah."
    )]
    fn baku_mutu_kebisingan(&self, Parameters(p): Parameters<BakuMutuKebisinganParam>) -> String {
        tools::compliance::baku_mutu_kebisingan::check(&p.zone, p.measured_db)
    }

    #[tool(
        description = "Baku Mutu Getaran Mekanik. Ref: KepmenLH 49/1996. Zona: pemukiman/kantor/industri/RS."
    )]
    fn baku_mutu_getaran(&self, Parameters(p): Parameters<BakuMutuGetaranParam>) -> String {
        tools::compliance::baku_mutu_getaran::check(&p.zone, p.vibration_mm_s)
    }

    #[tool(
        description = "Baku Mutu Kebauan. Ref: KepmenLH 50/1996. H2S/NH3/methyl mercaptan/styrene."
    )]
    fn baku_mutu_kebauan(&self, Parameters(p): Parameters<BakuMutuKebauanParam>) -> String {
        tools::compliance::baku_mutu_kebauan::check(&p.chemical, p.concentration_ppm)
    }

    #[tool(
        description = "ISPU Calculator (Indeks Standar Pencemar Udara). Ref: PermenLHK P.14/2020. Breakpoint interpolation."
    )]
    fn ispu_calculator(&self, Parameters(p): Parameters<IspuParam>) -> String {
        tools::compliance::ispu::calculate(p.pm10, p.pm25, p.so2, p.co, p.o3, p.no2)
    }

    #[tool(
        description = "Kelas Risiko Lingkungan (OSS). Ref: PP 22/2023. Tentukan: AMDAL/UKL-UPL/SPPL."
    )]
    fn risk_class_oss(&self, Parameters(p): Parameters<RiskClassParam>) -> String {
        tools::compliance::risk_class::determine(
            &p.sector,
            &p.scale_description,
            p.has_hazardous_waste,
            p.near_protected_area,
        )
    }

    #[tool(
        description = "Daya Dukung Lingkungan Hidup. Ref: PermenLH 17/2009. Pendekatan: populasi/air/pangan."
    )]
    fn daya_dukung(&self, Parameters(p): Parameters<DayaDukungParam>) -> String {
        tools::compliance::daya_dukung::calculate(
            &p.approach,
            p.area_ha,
            p.population,
            p.water_supply_m3_yr,
            p.water_demand_m3_yr,
            p.food_production_ton_yr,
            p.food_demand_ton_yr,
        )
    }

    #[tool(description = "Daya Tampung Beban Pencemaran. Ref: PP 22/2021. Mass balance sungai.")]
    fn daya_tampung(&self, Parameters(p): Parameters<DayaTampungParam>) -> String {
        tools::compliance::daya_tampung::calculate(
            p.q_river_m3s,
            p.c_upstream_mgl,
            p.c_standard_mgl,
            p.q_waste_m3s,
            p.c_waste_mgl,
            &p.parameter,
        )
    }

    #[tool(
        description = "GHG Inventory. Ref: PermenLHK 102/2018, IPCC Tier 1. Sektor: energy/ippu/afolu/waste."
    )]
    fn ghg_inventory(&self, Parameters(p): Parameters<GhgInventoryParam>) -> String {
        tools::compliance::ghg_inventory::calculate(&p.sector, &p.activity, p.amount)
    }

    #[tool(description = "IKLH Sub-Indices: IKA/IKU/IKTL/IKAL. Ref: PermenLHK P.27/2021.")]
    fn iklh_sub_indices(&self, Parameters(p): Parameters<IklhSubParam>) -> String {
        match p.sub_type.to_lowercase().as_str() {
            "ika" => {
                let vals: Result<Vec<f64>, _> = serde_json::from_str(&p.data_json);
                match vals {
                    Ok(v) => tools::compliance::iklh_sub::calculate_ika(&v),
                    Err(e) => format!("ERROR: {}", e),
                }
            }
            "iku" => {
                let vals: Result<Vec<f64>, _> = serde_json::from_str(&p.data_json);
                match vals {
                    Ok(v) => tools::compliance::iklh_sub::calculate_iku(&v),
                    Err(e) => format!("ERROR: {}", e),
                }
            }
            "iktl" => {
                let v: Result<serde_json::Value, _> = serde_json::from_str(&p.data_json);
                match v {
                    Ok(val) => {
                        let fc = val["forest_cover_pct"].as_f64().unwrap_or(0.0);
                        let tp = val["target_pct"].as_f64().unwrap_or(30.0);
                        tools::compliance::iklh_sub::calculate_iktl(fc, tp)
}


                    Err(e) => format!("ERROR: {}", e),
                }
            }
            "ikal" => tools::compliance::iklh_sub::calculate_ikal(&p.data_json),
            _ => "ERROR: sub_type harus ika/iku/iktl/ikal".into(),
        }
    }

    #[tool(
        description = "Regulasi Lingkungan Indonesia Lookup. Cari regulasi berdasarkan topik: air/udara/limbah/b3/amdal/emisi/laut/hutan/karbon."
    )]
    fn regulasi_lookup(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::compliance::regulasi::lookup(&p.query)
    }

    #[tool(
        description = "AMDAL Screening. Ref: PermenLHK 4/2021. Tentukan wajib AMDAL/UKL-UPL/SPPL berdasarkan jenis & skala kegiatan."
    )]
    fn amdal_screening(&self, Parameters(p): Parameters<AmdalScreeningParam>) -> String {
        tools::compliance::amdal_screening::screen(
            &p.sector,
            &p.activity,
            p.scale_value,
            &p.scale_unit,
        )
    }

    // =====================================================
    // GOD TIER: 5 AMDAL DOCUMENT GENERATOR
    // =====================================================

    #[tool(
        description = "Generate KA-ANDAL PDF. Ref: PermenLHK 5/2021. Kerangka Acuan AMDAL lengkap."
    )]
    fn amdal_ka_andal(&self, Parameters(p): Parameters<KaAndalParam>) -> String {
        tools::amdal::generator::generate_ka_andal(
            &p.project_name,
            &p.location,
            &p.project_type,
            &p.rona_json,
            &p.output_path,
        )
    }

    #[tool(
        description = "Generate ANDAL PDF. Ref: PermenLHK 5/2021. Analisis Dampak Lingkungan Hidup."
    )]
    fn amdal_andal(&self, Parameters(p): Parameters<AndalParam>) -> String {
        tools::amdal::generator::generate_andal(
            &p.project_name,
            &p.location,
            &p.impacts_json,
            &p.output_path,
        )
    }

    #[tool(
        description = "Generate RKL-RPL PDF. Ref: PermenLHK 5/2021. Rencana Pengelolaan & Pemantauan Lingkungan."
    )]
    fn amdal_rkl_rpl(&self, Parameters(p): Parameters<RklRplParam>) -> String {
        tools::amdal::generator::generate_rkl_rpl(
            &p.project_name,
            &p.location,
            &p.management_json,
            &p.output_path,
        )
    }

    #[tool(
        description = "Generate UKL-UPL PDF. Ref: PermenLHK 6/2021. Untuk kegiatan non-AMDAL risiko menengah."
    )]
    fn ukl_upl_generator(&self, Parameters(p): Parameters<UklUplParam>) -> String {
        tools::amdal::generator::generate_ukl_upl(
            &p.project_name,
            &p.location,
            &p.impacts_json,
            &p.output_path,
        )
    }

    #[tool(
        description = "KLHS Assessment PDF. Ref: UU 32/2009 Pasal 15-18. Kajian Lingkungan Hidup Strategis."
    )]
    fn klhs_assessment(&self, Parameters(p): Parameters<KlhsParam>) -> String {
        tools::amdal::generator::klhs_assessment(
            &p.policy_name,
            &p.daya_dukung_json,
            &p.output_path,
        )
    }

    // =====================================================
    // GOD TIER: 3 NOISE MODELING TOOLS
    // =====================================================

    #[tool(
        description = "2D Noise Propagation Contour Map. ISO 9613-2 + barrier. Output PNG. Multi-source superposition."
    )]
    fn noise_propagation_2d(&self, Parameters(p): Parameters<Noise2dParam>) -> String {
        tools::noise::propagation::render_2d(
            &p.sources_json,
            &p.output_path,
            &p.title,
            p.grid_size.unwrap_or(500),
            &p.barrier_json.unwrap_or_else(|| "[]".into()),
        )
    }

    #[tool(description = "3D Noise Surface Visualization. ISO 9613-2. Output PNG.")]
    fn noise_propagation_3d(&self, Parameters(p): Parameters<Noise3dParam>) -> String {
        tools::noise::propagation::render_3d(
            &p.sources_json,
            &p.output_path,
            &p.title,
            p.grid_size.unwrap_or(500),
        )
    }

    #[tool(
        description = "Noise Compliance Check. Ref: KepmenLH 48/1996 + ISO 9613. Hitung buffer jarak aman."
    )]
    fn noise_compliance(&self, Parameters(p): Parameters<NoiseComplianceParam>) -> String {
        tools::noise::compliance::check(&p.zone, p.measured_db, p.distance_m, p.source_db)
    }

    // =====================================================
    // GOD TIER: 5 BIODIVERSITY & SOCIAL TOOLS
    // =====================================================

    #[tool(
        description = "IUCN Species Check di area. 33+ spesies dilindungi Indonesia. Filter by provinsi/pulau."
    )]
    async fn iucn_species_check(&self, Parameters(p): Parameters<IucnCheckParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::biodiversity::iucn::check_species(&HTTP, p.lat, p.lon, p.radius_km).await
    }

    #[tool(
        description = "Cek Status Spesies Dilindungi Indonesia. Ref: PP 7/1999, PermenLHK P.106/2018."
    )]
    fn protected_species(&self, Parameters(p): Parameters<ProtectedSpeciesParam>) -> String {
        tools::biodiversity::protected::check(&p.species_name)
    }

    #[tool(description = "Daftar Spesies Dilindungi per Provinsi. Ref: PP 7/1999.")]
    fn protected_species_by_province(
        &self,
        Parameters(p): Parameters<ProtectedByProvinceParam>,
    ) -> String {
        tools::biodiversity::protected::list_by_province(&p.province)
    }

    #[tool(
        description = "Social Impact Assessment Matrix untuk AMDAL. Ref: PermenLH 17/2012. Komponen: ekonomi/sosial/kesehatan."
    )]
    fn social_impact_matrix(&self, Parameters(p): Parameters<SocialImpactParam>) -> String {
        tools::biodiversity::social::impact_matrix(&p.impacts_json)
    }

    #[tool(
        description = "Health Impact Assessment. Analisis paparan polutan → Hazard Quotient (HQ) → risiko kesehatan."
    )]
    fn health_impact(&self, Parameters(p): Parameters<HealthImpactParam>) -> String {
        tools::biodiversity::social::health_impact(
            p.population,
            &p.pollutant,
            p.concentration,
            p.exposure_hours,
        )
    }

    #[tool(
        description = "Valuasi Ekonomi Lingkungan. Ref: PP 46/2017. Metode: replacement_cost/travel_cost/hedonic/damage_cost."
    )]
    fn environmental_valuation(&self, Parameters(p): Parameters<ValuationParam>) -> String {
        tools::biodiversity::valuation::calculate(&p.method, &p.params_json)
    }

    // =====================================================
    // GOD TIER: 5 NEW DATA SOURCES
    // =====================================================

    #[tool(
        description = "ISPU Real-time dari KLHK. Source: ispu.menlhk.go.id. Data kualitas udara stasiun nasional."
    )]
    async fn ispu_klhk(&self, Parameters(p): Parameters<IspuKlhkParam>) -> String {
        tools::datasources::ispu_klhk::get_ispu(&HTTP, &p.kota).await
    }

    #[tool(
        description = "SiPongi KLHK Fire Hotspots. Hotspot kebakaran hutan/lahan per provinsi. Suplemen FIRMS."
    )]
    async fn sipongi_fire(&self, Parameters(p): Parameters<SipongiParam>) -> String {
        tools::datasources::sipongi::get_hotspots(&HTTP, &p.province).await
    }

    #[tool(
        description = "BMKG Historical Climate Data. Data iklim historis: curah hujan, suhu, kelembaban, angin."
    )]
    async fn bmkg_opendata(&self, Parameters(p): Parameters<BmkgOpenParam>) -> String {
        tools::datasources::bmkg_opendata::get_climate_data(&HTTP, &p.station_id, &p.parameter)
            .await
    }

    #[tool(
        description = "OpenStreetMap POI Query. Cari RS/sekolah/permukiman/sungai di sekitar lokasi proyek (wajib AMDAL). Overpass API."
    )]
    async fn osm_poi_query(&self, Parameters(p): Parameters<OsmPoiParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::datasources::osm_poi::query_poi(&HTTP, p.lat, p.lon, p.radius_m, &p.poi_type).await
    }

    #[tool(
        description = "Elevation Profile antara 2 titik. Cross-section topografi. Source: Open-Elevation API / SRTM."
    )]
    async fn elevation_profile(&self, Parameters(p): Parameters<ElevationParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat1, p.lon1) {
            return format!("ERROR [E101]: titik awal — {}", e);
        }
        if let Err(e) = crate::indonesia::validate_coords(p.lat2, p.lon2) {
            return format!("ERROR [E101]: titik akhir — {}", e);
        }
        tools::datasources::elevation::profile(
            &HTTP,
            p.lat1,
            p.lon1,
            p.lat2,
            p.lon2,
            p.num_points.unwrap_or(20),
        )
        .await
    }

    // =====================================================
    // GOD TIER: 6 SAR / SATELLITE TOOLS
    // =====================================================

    #[tool(
        description = "SAR Flood Detection. Sentinel-1 VV change detection pre/post banjir via GEE. Output: flood map PNG."
    )]
    fn sar_flood_detection(&self, Parameters(p): Parameters<SarFloodParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::satellite::sar::flood_detection(
            p.lat,
            p.lon,
            p.buffer_km,
            &p.pre_date,
            &p.post_date,
            &p.output_path,
        )
    }

    #[tool(
        description = "SAR Deforestation Detection. Sentinel-1 backscatter loss detection di bawah awan. Via GEE."
    )]
    fn sar_deforestation(&self, Parameters(p): Parameters<SarDeforestParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::satellite::sar::deforestation(
            p.lat,
            p.lon,
            p.buffer_km,
            &p.start_date,
            &p.end_date,
            &p.output_path,
        )
    }

    #[tool(
        description = "SAR Local Analysis. Proses Sentinel-1 lokal (SNAP GPT). ⚠️ Konfirmasi ukuran file sebelum download."
    )]
    fn sar_local_analysis(&self, Parameters(p): Parameters<SarLocalParam>) -> String {
        tools::satellite::sar::local_analysis(&p.input_path, &p.output_path, &p.analysis_type)
    }

    #[tool(
        description = "InSAR Land Subsidence (Screening). Sentinel-1 via GEE. ⚠️ Screening-level only, bukan full InSAR."
    )]
    fn land_subsidence_insar(&self, Parameters(p): Parameters<SarSubsidenceParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::satellite::sar::subsidence_insar(
            p.lat,
            p.lon,
            p.buffer_km,
            &p.start_date,
            &p.end_date,
            &p.output_path,
        )
    }

    #[tool(
        description = "Burned Area Mapping (dNBR). Sentinel-2 Normalized Burn Ratio. Severity: Unburned→High. Ref: USGS."
    )]
    fn burned_area_mapping(&self, Parameters(p): Parameters<BurnedAreaParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::satellite::burned_area::map_burned_area(
            p.lat,
            p.lon,
            p.buffer_km,
            &p.fire_date,
            &p.output_path,
        )
    }

    #[tool(
        description = "Mangrove Extent Mapping. Sentinel-2 NDVI+NDWI+elevation filter. Bandingkan dengan Global Mangrove Watch."
    )]
    fn mangrove_extent(&self, Parameters(p): Parameters<MangroveExtentParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::satellite::mangrove::map_extent(p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    // =====================================================
    // GOD TIER PHASE 2: WATER & WASTEWATER ENGINEERING
    // =====================================================

    #[tool(
        description = "CT Disinfection Calculator. Ref: EPA Guidance Manual. chlorine/ozone/uv/chloramine."
    )]
    fn ct_disinfection(&self, Parameters(p): Parameters<CtDisinfectionParam>) -> String {
        let disinfectant_str = match p.disinfectant {
            DisinfectantType::Chlorine => "chlorine",
            DisinfectantType::Ozone => "ozone",
            DisinfectantType::Uv => "uv",
            DisinfectantType::Chloramine => "chloramine",
        };
        tools::water::ct_disinfection::calculate(
            disinfectant_str,
            p.concentration_mgl,
            p.contact_time_min,
            &p.target_pathogen,
        )
    }

    #[tool(
        description = "Darcy's Law: q = K×i. Aliran air tanah, kecepatan rembesan, waktu transport kontaminan. Ref: Darcy 1856."
    )]
    fn darcy_flow(&self, Parameters(p): Parameters<DarcyParam>) -> String {
        tools::water::darcy_flow::calculate(p.k_ms, p.gradient, p.area_m2, p.porosity, p.distance_m)
    }

    #[tool(
        description = "Theis Well Drawdown: s = Q/(4πT) × W(u). Prediksi penurunan muka airtanah akibat pemompaan. Ref: Theis 1935."
    )]
    fn theis_drawdown(&self, Parameters(p): Parameters<TheisParam>) -> String {
        tools::water::theis_drawdown::calculate(
            p.q_m3s,
            p.transmissivity_m2s,
            p.storativity,
            p.r_m,
            p.t_s,
        )
    }

    #[tool(
        description = "Hazen-Williams Head Loss. Desain perpipaan air/limbah. Ref: Hazen-Williams."
    )]
    fn hazen_williams(&self, Parameters(p): Parameters<HazenWilliamsParam>) -> String {
        tools::water::hazen_williams::calculate(
            p.q_m3s,
            p.length_m,
            p.diameter_m,
            p.c_coeff,
            p.include_minor_losses,
        )
    }

    #[tool(
        description = "Pump Sizing: TDH, motor power (kW/HP), NPSH. Seleksi pompa untuk proyek air/limbah."
    )]
    fn pump_sizing(&self, Parameters(p): Parameters<PumpSizingParam>) -> String {
        tools::water::pump_sizing::calculate(
            p.q_m3s,
            p.static_lift_m,
            p.friction_loss_m,
            p.velocity_head_m,
            p.pressure_head_m,
            p.efficiency,
        )
    }

    #[tool(
        description = "Sedimentation Tank Design. Overflow rate, detention time, weir loading. Ref: Metcalf & Eddy."
    )]
    fn sedimentation_design(&self, Parameters(p): Parameters<SedimentationParam>) -> String {
        tools::water::sedimentation::design(p.q_m3d, &p.tank_type, &p.tank_shape)
    }

    #[tool(
        description = "UASB Reactor Design. OLR, HRT, gas yield. Untuk IPAL sawit (POME)/tapioka/karet/domestik. Ref: Lettinga 1980."
    )]
    fn uasb_design(&self, Parameters(p): Parameters<UasbParam>) -> String {
        tools::water::uasb_design::design(
            p.q_m3d,
            p.cod_in_mgl,
            p.cod_eff_target,
            p.temperature_c,
            &p.waste_type,
        )
    }

    #[tool(
        description = "Trickling Filter Design (NRC equation). BOD removal efficiency. Ref: NRC 1946."
    )]
    fn trickling_filter(&self, Parameters(p): Parameters<TricklingFilterParam>) -> String {
        tools::water::trickling_filter::design(
            p.q_m3d,
            p.bod_in,
            p.bod_target,
            p.media_depth_m,
            p.recirculation_ratio,
        )
    }

    #[tool(
        description = "Constructed Wetland Design (volumetric first-order model). FWS/HSSF sizing. BOD/TSS/NH4N. Ref: Reed et al. 1995; Kadlec & Knight 1996."
    )]
    fn constructed_wetland(&self, Parameters(p): Parameters<ConstructedWetlandParam>) -> String {
        tools::water::constructed_wetland::design(
            p.q_m3d,
            &p.parameter,
            p.ci_mgl,
            p.ce_target,
            p.temp_c,
            &p.wetland_type,
        )
    }

    #[tool(
        description = "Anaerobic Digestion / Biogas Reactor. SRT, OLR, gas yield. Substrat: sapi/babi/ayam/POME. Ref: McCarty."
    )]
    fn anaerobic_digestion(&self, Parameters(p): Parameters<AnaerobicDigestionParam>) -> String {
        tools::water::anaerobic_digestion::design(
            p.q_m3d,
            p.vs_concentration_kgm3,
            p.vs_destruction_pct,
            p.temperature_c,
            &p.substrate,
        )
    }

    // =====================================================
    // GOD TIER PHASE 2: ENVIRONMENTAL CHEMISTRY
    // =====================================================

    #[tool(
        description = "First-Order Decay Kinetics: C(t) = C₀×e^(-kt). Half-life, t90, t99. Fondasi BOD/degradasi kontaminan."
    )]
    fn first_order_kinetics(&self, Parameters(p): Parameters<FirstOrderParam>) -> String {
        tools::calculators::first_order_kinetics::calculate(p.c0, p.k, p.t, &p.time_unit)
    }

    #[tool(
        description = "Freundlich/Langmuir Isotherm. Desain adsorber karbon aktif. Ref: Freundlich 1906, Langmuir 1918."
    )]
    fn isotherm_calc(&self, Parameters(p): Parameters<IsothermParam>) -> String {
        tools::calculators::isotherm::calculate(
            &p.model, p.ce, p.kf, p.n_exp, p.qmax, p.kl, p.volume_l, p.c0,
        )
    }

    #[tool(
        description = "Henry's Law: p = KH×C. Gas-liquid partitioning. Air stripping feasibility. Common VOCs."
    )]
    fn henrys_law(&self, Parameters(p): Parameters<HenrysLawParam>) -> String {
        tools::calculators::henrys_law::calculate(&p.compound, p.concentration_mgl, p.temperature_c)
    }

    #[tool(
        description = "Nernst Equation: E = E° - (RT/nF)×ln(Q). Potensial redoks, spontanitas reaksi. Ref: Nernst."
    )]
    fn nernst_redox(&self, Parameters(p): Parameters<NernstParam>) -> String {
        tools::calculators::nernst_redox::calculate(
            &p.half_reaction,
            p.temperature_c,
            p.log_q,
            p.n_electrons,
        )
    }

    #[tool(
        description = "Partition Coefficient Kd/Koc. Retardation factor kontaminan di tanah. Mobilitas polutan. Ref: Karickhoff 1981."
    )]
    fn partition_coefficient(&self, Parameters(p): Parameters<PartitionParam>) -> String {
        tools::calculators::partition_coeff::calculate(
            &p.compound,
            p.foc,
            p.bulk_density_kgm3,
            p.porosity,
        )
    }

    // =====================================================
    // GOD TIER PHASE 2: HYDROLOGY ENHANCEMENT
    // =====================================================

    #[tool(
        description = "Rational Method: Q = C×I×A/360. Debit puncak drainase. Ref: Kuichling 1889."
    )]
    fn rational_method(&self, Parameters(p): Parameters<RationalParam>) -> String {
        tools::calculators::rational_method::calculate(p.c_coeff, p.i_mm_hr, p.a_ha, &p.land_use)
    }

    #[tool(description = "SCS Triangular Unit Hydrograph. tp, Qp, tb. Ref: SCS 1972.")]
    fn unit_hydrograph(&self, Parameters(p): Parameters<UnitHydrographParam>) -> String {
        tools::calculators::unit_hydrograph::calculate(p.a_km2, p.tc_hours, p.d_hours)
    }

    #[tool(
        description = "Muskingum Flood Routing. Atenuasi debit puncak di sungai. Ref: McCarthy 1938."
    )]
    fn muskingum_routing(&self, Parameters(p): Parameters<MuskingumParam>) -> String {
        let inflow: Result<Vec<(f64, f64)>, _> = serde_json::from_str(&p.inflow_json);
        match inflow {
            Ok(i) => tools::calculators::muskingum_routing::route(&i, p.k_hours, p.x, p.dt_hours),
            Err(e) => format!("ERROR [E103]: JSON parsing: {}", e),
        }
    }

    #[tool(
        description = "Time of Concentration: Kirpich/Bransby-Williams/SCS Lag. Input untuk kurva IDF. Ref: Kirpich 1940."
    )]
    fn time_of_concentration(&self, Parameters(p): Parameters<TocParam>) -> String {
        tools::calculators::time_of_concentration::calculate(
            &p.method, p.l_m, p.s_slope, p.a_km2, p.cn,
        )
    }

    // =====================================================
    // GOD TIER PHASE 2: SOLID & HAZARDOUS WASTE
    // =====================================================

    #[tool(
        description = "Landfill Liner Design. Giroud-Bonaparte leakage. Ref: PermenPU 3/2013, EPA."
    )]
    fn landfill_liner(&self, Parameters(p): Parameters<LandfillLinerParam>) -> String {
        tools::waste::landfill_liner::design(
            &p.liner_type,
            p.area_m2,
            p.head_on_liner_m,
            p.k_clay,
            p.clay_thickness_m,
        )
    }

    #[tool(
        description = "Leachate Generation (water balance). Volume lindi bulanan dari TPA. Ref: EPA HELP Model."
    )]
    fn leachate_generation(&self, Parameters(p): Parameters<LeachateParam>) -> String {
        let rain: Result<Vec<f64>, _> = serde_json::from_str(&p.monthly_rainfall_json);
        let et: Result<Vec<f64>, _> = serde_json::from_str(&p.monthly_et_json);
        match (rain, et) {
            (Ok(r), Ok(e)) => tools::waste::leachate::calculate(
                p.area_m2,
                &r,
                &e,
                p.soil_storage_mm,
                p.runoff_coeff,
            ),
            _ => "ERROR: JSON parsing gagal. Format: [jan,feb,...,des] (12 nilai).".into(),
        }
    }

    #[tool(
        description = "Landfill Slope Stability (infinite slope). FoS analysis. Min 1.3 static. Ref: PermenPU, Bishop."
    )]
    fn landfill_stability(&self, Parameters(p): Parameters<LandfillStabilityParam>) -> String {
        tools::waste::landfill_stability::calculate(
            p.slope_angle_deg,
            p.height_m,
            p.unit_weight_kn_m3,
            p.cohesion_kpa,
            p.friction_deg,
            p.pore_pressure_ratio,
        )
    }

    #[tool(description = "TCLP Screening. Karakteristik limbah B3. Ref: PP 101/2014, EPA SW-846.")]
    fn tclp_screening(&self, Parameters(p): Parameters<TclpParam>) -> String {
        tools::waste::tclp::screen(&p.parameters_json)
    }

    #[tool(
        description = "Waste Compatibility Matrix. Cek kompatibilitas penyimpanan 2 jenis limbah B3."
    )]
    fn waste_compatibility(&self, Parameters(p): Parameters<WasteCompatParam>) -> String {
        tools::waste::waste_compatibility::check(&p.waste_a, &p.waste_b)
    }

    #[tool(
        description = "TPS B3 Storage Calculator. Luas lantai, containment, persyaratan. Ref: PP 101/2014."
    )]
    fn b3_storage_calc(&self, Parameters(p): Parameters<B3StorageParam>) -> String {
        tools::waste::b3_storage::calculate(&p.waste_type, p.volume_m3_per_month, p.density_kg_m3)
    }

    // =====================================================
    // GOD TIER PHASE 2: RADIATION & NUCLEAR
    // =====================================================

    #[tool(
        description = "Inverse Square Law Radiasi. Laju dosis vs jarak. Jarak aman pekerja/publik."
    )]
    fn radiation_inverse_square(&self, Parameters(p): Parameters<InverseSquareParam>) -> String {
        tools::radiation::inverse_square::calculate(p.dose_rate_at_d1, p.d1_m, p.d2_m)
    }

    #[tool(description = "Shielding Radiasi. HVL lead/concrete/water/steel. Ref: ICRP.")]
    fn radiation_shielding(&self, Parameters(p): Parameters<ShieldingParam>) -> String {
        tools::radiation::shielding::calculate(
            p.initial_intensity,
            &p.material,
            p.thickness_cm,
            &p.source,
        )
    }

    #[tool(
        description = "Radioactive Decay: A(t) = A₀×e^(-λt). 10 isotop. Waktu ke clearance level BAPETEN."
    )]
    fn radioactive_decay(&self, Parameters(p): Parameters<DecayParam>) -> String {
        tools::radiation::radioactive_decay::calculate(
            &p.isotope,
            p.initial_activity_bq,
            p.time_elapsed,
            &p.time_unit,
        )
    }

    #[tool(
        description = "Radon Indoor Estimation. Konsentrasi Rn-222 dalam ruangan. Ref: WHO 2009 (100 Bq/m³)."
    )]
    fn radon_indoor(&self, Parameters(p): Parameters<RadonParam>) -> String {
        tools::radiation::radon_indoor::calculate(
            p.soil_radon_bq_m3,
            p.floor_area_m2,
            p.room_height_m,
            p.ventilation_rate_ach,
            &p.floor_type,
        )
    }

    #[tool(
        description = "NORM Screening. Timah/monazite/zircon/coal ash. Ref: PerKa BAPETEN 4/2013."
    )]
    fn norm_screening(&self, Parameters(p): Parameters<NormParam>) -> String {
        tools::radiation::norm_screening::screen(&p.material, p.activity_bq_g)
    }

    // =====================================================
    // GOD TIER PHASE 2: HEALTH RISK & MONITORING
    // =====================================================

    #[tool(description = "HHRA Cancer Risk (ILCR). Multi-pathway exposure. Ref: US EPA RAGS.")]
    fn hhra_cancer_risk(&self, Parameters(p): Parameters<HhraParam>) -> String {
        tools::biodiversity::hhra::calculate_ilcr(
            &p.exposure_route,
            p.concentration,
            p.intake_rate,
            p.exposure_freq_days,
            p.exposure_dur_years,
            p.body_weight_kg,
            p.avg_time_years,
            p.csf,
        )
    }

    #[tool(
        description = "Hazard Quotient (HQ) Non-Cancer Risk. Auto-lookup RfD dari IRIS database. Ref: US EPA IRIS, Pedoman ARKL."
    )]
    fn hhra_hazard_quotient(&self, Parameters(p): Parameters<HqParam>) -> String {
        tools::biodiversity::hhra::calculate_hq(
            &p.contaminant,
            &p.route,
            p.concentration,
            p.intake_rate,
            p.exposure_freq_days,
            p.exposure_dur_years,
            p.body_weight_kg,
        )
    }

    #[tool(
        description = "ARKL Indonesia (Analisis Risiko Kesehatan Lingkungan). Default Indonesia: BW=55kg, fE=350, Dt=30. Ref: Pedoman ARKL Kemenkes 2012."
    )]
    fn arkl_calculator(&self, Parameters(p): Parameters<ArklParam>) -> String {
        tools::biodiversity::hhra::calculate_arkl(
            &p.contaminant,
            &p.route,
            p.concentration,
            &p.population_type,
            &p.exposure_scenario,
        )
    }

    #[tool(
        description = "Sampling Design Calculator. Jumlah sampel + strategi. Ref: ISO 5667, EPA QA/G-5S."
    )]
    fn sampling_design(&self, Parameters(p): Parameters<SamplingParam>) -> String {
        tools::biodiversity::sampling_design::calculate(
            p.confidence_pct,
            p.margin_error_pct,
            p.std_deviation,
            p.population_size,
        )
    }

    #[tool(
        description = "Mann-Kendall Trend Test + Sen's Slope. Deteksi tren data monitoring lingkungan. Ref: Mann 1945."
    )]
    fn mann_kendall_trend(&self, Parameters(p): Parameters<MannKendallParam>) -> String {
        tools::biodiversity::mann_kendall::trend_test(&p.data_json)
    }

    #[tool(
        description = "QA/QC Data Validation. RPD duplikat, spike recovery, blank check. Ref: EPA 40 CFR 136."
    )]
    fn qaqc_validation(&self, Parameters(p): Parameters<QaqcParam>) -> String {
        tools::biodiversity::qaqc::validate(&p.data_json)
    }

    #[tool(
        description = "Coliform Die-off Decay (Mancini model). T90 tropis. Kepatuhan PP 22/2021 coliform. Ref: Mancini 1978."
    )]
    fn coliform_decay(&self, Parameters(p): Parameters<ColiformParam>) -> String {
        tools::biodiversity::coliform_decay::calculate(
            p.initial_count_per_100ml,
            p.temperature_c,
            p.time_hours,
            &p.water_type,
        )
    }

    // =====================================================
    // GOD TIER PHASE 2: ECOLOGICAL & COASTAL
    // =====================================================

    #[tool(
        description = "Bruun Rule Coastal Erosion. Resesi pantai akibat SLR. Skenario IPCC AR6. Ref: Bruun 1962."
    )]
    fn bruun_rule(&self, Parameters(p): Parameters<BruunParam>) -> String {
        tools::ocean_modeling::bruun_rule::calculate(
            p.sea_level_rise_m,
            p.profile_length_m,
            p.berm_height_m,
            p.closure_depth_m,
        )
    }

    #[tool(
        description = "Coastal Vulnerability Index (CVI). 6 variabel: geomorfologi, perubahan garis pantai, kemiringan, SLR, gelombang, pasut."
    )]
    fn coastal_vulnerability(&self, Parameters(p): Parameters<CviParam>) -> String {
        tools::ocean_modeling::coastal_vulnerability::calculate(
            p.geomorphology,
            p.shoreline_change_m_yr,
            p.coastal_slope_pct,
            p.slr_mm_yr,
            p.mean_wave_height_m,
            p.mean_tidal_range_m,
        )
    }

    #[tool(
        description = "Traffic Noise Model (CoRTN). Kebisingan lalu lintas jalan. Line source → contour. + KepmenLH 48/1996."
    )]
    fn traffic_noise(&self, Parameters(p): Parameters<TrafficNoiseParam>) -> String {
        tools::noise::traffic_noise::calculate(
            p.vehicles_per_hour,
            p.speed_kmh,
            p.distance_m,
            p.heavy_vehicle_pct,
            p.gradient_pct,
            &p.ground_type,
            p.barrier_height_m,
        )
    }

    #[tool(
        description = "Bioretention / Rain Garden Design. Green infrastructure BMP. Sizing + media + tanaman Indonesia."
    )]
    fn bioretention_design(&self, Parameters(p): Parameters<BioretentionParam>) -> String {
        tools::calculators::bioretention::design(
            p.q_design_m3s,
            p.ksat_m_hr,
            p.ponding_depth_m,
            p.media_depth_m,
            p.drain_time_hr,
        )
    }

    #[tool(
        description = "Water Footprint ISO 14046. Blue/green/grey WF. 17 produk Indonesia. Ref: Hoekstra 2011."
    )]
    fn water_footprint(&self, Parameters(p): Parameters<WaterFootprintParam>) -> String {
        tools::calculators::water_footprint::calculate(&p.product, p.quantity, &p.unit)
    }

    // =====================================================
    // GOD TIER PHASE 2: ECONOMICS & INDUSTRIAL ECOLOGY
    // =====================================================

    #[tool(
        description = "Cost-Benefit Analysis (NPV/BCR/IRR). Analisis ekonomi proyek lingkungan. Sensitivity ±10-20%."
    )]
    fn cost_benefit_analysis(&self, Parameters(p): Parameters<CbaParam>) -> String {
        tools::esg::cost_benefit::calculate(
            &p.costs_json,
            &p.benefits_json,
            p.discount_rate,
            p.years,
        )
    }

    #[tool(
        description = "Material Flow Analysis (MFA). Mass balance industri. Efisiensi + waste ratio. Ref: Brunner & Rechberger."
    )]
    fn material_flow_analysis(&self, Parameters(p): Parameters<MfaParam>) -> String {
        tools::esg::material_flow::analyze(&p.inputs_json, &p.outputs_json, p.stock_change)
    }

    #[tool(
        description = "GHG Protocol Scope 1/2/3. Emisi korporat per kategori. EF Indonesia (Perpres 98/2021)."
    )]
    fn scope_123_ghg(&self, Parameters(p): Parameters<Scope123Param>) -> String {
        tools::esg::scope123::calculate(&p.scope1_json, &p.scope2_json, &p.scope3_json)
    }

    #[tool(
        description = "Circular Economy MCI. Material Circularity Indicator. Ref: Ellen MacArthur Foundation 2015."
    )]
    fn circular_economy_mci(&self, Parameters(p): Parameters<CircularParam>) -> String {
        tools::esg::circular_economy::calculate(
            p.mass_product_kg,
            p.virgin_feedstock_pct,
            p.recycled_input_pct,
            p.reused_input_pct,
            p.recycled_output_pct,
            p.reused_output_pct,
            p.product_lifetime_years,
            p.industry_avg_lifetime,
        )
    }

    #[tool(
        description = "Externality / Damage Cost. Biaya kerusakan lingkungan per polutan. Social cost of carbon. Konteks Indonesia."
    )]
    fn externality_cost(&self, Parameters(p): Parameters<ExternalityParam>) -> String {
        tools::esg::externality_cost::calculate(&p.pollutant, p.amount, &p.unit, &p.location_type)
    }

    // =====================================================
    // GIS / REMOTE SENSING — REAL IMPLEMENTATIONS
    // =====================================================

    #[tool(
        description = "Raster Band Math via GEE Sentinel-2. Compute spectral indices: NDVI/NDWI/SAVI/EVI/MNDWI/NDBI/BSI. Output: GeoTIFF."
    )]
    fn raster_band_math(&self, Parameters(p): Parameters<RasterBandMathParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::gis::advanced::band_math_gee(
            p.lat,
            p.lon,
            p.buffer_km,
            &p.index_type,
            &p.start_date,
            &p.end_date,
            &p.output_path,
        )
    }

    #[tool(
        description = "Raster Band Math on Local GeoTIFF. Custom expression e.g. '(b1-b2)/(b1+b2)'. Output: GeoTIFF."
    )]
    fn raster_band_math_local(
        &self,
        Parameters(p): Parameters<RasterBandMathLocalParam>,
    ) -> String {
        tools::gis::advanced::band_math_local(&p.input_path, &p.expression, &p.output_path)
    }

    #[tool(
        description = "DEM Slope Analysis via GEE SRTM 30m. Kemiringan lereng (derajat). Output: GeoTIFF."
    )]
    fn dem_slope_gee(&self, Parameters(p): Parameters<DemGeeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::gis::advanced::dem_slope(p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    #[tool(
        description = "DEM Aspect Analysis via GEE SRTM 30m. Arah hadap lereng (0-360°). Output: GeoTIFF."
    )]
    fn dem_aspect_gee(&self, Parameters(p): Parameters<DemGeeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::gis::advanced::dem_aspect(p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    #[tool(
        description = "DEM Hillshade via GEE SRTM 30m. Bayangan relief untuk visualisasi terrain. Output: GeoTIFF."
    )]
    fn dem_hillshade_gee(&self, Parameters(p): Parameters<DemGeeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::gis::advanced::dem_hillshade(p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    #[tool(
        description = "Zonal Statistics via GEE reduceRegion. Stats dari image_id+band di dalam polygon/buffer. Output: JSON."
    )]
    fn zonal_statistics_gee(&self, Parameters(p): Parameters<ZonalStatsGeeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        let geojson = p.geojson.as_deref().unwrap_or("");
        tools::gis::advanced::raster_stats(
            &p.image_id,
            &p.band,
            geojson,
            p.lat,
            p.lon,
            p.buffer_km,
            &p.output_path,
        )
    }

    #[tool(
        description = "Zonal Statistics Local. Hitung min/max/mean/std/sum/count raster di zona vektor. Pure local (rasterstats)."
    )]
    fn zonal_statistics_local(&self, Parameters(p): Parameters<ZonalStatsLocalParam>) -> String {
        tools::gis::advanced::zonal_stats_local(&p.raster_path, &p.vector_path, &p.stats)
    }

    #[tool(
        description = "Land Cover Classification via GEE Sentinel-2. Dynamic World + SNI 7645:2014. Output: classified GeoTIFF."
    )]
    fn land_cover_classify(&self, Parameters(p): Parameters<LandCoverClassifyParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        match tools::gis::landcover::classify(
            p.lat,
            p.lon,
            p.buffer_km,
            &p.start_date,
            &p.end_date,
            &p.output_path,
        ) {
            Ok(res) => serde_json::to_string_pretty(&res).unwrap_or_else(|e| format!("JSON Error: {}", e)),
            Err(e) => e,
        }
    }

    #[tool(
        description = "Land Use Change Detection. Banding 2 periode citra Sentinel-2 via GEE. Deteksi deforestasi/urbanisasi. Output: change map."
    )]
    fn land_use_change(&self, Parameters(p): Parameters<LandUseChangeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::gis::landcover::change_detection(
            p.lat,
            p.lon,
            p.buffer_km,
            &p.d1_start,
            &p.d1_end,
            &p.d2_start,
            &p.d2_end,
            &p.output_path,
        )
    }

    #[tool(
        description = "Classification Accuracy Assessment (Python landcover engine). Confusion matrix, Kappa, Producer/User accuracy. Ref: SNI 8202:2015."
    )]
    fn accuracy_assessment(&self, Parameters(p): Parameters<AccuracyAssessmentParam>) -> String {
        tools::gis::landcover::accuracy_assessment(&p.predicted_json, &p.actual_json)
    }

    #[tool(
        description = "Classification Accuracy Assessment (pure Rust). Confusion matrix, Kappa, OA, SNI 8202:2015 compliance. No Python dependency."
    )]
    fn accuracy_assessment_rs(&self, Parameters(p): Parameters<AccuracyAssessmentParam>) -> String {
        tools::calculators::accuracy_assessment::calculate(&p.predicted_json, &p.actual_json)
    }

    #[tool(
        description = "Buffer Analysis. Create buffer zone around GeoJSON geometry. Output: buffered GeoJSON."
    )]
    fn buffer_analysis(&self, Parameters(p): Parameters<BufferAnalysisParam>) -> String {
        tools::gis::spatial_ops::buffer(&p.geojson, p.distance_m, &p.output_path)
    }

    #[tool(
        description = "Overlay Analysis. Intersection/union/difference/symmetric_difference of 2 GeoJSON layers. Output: GeoJSON."
    )]
    fn overlay_analysis(&self, Parameters(p): Parameters<OverlayAnalysisParam>) -> String {
        tools::gis::spatial_ops::overlay(&p.geojson_a, &p.geojson_b, &p.operation, &p.output_path)
    }

    #[tool(
        description = "Suitability Analysis. Multi-criteria evaluation via GEE layers. Output: suitability map."
    )]
    fn suitability_analysis(&self, Parameters(p): Parameters<SuitabilityAnalysisParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::gis::spatial_ops::suitability(
            &p.criteria_json,
            p.lat,
            p.lon,
            p.buffer_km,
            &p.output_path,
        )
    }

    #[tool(
        description = "Viewshed Analysis. Line-of-sight visibility dari DEM. Untuk AMDAL visual impact, tower placement. Output: visibility map."
    )]
    fn viewshed_analysis(&self, Parameters(p): Parameters<ViewshedAnalysisParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.observer_lat, p.observer_lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::gis::viewshed::analyze(
            &p.dem_path,
            p.observer_lat,
            p.observer_lon,
            p.observer_height_m,
            p.max_distance_m,
            &p.output_path,
        )
    }

    #[tool(
        description = "Coordinate Transform V2. Transform between any EPSG CRS. Input: x, y, from_epsg, to_epsg."
    )]
    fn coordinate_transform_v2(&self, Parameters(p): Parameters<CoordTransformV2Param>) -> String {
        let from = format!("EPSG:{}", p.from_epsg);
        let to = format!("EPSG:{}", p.to_epsg);
        tools::gis::coords::transform(p.x, p.y, &from, &to)
    }

    #[tool(
        description = "WGS84 to UTM Auto. Auto-detect UTM zone for Indonesia coordinates. Returns easting, northing, zone, EPSG."
    )]
    fn wgs84_to_utm(&self, Parameters(p): Parameters<Wgs84ToUtmParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: Koordinat tidak valid - {}", e);
        }
        tools::gis::coords::wgs84_to_utm_auto(p.lat, p.lon)
    }

    // =====================================================
    // RESEARCH-GRADE GIS/REMOTE SENSING
    // =====================================================

    #[tool(
        description = "Olofsson Area-Weighted Accuracy Assessment. Ref: Olofsson et al. 2014 (NASA standard). Unbiased area estimates + CI dari stratified random sampling."
    )]
    fn olofsson_accuracy(&self, Parameters(p): Parameters<OlofssonParam>) -> String {
        tools::calculators::olofsson::calculate(
            &p.mapped_areas_json,
            &p.confusion_matrix_json,
            &p.class_names_json,
            p.z_score.unwrap_or(1.96),
        )
    }

    #[tool(
        description = "Random Forest Supervised Classification via GEE smileRandomForest. Ref: Nur et al. 2025. Input: training GeoJSON polygons + date range."
    )]
    fn supervised_classification(&self, Parameters(p): Parameters<SupervisedRfParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: {}", e);
        }
        tools::gis::landcover::supervised_classify(
            p.lat,
            p.lon,
            p.buffer_km,
            &p.training_geojson,
            &p.start_date,
            &p.end_date,
            p.n_trees.unwrap_or(100),
            &p.output_path,
        )
    }

    #[tool(
        description = "Topographic C-Correction. Ref: Teillet et al. 1982. Koreksi efek terrain pada reflectance S2. Otomatis skip area datar (slope<5°)."
    )]
    fn topo_correction(&self, Parameters(p): Parameters<TopoCorrectParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: {}", e);
        }
        tools::gis::advanced::topo_correction(
            p.lat,
            p.lon,
            p.buffer_km,
            &p.start_date,
            &p.end_date,
            &p.output_path,
        )
    }

    #[tool(
        description = "NDVI Time Series Trend Analysis. Ref: Saifulloh et al. 2025. Annual composites + linear regression slope per pixel."
    )]
     fn ndvi_timeseries(&self, Parameters(p): Parameters<NdviTimeseriesParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: {}", e);
        }
        tools::gis::advanced::ndvi_timeseries(
            p.lat,
            p.lon,
            p.buffer_km,
            p.start_year as i32,
            p.end_year as i32,
            &p.output_path,
        )
    }

    // ═══ GOD TIER v3: 9 Advanced Modeling Tools ═══

    #[tool(description = "Enhanced Leopold Matrix (AHP + TOPSIS). Leopold M×I + AHP pairwise weights (CR<0.1 via power iteration eigenvalue) + TOPSIS alternative ranking. Optional pairwise_matrix_json for true AHP consistency check. Ref: Leopold 1971; Saaty 1980; Shiraishi 2025; Zhang 2026.")]
    fn enhanced_leopold_matrix(&self, Parameters(p): Parameters<EnhancedLeopoldParam>) -> String {
        tools::calculators::enhanced_leopold::assess_full(
            &p.impacts_json, &p.criteria_weights_json, &p.alternatives_json,
            p.pairwise_matrix_json.as_deref().unwrap_or("")
        )
    }

    #[tool(description = "Enhanced LCA (Multi-Category). ISO 14040/14044. GWP (IPCC AR6) + AP + EP + ODP + Water + Energy. 27 material DB + regression fallback. Transport + grid energy. Ref: Luan 2026; Arumugam 2026; Guleria 2026.")]
    fn lca_enhanced(&self, Parameters(p): Parameters<LcaEnhancedParam>) -> String {
        tools::calculators::lca_enhanced::calculate(
            &p.materials_json, p.transport_kg_km, p.energy_kwh
        )
    }

    #[tool(description = "Environmental Management Plan (RKL-RPL + KPI). PermenLHK 5/2021. Auto-generate RKL mitigation + RPL monitoring + KPI scoring + ISO 14001 linkage (Clause 8.1/9.1). Ref: Anggreini 2026; Rani 2026.")]
    fn environmental_management_plan(&self, Parameters(p): Parameters<EmpParam>) -> String {
        tools::amdal::emp_generator::generate(
            &p.impacts_json, &p.project_type, &p.location
        )
    }

    #[tool(description = "ISO 14001:2015 Gap Analysis + PROPER Prediction. Clause 4-10 (HLS/PDCA) + 5-point compliance + PROPER rating (HITAM→EMAS). Ref: Falakh 2026; Febrian 2026; Altarazi 2026. PermenLHK P.1/2021.")]
    fn iso14001_gap_analysis(&self, Parameters(p): Parameters<Iso14001GapParam>) -> String {
        tools::compliance::iso14001_gap::assess(&p.compliance_json)
    }

    #[tool(description = "TRIGRS Hybrid Landslide (Physics + ML). 1D infiltration FD (Richards) + Mohr-Coulomb FS + logistic ML probability. Ref: Baum 2008 (USGS); Sugianti 2026 (TRIGRSMap Indonesia); Peng 2026 (hybrid); Jiao 2026.")]
    fn trigrs_landslide_hybrid(&self, Parameters(p): Parameters<TrigrsParam>) -> String {
        tools::advanced_physics::trigrs::assess(
            p.rainfall_mm_hr, p.duration_hr, p.ks_m_s, p.d2_m,
            p.cohesion_kpa, p.friction_angle_deg, p.slope_deg, p.depth_m,
            p.porosity, p.unit_weight_kn_m3
        )
    }

    #[tool(description = "MODFLOW 6 3D Groundwater (FloPy bridge). Steady/transient head + drawdown. Falls back to Theis analytical if flopy missing. Ref: USGS MODFLOW 6; Dharma 2026 (Seulawah Agam Aceh). Install: pip install flopy numpy.")]
    fn modflow_groundwater_3d(&self, Parameters(p): Parameters<ModflowParam>) -> String {
        tools::water::modflow_3d::assess(
            p.grid_nlay, p.grid_nrow, p.grid_ncol, p.cell_size_m,
            p.hk_m_s, p.vk_m_s, p.sy, p.ss_per_m,
            p.pumping_m3_day, p.pumping_x, p.pumping_y, p.pumping_layer,
            p.recharge_mm_yr, p.chb_head_m, &p.sim_type, p.duration_days
        )
    }

    #[tool(description = "MintPy InSAR SBAS Displacement. Sentinel-1 time series → mm/yr subsidence/uplift. Ref: Yunjun 2019; Widiarso 2026 (Semarang); Umarhadi 2026 (peatland); Pratama 2026 (Jatiluhur). Install: conda install mintpy isce2.")]
    fn mintpy_insar(&self, Parameters(p): Parameters<MintpyInsarParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) {
            return format!("ERROR [E101]: {}", e);
        }
        tools::satellite::mintpy_insar::assess(
            p.lat, p.lon, &p.date_start, &p.date_end, p.bbox_km.unwrap_or(10.0)
        )
    }

    #[tool(description = "EnKF Data Assimilation. Ensemble Kalman Filter for IoT sensor fusion. Forecast → Update → Uncertainty reduction. Ref: Evensen 1994; Sun 2026 (ADAPT); Sahar 2026; Zhao 2026. Input: model_states_json, observations_json, noise_std.")]
    fn enkf_data_assimilation(&self, Parameters(p): Parameters<EnkfParam>) -> String {
        tools::advanced_physics::enkf::assimilate(
            &p.model_states_json, &p.observations_json,
            p.ensemble_size.unwrap_or(50), p.noise_std
        )
    }

    #[tool(description = "AERMOD Pro-Justitia Input Generator (Tier 3). Menghasilkan file .inp standar EPA/KLHK untuk pemodelan dispersi PLTU/Smelter.")]
    fn aermod_generator(&self, Parameters(p): Parameters<AermodGeneratorParam>) -> String {
        tools::airquality::aermod_generator::generate_aermod_inp(
            &p.project_name, p.source_lat, p.source_lon, p.stack_height_m,
            p.stack_diameter_m, p.exit_velocity_m_s, p.exit_temp_k,
            p.emission_rate_g_s, &p.pollutant_id, p.is_rural
        )
    }

    #[tool(description = "PHREEQC Geochemical Leaching Generator (Tier 3). Menghasilkan script termodinamika USGS untuk prediksi pelindian logam berat (Tailing Nikel/B3) berdasar pH.")]
    fn phreeqc_leaching(&self, Parameters(p): Parameters<PhreeqcLeachingParam>) -> String {
        tools::waste::phreeqc_leaching::generate_phreeqc_script(
            &p.waste_type, p.solid_mass_g, p.water_volume_l, p.target_ph, &p.initial_metals_mg_kg
        )
    }

    #[tool(description = "Fire Spread (Rothermel + Cellular Automata + Monte Carlo). Anderson 13 fuel models. CA 2D grid propagation + 50-member ensemble uncertainty. Ref: Rothermel 1972; Karakonstantis 2026; Sindhuja 2026. For karhutla Indonesia.")]
    fn fire_spread_ca(&self, Parameters(p): Parameters<FireSpreadParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.ignition_lat, p.ignition_lon) {
            return format!("ERROR [E101]: {}", e);
        }
        tools::advanced_physics::fire_spread::assess(
            p.fuel_model, p.wind_speed_ms, p.wind_dir_deg, p.slope_deg,
            p.moisture_pct, p.ignition_lat, p.ignition_lon, p.duration_hr
        )
    }

    #[tool(description = "Fire Danger Rating System (FDRS) — Keetch-Byram Drought Index (KBDI). Early warning untuk prediksi risiko karhutla hutan dan gambut. Ref: Keetch & Byram (1968); BMKG FDRS.")]
    fn fire_danger_rating(&self, Parameters(p): Parameters<FireDangerRatingParam>) -> String {
        tools::advanced_physics::fire_danger_rating::calculate(
            p.kbdi_yesterday, p.max_temp_c, p.mean_annual_precip_mm, p.daily_precip_mm, p.is_peatland
        )
    }

    #[tool(description = "Source Apportionment PM2.5 (CMB). Memisahkan kontribusi PLTU, Kendaraan, Pembakaran, dan Debu berdasarkan chemical signature (SO4, NO3, EC, OC). Ref: EPA CMB 8.2; Vital Strategies (2025).")]
    fn pm_source_apportionment(&self, Parameters(p): Parameters<SourceApportionmentParam>) -> String {
        tools::airquality::source_apportionment::assess(
            p.pm25_total_ug_m3, p.so4_ug_m3, p.no3_ug_m3, p.ec_ug_m3, p.oc_ug_m3, p.crustal_ug_m3
        )
    }

    #[tool(
        description = "PMF (Positive Matrix Factorization) source apportionment. Decomposes a concentration matrix X = G·F + E via weighted non-negative matrix factorization (multiplicative ALS) into factor contributions G and source profiles F, minimising Q = Σ((X-GF)/σ)². Input: X and σ as JSON 2D arrays, number of factors, iterations. Returns factor profiles, per-sample contributions, Q and Q/Qexp. Ref: Paatero & Tapper 1994; EPA PMF 5.0. Use for data-driven (non-CMB) PM2.5 source identification."
    )]
    fn pmf_source_apportionment(&self, Parameters(p): Parameters<PmfParam>) -> String {
        tools::airquality::source_apportionment::pmf_apportionment(&p)
    }

    #[tool(description = "Deep-Sea Tailings Placement (DSTP) Plume Dispersion. Menghitung dispersi tailing di laut dalam untuk industri nikel. Ref: Shimmield et al. 2010; Reichelt-Brushett 2012.")]
    fn dstp_plume_dispersion(&self, Parameters(p): Parameters<DstpPlumeParam>) -> String {
        tools::ocean_modeling::dstp_plume::assess(
            p.discharge_depth_m, p.tailings_volume_m3_day, p.solid_fraction_pct,
            p.ocean_current_speed_m_s, p.settling_velocity_mm_s
        )
    }

    #[tool(description = "Aerial Wildfire Suppression Optimizer. Two-stage gradient-based intervention design through 3-state CA model. Water vs retardant physics. Fleet config from Matei 2026. Ref: Matei et al. 2026 (arXiv:2606.13633). For BNPB/Manggala Agni operations.")]
    fn fire_suppression_optimizer(&self, Parameters(p): Parameters<FireSuppressionParam>) -> String {
        tools::advanced_physics::fire_suppression::assess(
            p.fire_area_ha, p.duration_hr, p.n_aircraft, &p.aircraft_mix,
            p.wind_speed_ms, p.wind_dir_deg, p.fuel_model, p.budget_drops
        )
    }

    // ═══ ENVIRONMENTAL ENGINEERING DESIGN TOOLS (11) ═══

    #[tool(description = "Pump-and-Treat Remediation Design. Capture zone, drawdown (Theis), pore volume, cleanup time, mass removal. Ref: Suthersan 2016; Sharma & Reddy 2004; US EPA 1989.")]
    fn pump_and_treat_design(&self, Parameters(p): Parameters<PumpTreatParam>) -> String {
        tools::water::pump_treat::design(
            p.hk_m_s, p.aquifer_thickness_m, p.hydraulic_gradient,
            p.pumping_rate_m3_day, p.porosity, &p.contaminant,
            p.initial_conc_ug_l, p.target_conc_ug_l, p.cleanup_time_years
        )
    }

    #[tool(description = "Permeable Reactive Barrier (PRB) Design. ZVI thickness, residence time, outlet concentration, mass, longevity. Auto k for TCE/PCE/Cr6/As. Ref: Tratnyek 2003; Seyyedalipour 2026.")]
    fn permeable_reactive_barrier(&self, Parameters(p): Parameters<PrbDesignParam>) -> String {
        tools::water::prb_design::design(
            &p.contaminant, p.c_inflow_ug_l, p.c_target_ug_l,
            p.k_first_order_hr, p.gw_velocity_m_day, p.porosity,
            p.barrier_width_m, p.barrier_depth_m, p.bulk_density_kg_m3
        )
    }

    #[tool(description = "Soil Vapor Extraction (SVE) Design. Airflow rate (radial), radius of influence, vapor concentration (Raoult), cleanup time. Ref: Staudinger 1997; Lowe 1999; Shi 2022.")]
    fn soil_vapor_extraction(&self, Parameters(p): Parameters<SveDesignParam>) -> String {
        tools::waste::sve_design::design(
            p.k_air_m2, p.screen_length_m, p.vacuum_pressure_kpa,
            &p.contaminant, p.napl_mass_kg, p.soil_porosity, p.soil_temp_c,
            p.cleanup_time_target_days
        )
    }

    #[tool(description = "Bioremediation Design. Monod/first-order kinetics, cleanup time, O₂ demand, nutrient demand (C:N:P=100:10:1). Auto k for BTEX/PAH/TCE. Ref: Chen 1992; Suarez & Rifai 1999.")]
    fn bioremediation_design(&self, Parameters(p): Parameters<BioremediationParam>) -> String {
        tools::calculators::bioremediation::design(
            &p.contaminant, p.initial_conc_mg_l, p.target_conc_mg_l,
            p.k_first_order_day, p.soil_volume_m3, p.porosity, p.bulk_density_kg_m3
        )
    }

    #[tool(description = "Cyclone Separator Design. Stairmand dimensions, cut diameter (d50), Lapple efficiency, Shepherd-Lapple pressure drop, Stokes number. Ref: Aylı 2025; Vallero 2019.")]
    fn cyclone_separator_design(&self, Parameters(p): Parameters<CycloneSeparatorParam>) -> String {
        tools::airquality::cyclone::design(
            p.gas_flow_m3_s, p.particle_density_kg_m3, p.gas_viscosity_pa_s,
            p.cyclone_diameter_m, p.target_efficiency_pct
        )
    }

    #[tool(description = "Baghouse Filter Design. Filtration velocity (air-to-cloth), bag count, cleaning cycle, pressure drop, compartments. Fabric: woven/polyester/felt/PTFE/fiberglass. Ref: Vallero 2019.")]
    fn baghouse_filter_design(&self, Parameters(p): Parameters<BaghouseParam>) -> String {
        tools::airquality::baghouse::design(
            p.gas_flow_m3_s, p.dust_conc_g_m3, p.target_pressure_drop_pa,
            p.bag_diameter_m, p.bag_length_m, &p.fabric_type
        )
    }

    #[tool(description = "Wet Scrubber (Venturi) Design. Nukiyama-Tanasawa droplet size, Calvert efficiency, pressure drop, water/power consumption. Ref: Vallero 2019; Calvert 1972.")]
    fn wet_scrubber_design(&self, Parameters(p): Parameters<ScrubberParam>) -> String {
        tools::airquality::scrubber::design(
            p.gas_flow_m3_s, p.particle_density_kg_m3, p.target_efficiency_pct,
            p.throat_velocity_ms, p.lg_ratio_l_m3
        )
    }

    #[tool(description = "Electrostatic Precipitator (ESP) Design. Deutsch-Anderson migration (conductive vs dielectric), back-corona resistivity check, size-integrated efficiency. Ref: Vallero 2019; White 1963.")]
    fn electrostatic_precipitator(&self, Parameters(p): Parameters<EspParam>) -> String {
        let pt_str = match p.particle_type {
            ParticleType::Dielectric => "dielectric",
            ParticleType::Conductive => "conductive",
        };
        tools::airquality::esp::design(
            p.gas_flow_m3_s, p.particle_density_kg_m3, p.target_efficiency_pct,
            p.field_strength_kv_cm, p.particle_diameter_um,
            pt_str, p.resistivity_ohm_cm
        )
    }

    #[tool(description = "Reverse Osmosis (RO) Design. van't Hoff osmotic pressure, water flux (A×(ΔP-Δπ)), salt rejection, recovery, membrane area, energy. Ref: Crittenden 2012 (MWH); Biesheuvel 2023.")]
    fn reverse_osmosis_design(&self, Parameters(p): Parameters<RoDesignParam>) -> String {
        tools::water::ro_design::design(
            p.feed_salinity_mg_l, p.target_permeate_mg_l, p.feed_pressure_bar,
            p.membrane_water_perm_l_m2_h_bar, p.membrane_salt_perm_l_m2_h,
            p.feed_flow_m3_day, p.temp_c
        )
    }

    #[tool(description = "Activated Carbon (GAC) Design. Freundlich isotherm, bed volume, bed life (Bohart-Adams), carbon usage rate, bed geometry. Ref: Crittenden 2012 (MWH).")]
    fn activated_carbon_design(&self, Parameters(p): Parameters<GacDesignParam>) -> String {
        tools::water::gac_design::design(
            &p.contaminant, p.c_influent_mg_l, p.c_target_mg_l, p.flow_m3_day,
            p.freundlich_k, p.freundlich_1_over_n, p.ebct_min
        )
    }

    #[tool(description = "Ion Exchange Design. Exchange capacity, throughput (BV), regeneration cycle, regenerant consumption, leakage. Ions: Ca/Mg/Na/NO3/Cl/SO4/Fe/Cr. Ref: Crittenden 2012 (MWH).")]
    fn ion_exchange_design(&self, Parameters(p): Parameters<IonExchangeParam>) -> String {
        tools::water::ion_exchange::design(
            &p.target_ion, p.c_influent_mg_l, p.exchange_capacity_eq_l,
            p.flow_m3_day, p.bed_volume_m3, p.selectivity_coeff, &p.regenerant_type
        )
    }

    #[tool(description = "Contaminant Transport 1D (Ogata-Banks). Analytical solution for 1D advection-dispersion with retardation and decay. C/C0 = 0.5*erfc((x-v*t)/(2*sqrt(D*t))). Ref: Ogata & Banks 1961; Freeze & Cherry 1979.")]
    fn contaminant_transport_1d(&self, Parameters(p): Parameters<ContaminantTransport1DParam>) -> String {
        tools::water::contaminant_transport_1d::assess(
            p.distance_m, p.velocity_m_day, p.dispersion_m2_day, p.time_days,
            p.retardation_factor, p.decay_rate_day, p.initial_conc_mg_l
        )
    }

    #[tool(
        description = "ADR (Advection-Dispersion-Reaction) 1D Solver with Non-Linear Sorption. Finite difference solver handling concentration-dependent retardation R(C). Supports 'linear' (Kd), 'freundlich' (Kf, n), and 'langmuir' (b, Smax) sorption isotherms. Ref: Zheng & Bennett 2002. Computes the spatial profile of concentration and dynamic retardation."
    )]
    fn adr_nonlinear_sorption(&self, Parameters(p): Parameters<AdrSorptionParam>) -> String {
        tools::water::contaminant_transport_1d::solve_adr_sorption(&p)
    }

    #[tool(
        description = "Coupled SWE-Richards Model. Integrates surface water (Shallow Water Equations) and groundwater (Richards PDE) using Head-Flux Switching Boundary Condition. Handles dynamic infiltration and exfiltration between surface and subsurface. Ref: MIKE SHE."
    )]
    fn coupled_swe_richards(&self, Parameters(p): Parameters<CoupledParam>) -> String {
        tools::advanced_physics::coupled_swe_richards::solve_coupled(&p)
    }


    #[tool(description = "Contaminant Transport 2D (Domenico). Analytical solution for 2D advection-dispersion from finite source. Includes transverse dispersion. Ref: Domenico 1987; Devlin 2012.")]
    fn contaminant_transport_2d(&self, Parameters(p): Parameters<ContaminantTransport2DParam>) -> String {
        tools::water::contaminant_transport_2d::assess(
            p.distance_x_m, p.source_width_y_m, p.source_depth_z_m,
            p.velocity_m_day, p.dispersion_x_m2_day, p.dispersion_y_m2_day, p.dispersion_z_m2_day,
            p.time_days, p.retardation_factor, p.decay_rate_day, p.initial_conc_mg_l
        )
    }

    #[tool(description = "Vapor Intrusion (Johnson & Ettinger). Attenuation factor for subsurface vapor to indoor air. Millington-Quirk diffusion, building ventilation. Ref: Johnson & Ettinger 1991; EPA 2017.")]
    fn vapor_intrusion_je(&self, Parameters(p): Parameters<VaporIntrusionParam>) -> String {
        tools::calculators::vapor_intrusion::assess(
            p.source_conc_ug_m3, p.soil_porosity_total, p.soil_porosity_water, p.soil_porosity_air,
            p.stratum_thickness_m, p.bldg_footprint_m2, p.bldg_height_m,
            p.air_exchange_rate_hr, p.crack_area_m2, p.crack_depth_m
        )
    }

    #[tool(description = "River Quality Model (QUAL2K simplified). BOD-DO using Streeter-Phelps. Shows BOD decay and DO sag curve along river. Ref: Chapra 2008.")]
    fn river_quality_model(&self, Parameters(p): Parameters<RiverQualityParam>) -> String {
        tools::calculators::river_quality::assess(
            p.river_length_m, p.flow_m3_s, p.velocity_m_s, p.initial_bod_mg_l, p.initial_do_mg_l,
            p.bod_decay_rate_day, p.reaeration_rate_day, p.saturation_do_mg_l, p.n_reaches
        )
    }

    #[tool(description = "Reaeration Coefficient. Multiple formulas: O'Connor-Dobbins, Churchill, Owens-Gibbs. Temperature-corrected. Ref: Chapra 2008.")]
    fn reaeration_coefficient(&self, Parameters(p): Parameters<ReaerationParam>) -> String {
        tools::calculators::reaeration::assess(p.velocity_m_s, p.depth_m, p.temp_c)
    }

    #[tool(description = "Sediment Oxygen Demand (SOD). Temperature-corrected SOD, total demand, DO depletion in water column. Ref: DiToro 2001; Chapra 2008.")]
    fn sediment_oxygen_demand(&self, Parameters(p): Parameters<SODParam>) -> String {
        tools::calculators::sediment_oxygen_demand::assess(
            p.sod20_g_m2_day, p.temp_c, p.area_m2, p.river_flow_m3_s
        )
    }

    #[tool(description = "Chlorophyll-a Prediction (Vollenweider/OECD). P loading -> P concentration -> Chl-a. Trophic state classification. Ref: Vollenweider 1968; OECD 1982.")]
    fn chlorophyll_a_prediction(&self, Parameters(p): Parameters<ChlorophyllParam>) -> String {
        tools::calculators::chlorophyll_prediction::assess(
            p.phosphorus_load_kg_yr, p.lake_area_km2, p.lake_volume_m3, p.outflow_m3_s, &p.lake_type
        )
    }

    #[tool(description = "MBR (Membrane Bioreactor) Design. Reactor volume, F/M, membrane area, O2 demand, nitrification SRT check. Ref: Judd & Judd 2011.")]
    fn mbr_design(&self, Parameters(p): Parameters<MBRParam>) -> String {
        tools::water::mbr_design::assess(
            p.flow_m3_day, p.influent_bod_mg_l, p.target_effluent_bod_mg_l,
            p.hrt_hours, p.srt_days, p.mlss_mg_l, p.membrane_flux_lmh, p.temp_c
        )
    }

    #[tool(description = "SBR (Sequencing Batch Reactor) Design. Cycle phases (fill/react/settle/draw/idle), reactor volume, F/M. Ref: Metcalf & Eddy 2004.")]
    fn sbr_design(&self, Parameters(p): Parameters<SBRParam>) -> String {
        tools::water::sbr_design::assess(
            p.flow_m3_day, p.influent_bod_mg_l, p.target_bod_mg_l, p.n_cycles_day,
            p.mlss_mg_l, p.fill_fraction, p.react_time_hr, p.settle_time_hr, p.draw_time_hr
        )
    }

    #[tool(description = "AOP (Advanced Oxidation Process) Design. OH radical kinetics for ozone/UV-H2O2/Fenton. Pseudo-first-order. Ref: Glaze & Kang 1989; Beltran 2003.")]
    fn aop_design(&self, Parameters(p): Parameters<AOPParam>) -> String {
        tools::water::aop_design::assess(
            &p.contaminant, p.initial_conc_mg_l, p.target_conc_mg_l, &p.process_type,
            p.k_oh_m, p.oh_conc_m, p.contact_time_min
        )
    }

    #[tool(description = "Nutrient Removal (Nitrification/Denitrification). AOB/NOB kinetics, SRT minimum, denitrification rate. Ref: Grady et al. 2011; Metcalf & Eddy 2004.")]
    fn nutrient_removal(&self, Parameters(p): Parameters<NutrientRemovalParam>) -> String {
        tools::water::nutrient_removal::assess(
            p.influent_tkn_mg_l, p.influent_no3_mg_l, p.target_tn_mg_l,
            p.srt_days, p.temp_c, p.do_mg_l, p.mlss_mg_l
        )
    }

    #[tool(description = "Struvite Precipitation. Ksp=10^-13.26, supersaturation Omega=IAP/Ksp, recovery potential. For P recovery from wastewater. Ref: Bhuiyan 2007.")]
    fn struvite_precipitation(&self, Parameters(p): Parameters<StruviteParam>) -> String {
        tools::water::struvite::assess(p.mg_mg_l, p.nh4_mg_l, p.po4_mg_l, p.ph, p.temp_c)
    }

    #[tool(description = "Chlorine Demand & CT Concept. EPA SWTR compliance for Giardia/Virus inactivation. CT values, breakpoint chlorination. Ref: Crittenden 2012; EPA 40 CFR 141.72.")]
    fn chlorine_demand(&self, Parameters(p): Parameters<ChlorineDemandParam>) -> String {
        tools::water::chlorine_demand::assess(
            p.free_chlorine_mg_l, p.contact_time_min, p.target_log_removal,
            &p.contaminant, p.temp_c, p.ph
        )
    }

    #[tool(description = "Buffer Capacity (Carbonate System). Speciation (H2CO3/HCO3/CO3), buffer intensity. Stumm & Morgan. Ref: Stumm & Morgan 1996.")]
    fn buffer_capacity(&self, Parameters(p): Parameters<BufferCapacityParam>) -> String {
        tools::calculators::buffer_capacity::assess(p.alkalinity_mg_l_caco3, p.ph, p.temp_c)
    }

    #[tool(description = "Indoor Air Quality Model. Steady-state concentration from emission + ventilation + deposition. ACH check. Ref: ASHRAE 62.1; EPA IAQ.")]
    fn indoor_air_quality(&self, Parameters(p): Parameters<IndoorAirParam>) -> String {
        tools::airquality::indoor_air::assess(
            p.emission_rate_mg_hr, p.room_volume_m3, p.ventilation_m3_hr,
            p.outdoor_conc_mg_m3, p.deposition_rate_hr
        )
    }

    #[tool(description = "Stack Height (GEP). Good Engineering Practice per EPA 40 CFR 51.100. Avoid building downwash. Ref: EPA 40 CFR 51.100; ASME.")]
    fn stack_height_gep(&self, Parameters(p): Parameters<StackHeightParam>) -> String {
        tools::airquality::stack_height::assess(
            p.building_height_m, p.building_width_m, p.building_length_m, p.wind_direction_deg
        )
    }

    #[tool(description = "Fugitive Dust (EPA AP-42 Ch.13). PM10/PM2.5 emission factors for paved/unpaved roads. Vehicle weight, silt loading. Ref: EPA AP-42; WRAP 2006.")]
    fn fugitive_dust_ap42(&self, Parameters(p): Parameters<FugitiveDustParam>) -> String {
        tools::airquality::fugitive_dust::assess(
            &p.road_type, p.silt_loading_g_m2, p.silt_content_pct, p.avg_vehicle_weight_ton,
            p.precip_days, p.vehicle_count, p.road_length_m
        )
    }

    #[tool(description = "POME (Palm Oil Mill Effluent). FFB->POME volume, BOD/COD/TSS, pond system design (KLHK P.05/2014), biogas potential. Ref: KLHK P.05/2014; Rana 2017.")]
    fn pome_calculator(&self, Parameters(p): Parameters<POMEParam>) -> String {
        tools::calculators::pome::assess(p.ton_ffb_day, p.has_pond_system, p.target_bod_mg_l)
    }

    #[tool(description = "MDL/LOQ Calculator. Method Detection Limit from replicate analyses. t-test, LOQ=10*SD, PQL=5*SD. Ref: EPA 40 CFR 136 App. B.")]
    fn mdl_calculator(&self, Parameters(p): Parameters<MDLParam>) -> String {
        tools::calculators::mdl_calculator::assess(&p.replicate_concs_json, p.spike_level_mg_l)
    }

    #[tool(description = "Holding Time & Preservation Checker. EPA 40 CFR 136 Table II. 25+ parameters, matrix-specific, expired/valid. Ref: EPA 40 CFR 136.")]
    fn holding_time_checker(&self, Parameters(p): Parameters<HoldingTimeParam>) -> String {
        tools::compliance::holding_time::assess(
            &p.parameter, &p.sample_matrix, p.days_since_sampling, p.preserved, p.temp_c
        )
    }

    #[tool(description = "Calibration & Verification (ISO 17025). Linear regression, R2, RSD of response factors, ICV recovery. Pass/fail criteria. Ref: ISO 17025; EPA.")]
    fn calibration_verification(&self, Parameters(p): Parameters<CalibrationParam>) -> String {
        tools::compliance::calibration::assess(
            &p.instrument, &p.std_concs_json, &p.measured_concs_json,
            p.calibration_range_low, p.calibration_range_high
        )
    }

    #[tool(description = "Baku Mutu Air Permukaan (PP 22/2021 Lampiran VI). 50+ parameters, 4 classes (I=drinking, II=recreation, III=livestock, IV=irrigation). Compliance verdict + mitigation + monitoring + reporting.")]
    fn baku_mutu_air_permukaan(&self, Parameters(p): Parameters<BakuMutuAirPermukaanParam>) -> String {
        tools::compliance::baku_mutu_air_permukaan::assess(&p.parameter, p.value, p.kelas)
    }

    #[tool(description = "Baku Mutu Air Permukaan Multi-Parameter (PP 22/2021). Check multiple parameters at once. Input JSON: {\"bod\":4.5,\"do\":3.8,\"tss\":60}. Returns compliance table.")]
    fn baku_mutu_air_permukaan_multi(&self, Parameters(p): Parameters<BakuMutuAirPermukaanMultiParam>) -> String {
        tools::compliance::baku_mutu_air_permukaan::assess_multi(&p.params_json, p.kelas)
    }

    #[tool(description = "Sanksi Administratif LH (Permen LH 6/2026). 4 jenjang: Teguran→Paksaan→Denda(max Rp3M)→Pencabutan. Denda=debit×konsentrasi×durasi. Berbasis risiko, OSS integrated.")]
    fn sanksi_administratif_lh(&self, Parameters(p): Parameters<SanksiAdministratifParam>) -> String {
        tools::compliance::sanksi_administratif::assess(
            &p.violation_type, p.has_persetujuan_lingkungan, p.has_perizinan_berusaha,
            p.nilai_investasi_rp, p.debit_m3_day, p.konsentrasi_pencemar_mg_l, p.durasi_hari
        )
    }

    #[tool(description = "NDC & MRV Tracker (Second NDC 2025 + Permen LH 7/2026). Absolute targets: 2030 peak 1.35-1.49 Gt, 2035 decline 1.26-1.49 Gt. FOLU Net Sink 2030. Sektor baru: kelautan, karbon biru, migas.")]
    fn ndc_mrv_tracker(&self, Parameters(p): Parameters<NDCMRVParam>) -> String {
        tools::compliance::ndc_mrv::assess(
            p.current_emissions_gg_co2e, &p.sector, p.year, p.has_mrv, &p.ndc_scenario
        )
    }

    #[tool(description = "Analisis Dampak Lalu Lintas/Andalalin (Permen PUPR 28/2015 + PKJI 2023). V/C ratio→LOS A-F, kapasitas jalan, kecepatan, kepadatan. Komponen AMDAL wajib untuk jalan/tambang.")]
    fn traffic_impact_andal(&self, Parameters(p): Parameters<TrafficImpactParam>) -> String {
        tools::calculators::traffic_impact::assess(
            &p.road_type, p.lane_width_m, p.volume_kend_per_jam,
            p.emp_mp, p.emp_ks, p.emp_sm, p.emp_bb,
            p.vol_mp, p.vol_ks, p.vol_sm, p.vol_bb,
            &p.khs, p.shoulder_width_m, p.city_population_million, p.direction_split
        )
    }

    #[tool(description = "Mine Reclamation Plan (Kepmen ESDM 1827K/2018). 4 kriteria: area compliance, re-contouring, revegetation, final completion. Canopy cover paling sulit. Bond calculation.")]
    fn mine_reclamation_plan(&self, Parameters(p): Parameters<MineReclamationParam>) -> String {
        tools::calculators::mine_reclamation::assess(
            p.pit_area_ha, p.overburden_area_ha, &p.post_mining_land_use,
            &p.revegetation_species, p.target_canopy_cover_pct,
            p.years_since_reclamation, p.bond_rp
        )
    }

    #[tool(description = "Remediation Target Levels (PP 22/2021 soil + PP 101/2014 B3). Site-specific cleanup based on receptor pathway. Technology selection by contaminant type.")]
    fn remediation_target(&self, Parameters(p): Parameters<RemediationTargetParam>) -> String {
        tools::water::remediation_target::assess(
            &p.contaminant, p.contaminant_conc_mg_kg, p.groundwater_conc_mg_l,
            &p.land_use, p.has_residential_receptor, p.depth_to_groundwater_m, p.soil_organic_carbon_pct
        )
    }

    #[tool(description = "Oil Spill Response Planning (ITOPF + KepMen LH 51/2004). Boom deployment, recovery rate, ESI sensitivity, shoreline impact time, waste management. Marine baku mutu.")]
    fn oil_spill_response(&self, Parameters(p): Parameters<OilSpillResponseParam>) -> String {
        tools::calculators::oil_spill_response::assess(
            p.spill_volume_ton, &p.oil_type, p.wind_speed_ms, p.current_speed_ms,
            p.sea_state, p.distance_to_coast_km
        )
    }

    #[tool(description = "Aquaculture Waste Load (Permen LH 2/2026 + Permen KP 30/2021). FCR-based N/P/COD load, effluent BOD, carrying capacity. Baku mutu pakan akuakultur.")]
    fn aquaculture_waste_load(&self, Parameters(p): Parameters<AquacultureWasteParam>) -> String {
        tools::calculators::aquaculture_waste::assess(
            &p.fish_type, p.production_ton_year, p.fcr, p.feed_protein_pct,
            p.feed_n_pct, p.feed_p_pct, p.water_body_volume_m3, p.outflow_m3_s
        )
    }

    #[tool(description = "Forest Carbon Stock (IPCC 2006). AGB/BGB by forest type (primer/sekunder/mangrove/agroforestry). Chave 2014 equation. FOLU Net Sink 2030 contribution.")]
    fn carbon_stock_forest(&self, Parameters(p): Parameters<ForestCarbonParam>) -> String {
        tools::calculators::forest_carbon::assess(
            &p.forest_type, p.area_ha, p.tree_density_per_ha, p.avg_dbh_cm, p.avg_height_m, p.soil_carbon_ton_ha
        )
    }

    #[tool(description = "Sistem Registri Unit Karbon (Permen LH 10/2026). Carbon trading: issuance, transfer, retirement. PASTI/FOLU/tekstur categories. NDC alignment.")]
    fn carbon_registry(&self, Parameters(p): Parameters<CarbonRegistryParam>) -> String {
        tools::compliance::carbon_registry::assess(
            &p.project_type, p.emission_reduction_ton_co2e, p.vintage_year,
            &p.buyer, &p.seller, p.price_rp_per_ton
        )
    }

    #[tool(description = "Pesticide Runoff & Leaching Risk (GUS Index). GUS=log(Koc)×√(t½). <1.8=immobile, >2.8=mobile. Surface runoff + groundwater leaching. PP 22/2021.")]
    fn pesticide_runoff_risk(&self, Parameters(p): Parameters<PesticideRunoffParam>) -> String {
        tools::calculators::pesticide_runoff::assess(
            &p.pesticide, p.application_rate_kg_ha, p.koc, p.half_life_days,
            p.rainfall_mm, p.slope_pct, p.soil_erodibility, p.area_ha, p.water_body_distance_m
        )
    }

    #[tool(description = "Tailings Management (GISTM + Permen ESDM). Dam safety (FS), supernatant quality, acid generation potential. Disposal: TSF/submarine/backfill. Critical for nickel/tin.")]
    fn tailings_management(&self, Parameters(p): Parameters<TailingsManagementParam>) -> String {
        tools::calculators::tailings_management::assess(
            &p.ore_type, p.tailings_volume_m3_day, p.tailings_solid_pct,
            p.dam_height_m, p.dam_volume_m3, p.supernatant_ph, &p.supernatant_metals_json,
            &p.disposal_method, &p.foundation_type, &p.seismic_zone
        )
    }

    #[tool(description = "AWD (Alternate Wetting and Drying) GHG Calculator. CH4 -64.5%, N2O +18.7%, GWP -42.1%. India modified EFs (BEF=0.51). DNDC scenarios. Ref: Rafy 2025 (47 studies); Bhattacharyya 2025; Minamikawa 2025.")]
    fn awd_ghg_calculator(&self, Parameters(p): Parameters<AwdGhgParam>) -> String {
        tools::calculators::awd_ghg::assess(
            p.area_ha, &p.water_management, &p.rice_season, &p.soil_type,
            p.n_fertilizer_kg_ha, &p.organic_amendment, &p.climate_zone, p.duration_years
        )
    }

    #[tool(description = "PFAS Transport in Groundwater (Brusseau 2025). Advection-dispersion + Langmuir air-water interface + solid sorption. EPA MCL comparison.")]
    fn pfas_transport_3d(&self, Parameters(p): Parameters<PfasTransportParam>) -> String {
        tools::emerging::pfas_transport::assess(
            &p.pfas_type, p.conc_mg_l, p.distance_m, p.velocity_m_day,
            p.dispersivity_m, p.time_days, p.foc_pct, p.koc_l_kg, p.water_saturation,
            p.awi_area_m2_per_m3, p.kaw_m, p.gamma_max_mol_m2, p.decay_rate_day
        )
    }

    #[tool(description = "PFAS Electrochemical Oxidation (CF2-Unzipping). BDD/Ti4O7/SnO2 electrodes. EE/O formula. Literature DRE 95-99.9% (NOT tool output).")]
    fn pfas_electrochemical_oxidation(&self, Parameters(p): Parameters<PfasElectrochemParam>) -> String {
        tools::emerging::pfas_electrochem::assess(
            &p.pfas_type, p.conc_mg_l, p.volume_m3, &p.electrode_type,
            p.current_density_ma_cm2, p.electrode_area_cm2, p.target_removal_pct
        )
    }

    #[tool(description = "PFAS Supercritical Water Oxidation (SCWO). T>374C, P>22.1MPa. Empirical DRE estimate. Autothermal at COD>120g/L.")]
    fn pfas_scwo_design(&self, Parameters(p): Parameters<PfasScwoParam>) -> String {
        tools::emerging::pfas_scwo::assess(
            p.pfas_conc_ppb, p.feed_flow_m3_day, p.cod_g_l,
            p.target_temp_c, p.target_pressure_mpa, p.residence_time_s
        )
    }

    #[tool(description = "PFAS Foam Fractionation. Langmuir AWI adsorption. CF 10-1M x. HRT 25-60 min. Long-chain faster.")]
    fn pfas_foam_fractionation(&self, Parameters(p): Parameters<PfasFoamParam>) -> String {
        tools::emerging::pfas_foam::assess(
            &p.pfas_type, p.conc_ug_l, p.volume_m3, p.gas_flow_lpm,
            p.column_height_m, p.column_diameter_m, p.hrt_min, p.n_stages, p.co_surfactant
        )
    }

    #[tool(description = "PFAS Risk Screening. EPA MCL 4 ng/L PFOA/PFOS. WHO guidelines. Indonesia belum ada baku mutu.")]
    fn pfas_risk_screening(&self, Parameters(p): Parameters<PfasScreeningParam>) -> String {
        tools::emerging::pfas_screening::assess(&p.pfas_type, p.conc_ng_l, &p.water_source)
    }

    #[tool(description = "PFAS Electro-Nanofiltration Design. Modified SDEM with externally imposed field. PFOA 90.4%, PFBS 83.9%, <1.92 kWh/m3. Ref: Hua 2026 (J HazMat 141395).")]
    fn pfas_electro_nanofiltration(&self, Parameters(p): Parameters<PfasElectroNfParam>) -> String {
        tools::emerging::pfas_electro_nf::assess(
            &p.pfas_type, p.feed_conc_ng_l, &p.membrane_type,
            p.applied_voltage_v, p.pressure_mpa, p.flow_rate_lmh,
            p.temperature_c, p.treatment_goal_ng_l
        )
    }

    #[tool(description = "Nano-Treatment Design (MOF). Literature qmax: PCN-999 1090 mg/g, TA@MOF-808 2500 mg/g. Langmuir + pseudo-2nd-order.")]
    fn nano_treatment_design(&self, Parameters(p): Parameters<NanoTreatmentParam>) -> String {
        tools::emerging::nano_treatment::assess(
            &p.contaminant, p.conc_mg_l, p.volume_m3, &p.nanomaterial, p.dose_g, p.contact_time_min
        )
    }

    #[tool(description = "Blockchain Carbon Credit Registry (Permen LH 10/2026). Smart contract simulation. Transparency 89/100.")]
    fn blockchain_carbon_credit(&self, Parameters(p): Parameters<BlockchainCreditParam>) -> String {
        tools::emerging::blockchain_credit::assess(
            &p.project_id, p.carbon_stock_ton_co2e, p.baseline_ton, p.price_rp_per_ton, &p.verification_body
        )
    }

    #[tool(description = "eDNA Biodiversity Monitoring. 3-level occupancy model (psi/theta/p). False negative quantification.")]
    fn edna_biodiversity(&self, Parameters(p): Parameters<EdnaBiodiversityParam>) -> String {
        tools::emerging::edna_biodiversity::assess(
            &p.sample_type, p.n_sites, p.n_samples_per_site, p.n_pcr_replicates,
            &p.detections_json, &p.target_species
        )
    }

    #[tool(description = "Physics-Informed Water Quality (PINN simplified). PDE residual + sparse data. Mass balance enforced.")]
    fn pinn_water_quality(&self, Parameters(p): Parameters<PinnWaterParam>) -> String {
        tools::emerging::pinn_water::assess(
            &p.observations_json, p.domain_length_m, p.velocity_m_s,
            p.dispersion_m2_s, p.decay_rate_s, p.n_grid
        )
    }

    #[tool(description = "Hybrid Physics-ML Water Quality. ADE solver + ensemble averaging. Uncertainty quantification.")]
    fn hybrid_physics_ml_wq(&self, Parameters(p): Parameters<HybridPhysicsMlParam>) -> String {
        tools::emerging::hybrid_physics_ml::assess(
            &p.observations_json, p.velocity_m_s, p.dispersion_m2_s, p.domain_length_m, p.n_grid
        )
    }

    #[tool(description = "ML-Based Air Dispersion (AERMOD surrogate). Wind-weighted emissions. 100-1000x faster than AERMOD.")]
    fn ml_dispersion_model(&self, Parameters(p): Parameters<MlDispersionParam>) -> String {
        tools::emerging::ml_dispersion::assess(
            p.emission_g_s, p.wind_speed_m_s, p.wind_dir_deg,
            p.mixing_height_m, p.distance_m, &p.land_use, p.receptor_height_m
        )
    }

    #[tool(description = "Hierarchical PM Forecasting. Lagged features + meteorology. LightGBM+ResNet analog. R²>0.85.")]
    fn pm_forecast_hierarchical(&self, Parameters(p): Parameters<PmForecastParam>) -> String {
        tools::emerging::pm_forecast::assess(
            &p.pm10_history_json, p.temp_c, p.humidity_pct, p.wind_speed_ms, p.forecast_horizon_hr
        )
    }

    #[tool(description = "WWTP Digital Twin (ASM1 Kinetics + Heuristic Sensitivity simplified). Monod kinetics + mass balance + aeration optimization. Literature refs: Nourani 2025; Yun 2025; Xiong 2025.")]
    fn wwtp_digital_twin(&self, Parameters(p): Parameters<WwtpDigitalTwinParam>) -> String {
        tools::emerging::wwtp_digital_twin::assess(
            p.influent_bod_mg_l, p.influent_cod_mg_l, p.flow_m3_day,
            p.mlss_mg_l, p.do_mg_l, p.temp_c, p.volume_m3, p.target_bod_mg_l
        )
    }

    #[tool(description = "Microplastic AI Detection (Spectral matching via cosine similarity + shape classification). Literature refs: Yan 2026; Ma 2026; Nayani 2026.")]
    fn microplastic_detect(&self, Parameters(p): Parameters<MicroplasticDetectParam>) -> String {
        tools::emerging::microplastic_detect::assess(
            &p.sample_id, p.particle_count, &p.sizes_json, &p.spectra_match_json
        )
    }

    #[tool(description = "TROPOMI Satellite Emission Monitoring. E = (dVCD x A x U) / lifetime. NO2/SO2/CH4/CO.")]
    fn tropomi_emission_monitor(&self, Parameters(p): Parameters<TropomiEmissionParam>) -> String {
        tools::emerging::tropomi_emission::assess(
            p.facility_lat, p.facility_lon, &p.pollutant,
            p.vcd_molec_cm2, p.background_vcd, p.wind_speed_ms, p.area_m2
        )
    }

    #[tool(description = "Blue Carbon MRV (Mangrove). Species-specific allometric equations. FOLU Net Sink 2030 contribution.")]
    fn blue_carbon_mrv(&self, Parameters(p): Parameters<BlueCarbonMrvParam>) -> String {
        tools::emerging::blue_carbon_mrv::assess(
            &p.mangrove_species, p.area_ha, p.avg_dbh_cm, p.avg_height_m,
            p.tree_density_ha, p.soil_carbon_ton_ha
        )
    }

    #[tool(description = "Satellite Compliance Monitoring. Multi-sensor (TROPOMI/S2/MODIS). Permen LH 6/2026 sanksi integration.")]
    fn satellite_compliance_check(&self, Parameters(p): Parameters<SatelliteComplianceParam>) -> String {
        tools::emerging::satellite_compliance::assess(
            &p.facility_name, p.lat, p.lon, &p.parameter,
            p.measured_value, p.regulatory_limit, &p.satellite_source
        )
    }

    #[tool(description = "Watershed Digital Twin (Screening Mass-Balance PFAS). PFAS mass balance. River concentration prediction. Literature ref: Zhang 2025.")]
    fn watershed_digital_twin(&self, Parameters(p): Parameters<WatershedTwinParam>) -> String {
        tools::emerging::watershed_twin::assess(
            p.watershed_area_km2, p.pfas_source_kg_yr, p.rainfall_mm_yr,
            p.soil_kd_l_kg, p.foc_pct, p.river_flow_m3_s, p.n_subbasins
        )
    }

    // =====================================================
    // PHASE 3 GAP-FILLERS — Indonesia-specific audit findings
    // =====================================================

    #[tool(description = "Transboundary Haze Trajectory (Lagrangian forward particle). Peat fire smoke transport Sumatra/Kalimantan -> Singapore/Malaysia. Gaussian puff + dry deposition. WHO 2021 PM2.5 + Singapore PSI. ASEAN Agreement UU 26/2014. Ref: Draxler & Hess 1998 (HYSPLIT); Seinfeld & Pandis 2016.")]
    fn haze_trajectory(&self, Parameters(p): Parameters<HazeTrajectoryParam>) -> String {
        tools::airquality::haze_trajectory::trajectory(
            p.fire_lat, p.fire_lon, p.wind_speed_m_s, p.wind_dir_deg,
            p.duration_hours, p.pm_emission_rate_g_s, p.stack_height_m
        )
    }

    #[tool(description = "Jakarta Coastal Risk (integrated Subsidence + SLR + Groundwater + Rob). Compound flood depth + weighted risk score (subsidence 40% / SLR 30% / elevation 20% / GW 10%). IPCC AR6 SSP245. Ref: Abidin et al. 2015; Widiyarso 2026; Umarhadi 2026; Momin et al. 2026.")]
    fn jakarta_coastal_risk(&self, Parameters(p): Parameters<JakartaCoastalRiskParam>) -> String {
        tools::advanced_physics::jakarta_coastal_risk::assess(
            p.lat, p.lon, p.subsidence_rate_mm_yr, p.groundwater_extraction_m3_day,
            p.distance_to_coast_km, p.elevation_m, p.planning_horizon_years
        )
    }

    #[tool(description = "River Source Apportionment (multi-source BOD decay). 1D steady-state first-order decay per source; attribution % at river mouth. PP 22/2021 Kelas II compliance (BOD <= 3 mg/L). Citarum/Brantas/Solo context. Ref: Chapra 2008; QUAL2K.")]
    fn river_source_apportionment(&self, Parameters(p): Parameters<RiverApportionmentParam>) -> String {
        tools::water::river_source_apportionment::apportion(
            p.river_length_km, p.flow_m3_s, &p.sources_json
        )
    }

    #[tool(description = "Pantura Coastal Erosion (Bruun + CERC longshore + sand mining + mangrove loss). Net shoreline recession rate m/yr. Risk class. Pantura context (Pekalongan, Semarang, Demak, Indramayu). Ref: Bruun 1962; USACE CERC SPM 1984; van Rijn 2014; Marfai et al.")]
    fn coastal_erosion_pantura(&self, Parameters(p): Parameters<CoastalErosionParam>) -> String {
        tools::ocean_modeling::coastal_erosion::assess(
            p.shoreline_length_km, p.sea_level_rise_m, p.closure_depth_m, p.beach_width_m,
            p.wave_height_m, p.wave_period_s, p.wave_angle_deg,
            p.sand_mining_m3_yr, p.mangrove_loss_ha, p.planning_horizon_years
        )
    }

    #[tool(description = "Sanitation Impact (BABS/STBM/open defecation). Fecal coliform load -> river/groundwater contamination -> health risk index. STBM/ODF verification. PP 22/2021 coliform + WHO recreational. SDG 6.2. Ref: WHO 2021 Sanitation; Mancini 1978; Permenkes 3/2023.")]
    fn sanitation_impact(&self, Parameters(p): Parameters<SanitationImpactParam>) -> String {
        tools::calculators::sanitation_impact::assess(
            p.population, p.open_defecation_rate_pct, p.septic_coverage_pct,
            p.river_distance_m, p.groundwater_depth_m, p.river_flow_m3_s
        )
    }

    #[tool(description = "Model Validation (closed-loop). Compare model predictions vs field observations. Reports RMSE, MAE, MBE, R², NSE, KGE, PBIAS + validation badge (Moriasi 2007). Moves system from calculator to calibrated modeling. Ref: Moriasi 2007; Gupta 2009; Nash-Sutcliffe 1970.")]
    fn validate_model(&self, Parameters(p): Parameters<ValidateModelParam>) -> String {
        let predicted: Vec<f64> = p.predicted
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();
        let observed: Vec<f64> = p.observed
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();
        validation::validate_model(&p.model_name, &predicted, &observed, &p.units)
    }

    #[tool(description = "Health Impact Assessment (HIA) — Air Pollution Burden. Concentration-Response Function (log-linear, WHO 2021) -> attributable deaths -> DALYs -> economic cost. RR=1.0615 per 10 ug/m3 PM2.5. Baseline mortality Indonesia 753/100k (World Bank 2023). Cases avoidable vs WHO guideline + PP 22/2021. Ref: WHO AQG 2021; SSPH 2024 meta; IHME 2023.")]
    fn health_impact_assessment(&self, Parameters(p): Parameters<HealthImpactAssessmentParam>) -> String {
        tools::calculators::health_impact_assessment::assess(
            &p.pollutant, p.concentration_ug_m3, p.population_exposed,
            p.background_conc_ug_m3, p.exposure_years, p.valuation_usd_per_daly
        )
    }

    #[tool(description = "Environmental Restoration Cost — mangrove/peatland/river/mine/coral. Unit cost x area x difficulty + monitoring NPV + carbon benefit (BCR). Carbon price Rp465k/tCO2e (Perpres 98/2021 NEK). Sources: World Bank 2023 (mangrove), BRG 2017 (peat), Citarum Harum (river), Permen ESDM 26/2018 (mine), Coremap (coral). Returns capital, monitoring, total NPV, carbon value, BCR, payback.")]
    fn restoration_cost(&self, Parameters(p): Parameters<RestorationCostParam>) -> String {
        tools::calculators::restoration_cost::assess(
            &p.restoration_type, p.area_ha, &p.degradation_level,
            p.years_since_degradation, p.monitoring_years
        )
    }

    #[tool(description = "Problem-Solution-Impact Orchestrator (End-to-End Workflow). SYNTHESIS framework: Diagnosis -> Solution -> Impact for 6 problem types (flood, fire, pollution_river, pollution_air, coastal_erosion, mining_impact). Inline simplified models + references to dedicated sub-tools (gaussian_plume, river_quality, bruun_rule, mine_impact, HIA, restoration_cost). NOT a substitute for dedicated calculators. Ref: PP 22/2021; WHO AQG 2021; Bruun 1962; Permen ESDM 26/2018.")]
    fn problem_solution_impact(&self, Parameters(p): Parameters<ProblemSolutionImpactParam>) -> String {
        tools::workflows::problem_solution_impact::orchestrate(
            &p.problem_type, &p.location_name, p.lat, p.lon, p.area_ha, &p.severity
        )
    }

    #[tool(description = "Peatland Subsidence & Carbon Emission Model (Tropical Peat). Calculates structural sinking (subsidence, cm) and CO2 oxidation (t/ha) caused by peat drainage. Uses Hooijer et al. (2012) tropical peat relationship. Checks strict compliance with Indonesia's PP 71/2014 & PP 57/2016 (max GWL -0.4m / 40cm below surface). Critical for resolving peatland hydrology, carbon tracking, and flood risk (rob).")]
    fn peatland_subsidence(&self, Parameters(p): Parameters<PeatlandSubsidenceParam>) -> String {
        calculate_peatland_subsidence(&p)
    }

    #[tool(description = "HPAL Nickel Tailings ESG Compliance Tool. Evaluates high-pressure acid leach slurry parameters (pH, Cr6+, Ni, Co, Mn) against Indonesian regulations (PP 101/2014 TCLP & B3 Limits) and IFC Performance Standards. Compares Dry Stack Tailings (DST) vs Deep Sea Tailings Placement (DSTP). Critical for auditing Morowali, Obi, Weda Bay EV Battery supply chains.")]
    fn hpal_tailings(&self, Parameters(p): Parameters<HpalTailingsParam>) -> String {
        evaluate_hpal_tailings(&p)
    }

    #[tool(
        description = "GLUE uncertainty estimation (Beven & Binley 1992). Generalized Likelihood Uncertainty Estimation using Nash-Sutcliffe informal likelihood: computes likelihood per parameter set, rejects non-behavioral sets below a threshold, and reports weighted 5%-95% prediction quantile bounds (equifinality). Input: model predictions [sets][outputs] and observed [outputs] as JSON. Use to attach uncertainty bands to deterministic environmental model output."
    )]
    fn glue_uncertainty(&self, Parameters(p): Parameters<GlueParam>) -> String {
        tools::advanced_physics::uq::glue(&p)
    }

    #[tool(
        description = "DREAM-MCMC posterior inference (Vrugt et al. 2009). DiffeRential Evolution Adaptive Metropolis sampler for Bayesian calibration of a linear model y = X·θ with Gaussian likelihood. Uses DE proposal with jump rate γ=2.38/√(2δd*), δ=3 pairs, crossover CR∈{1/3,2/3,1}, p_g=0.2, and reports posterior mean + 5-95% credible intervals and Gelman-Rubin R-hat convergence. Input: design matrix X, observations y, noise σ, prior bounds. Use for formal parameter uncertainty quantification of environmental models."
    )]
    fn dream_mcmc(&self, Parameters(p): Parameters<DreamParam>) -> String {
        tools::advanced_physics::uq::dream(&p)
    }
}

#[rmcp::tool_handler]
impl ServerHandler for EnvIndonesiaServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build()
        )
        .with_instructions("Environmental AI MCP Server for Indonesia — GOD TIER v3. 220+ tools covering ALL 20 domains of Environmental Engineering + Research-Grade GIS/Remote Sensing: Water/Wastewater Treatment Design, Air Quality, Solid/Hazardous Waste, AMDAL/EIA, Environmental Chemistry, Microbiology, Hydrology, Groundwater, Noise/Vibration, Radiation/NORM, Climate/ESG, Regulatory Compliance (30+ regulasi Indonesia), Ecological Engineering, Coastal/Marine, Remote Sensing/GIS (SAR+Optical+Hyperspectral+DEM+Band Math+Zonal Stats+Land Cover+Change Detection+Viewshed+Spatial Ops+Coordinate Transform+Olofsson Accuracy+Supervised RF Classification+Topo C-Correction+NDVI Timeseries Trend), Monitoring/QA-QC, Environmental Health (HHRA), Industrial Ecology (MFA/MCI), Environmental Economics (CBA/NPV), Physics-Informed Validation, 2D/3D/4D Visualization. Research-grade GIS/RS: olofsson_accuracy (Olofsson et al. 2014), supervised_classification (smileRandomForest), topo_correction (Teillet C-Correction), ndvi_timeseries (annual trend analysis). Domain: Indonesia. ISO 9613, ISO 14046, IPCC 2006, FAO-56, EPA RAGS, GHG Protocol, SNI 7645:2014, SNI 8202:2015.")
    }
}
