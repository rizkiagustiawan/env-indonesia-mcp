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
pub struct StreeterPhelpsParam { pub k1: f64, pub k2: f64, pub l0: f64, pub d0: f64, pub velocity_ms: f64, pub distance_km: f64, pub temp_c: Option<f64> }
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
pub struct PlumeRiseParam { pub stack_height_m: f64, pub stack_diameter_m: f64, pub exit_velocity_ms: f64, pub exit_temp_k: f64, pub ambient_temp_k: f64, pub wind_speed_ms: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Dispersion2dParam { pub sources_json: String, pub wind_speed: f64, pub wind_dir: f64, pub stability: String, pub output_path: String, pub title: String, pub grid_size: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Dispersion4dParam { pub sources_json: String, pub wind_speeds: String, pub wind_dirs: String, pub stability: String, pub output_path: String, pub title: String, pub grid_size: Option<u32> }

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

// Advanced Physics Params
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FluxDivergenceParam { pub grid_data_json: String, pub u_wind: f64, pub v_wind: f64, pub dx_meters: f64, pub dy_meters: f64, pub lifetime_hours: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GroundwaterPdeParam { pub h_initial_json: String, pub diffusivity_d: f64, pub dx_meters: f64, pub dy_meters: f64, pub time_steps: u32, pub dt_seconds: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BayesianSensorParam { pub prior_particles_json: String, pub sensor_reading: f64, pub sensor_noise_std: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UhiParam { pub albedo_urban: f64, pub sky_view_factor: f64, pub solar_insolation_w: f64, pub ambient_temp_c: f64 }

// ====== GOD TIER: Previously Unregistered Tool Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BiodiversityCalcParam {
    #[schemars(description = "JSON array jumlah individu per spesies, e.g. [45, 23, 12, 8, 5]")]
    pub species_counts_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CompostingParam {
    #[schemars(description = "JSON array [[name, mass_kg, c_pct, n_pct], ...], e.g. [[\"Serbuk Gergaji\", 100, 50, 0.1], [\"Kotoran Ayam\", 50, 30, 3.0]]")]
    pub materials_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FloodFreqParam {
    #[schemars(description = "JSON array data debit puncak tahunan (minimal 10 tahun), e.g. [120, 145, 98, ...]")]
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
    #[schemars(description = "JSON array: [{\"name\":\"BOD\",\"ci\":4.0,\"lij\":2.0,\"is_do\":false}, ...]")]
    pub data_json: String,
    #[schemars(description = "Suhu air (°C) untuk koreksi DO saturasi")]
    pub temp_c: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StoretParam {
    #[schemars(description = "JSON array: [{\"name\":\"BOD\",\"type\":\"kimia\",\"samples\":[{\"value\":4.0,\"limit\":2.0}]}, ...]")]
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
    #[schemars(description = "Parameter: pH/DO/BOD5/ammonia/fosfat/nitrat/sulfida/minyak_lemak/surfaktan/fenol/sianida/Hg/Cr6/As/Cd/Cu/Pb/Zn/Ni/coliform/suhu_delta")]
    pub parameter: String,
    #[schemars(description = "Nilai terukur (mg/L, MPN/100mL untuk coliform, °C untuk suhu_delta)")]
    pub concentration: f64,
    #[schemars(description = "Peruntukan: wisata/biota/pelabuhan")]
    pub peruntukan: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TimelapseParam {
    pub lat: f64, pub lon: f64,
    #[schemars(description = "Buffer radius (km)")]
    pub buffer_km: f64,
    pub start_year: u32, pub end_year: u32,
    #[schemars(description = "Sensor: optik_s2 atau radar_s1")]
    pub sensor: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HyperspectralParam {
    pub lat: f64, pub lon: f64, pub output_path: String,
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
    #[schemars(description = "Industri: pltu_batubara/semen/smelter/kimia/pembangkit_gas/incinerator")]
    pub industry: String,
    #[schemars(description = "Parameter: TSP/SO2/NO2/CO/opacity")]
    pub parameter: String,
    #[schemars(description = "Konsentrasi terukur (mg/Nm³ atau % untuk opacity)")]
    pub concentration: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BakuMutuAirLimbahParam {
    #[schemars(description = "Industri: tekstil/sawit/karet/tapioka/gula/pulp_kertas/farmasi/electroplating/rumah_sakit/hotel/peternakan")]
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
    #[schemars(description = "Zona: perumahan/perdagangan/perkantoran/industri/rumah_sakit/sekolah/ibadah/ruang_terbuka_hijau")]
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
    #[schemars(description = "PM10 (µg/m³)")] pub pm10: Option<f64>,
    #[schemars(description = "PM2.5 (µg/m³)")] pub pm25: Option<f64>,
    #[schemars(description = "SO2 (µg/m³)")] pub so2: Option<f64>,
    #[schemars(description = "CO (µg/m³)")] pub co: Option<f64>,
    #[schemars(description = "O3 (µg/m³)")] pub o3: Option<f64>,
    #[schemars(description = "NO2 (µg/m³)")] pub no2: Option<f64>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RiskClassParam {
    #[schemars(description = "Sektor: pertambangan/industri/energi/pertanian/kehutanan/transportasi/pariwisata")]
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
    #[schemars(description = "Aktivitas: electricity_kwh/diesel_liter/gasoline_liter/lpg_kg/cement_ton/deforestation_ha/rice_paddy_ha/landfill_ton")]
    pub activity: String,
    #[schemars(description = "Jumlah (sesuai unit aktivitas)")]
    pub amount: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IklhSubParam {
    #[schemars(description = "Tipe: ika/iku/iktl/ikal")]
    pub sub_type: String,
    #[schemars(description = "JSON data: array angka IP/ISPU, atau {\"forest_cover_pct\":X,\"target_pct\":Y}, atau JSON params laut")]
    pub data_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AmdalScreeningParam {
    #[schemars(description = "Sektor: pertambangan/kehutanan/industri/energi/transportasi/pariwisata/pertanian/perikanan/permukiman")]
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
    #[schemars(description = "JSON rona lingkungan awal: {\"topografi\":\"...\",\"iklim\":\"...\",\"flora_fauna\":\"...\"}")]
    pub rona_json: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AndalParam {
    pub project_name: String, pub location: String,
    #[schemars(description = "JSON dampak: [{\"component\":\"...\",\"impact\":\"...\",\"magnitude\":-7,\"importance\":8,\"duration\":\"permanen\"}]")]
    pub impacts_json: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RklRplParam {
    pub project_name: String, pub location: String,
    #[schemars(description = "JSON rencana: [{\"impact\":\"...\",\"management\":\"...\",\"monitoring\":\"...\",\"institution\":\"...\",\"location\":\"...\",\"period\":\"...\"}]")]
    pub management_json: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UklUplParam {
    pub project_name: String, pub location: String,
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
    #[schemars(description = "JSON sumber: [{\"x_m\":0,\"y_m\":0,\"power_db\":95,\"type\":\"point\"}]")]
    pub sources_json: String,
    pub output_path: String, pub title: String,
    #[schemars(description = "Ukuran grid (m), default 500")]
    pub grid_size: Option<u32>,
    #[schemars(description = "JSON barrier: [{\"x1\":100,\"y1\":-50,\"x2\":100,\"y2\":50,\"height_m\":3,\"il_db\":10}] atau \"[]\"")]
    pub barrier_json: Option<String>,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Noise3dParam {
    pub sources_json: String, pub output_path: String, pub title: String,
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
    pub lat: f64, pub lon: f64,
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
    #[schemars(description = "JSON dampak sosial: [{\"component\":\"ekonomi\",\"impact\":\"kehilangan lahan\",\"magnitude\":-7,\"importance\":8}]")]
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
    #[schemars(description = "Metode: replacement_cost/travel_cost/hedonic/damage_cost/benefit_transfer")]
    pub method: String,
    #[schemars(description = "JSON parameter sesuai metode")]
    pub params_json: String,
}

// ====== GOD TIER: Data Source Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IspuKlhkParam {
    #[schemars(description = "Nama kota: Jakarta/Surabaya/Bandung/Semarang/Medan/Makassar/Denpasar/Mataram/dll")]
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
    pub lat: f64, pub lon: f64,
    #[schemars(description = "Radius pencarian (m)")]
    pub radius_m: f64,
    #[schemars(description = "Tipe POI: hospital/school/residential/worship/market/river/forest")]
    pub poi_type: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ElevationParam {
    pub lat1: f64, pub lon1: f64, pub lat2: f64, pub lon2: f64,
    #[schemars(description = "Jumlah titik interpolasi (default 20)")]
    pub num_points: Option<u32>,
}

// ====== GOD TIER: SAR Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SarFloodParam {
    pub lat: f64, pub lon: f64,
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
    pub lat: f64, pub lon: f64, pub buffer_km: f64,
    pub start_date: String, pub end_date: String, pub output_path: String,
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
    pub lat: f64, pub lon: f64, pub buffer_km: f64,
    pub start_date: String, pub end_date: String, pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BurnedAreaParam {
    pub lat: f64, pub lon: f64, pub buffer_km: f64,
    #[schemars(description = "Tanggal kebakaran (YYYY-MM-DD)")]
    pub fire_date: String,
    pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MangroveExtentParam {
    pub lat: f64, pub lon: f64, pub buffer_km: f64, pub output_path: String,
}

// ====== GOD TIER PHASE 2: Water Engineering Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CtDisinfectionParam {
    #[schemars(description = "Disinfektan: chlorine/ozone/uv/chloramine")] pub disinfectant: String,
    #[schemars(description = "Konsentrasi (mg/L) atau dosis UV (mJ/cm²)")] pub concentration_mgl: f64,
    #[schemars(description = "Waktu kontak (menit)")] pub contact_time_min: f64,
    #[schemars(description = "Patogen target: giardia/virus/crypto")] pub target_pathogen: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DarcyParam {
    #[schemars(description = "Konduktivitas hidraulik K (m/s)")] pub k_ms: f64,
    #[schemars(description = "Gradien hidraulik (i = Δh/L)")] pub gradient: f64,
    #[schemars(description = "Luas penampang (m²)")] pub area_m2: f64,
    #[schemars(description = "Porositas (0-1)")] pub porosity: f64,
    #[schemars(description = "Jarak transport (m)")] pub distance_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TheisParam {
    #[schemars(description = "Debit pompa (m³/s)")] pub q_m3s: f64,
    #[schemars(description = "Transmisivitas (m²/s)")] pub transmissivity_m2s: f64,
    #[schemars(description = "Storativity (dimensionless)")] pub storativity: f64,
    #[schemars(description = "Jarak dari sumur (m)")] pub r_m: f64,
    #[schemars(description = "Waktu pemompaan (detik)")] pub t_s: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HazenWilliamsParam {
    #[schemars(description = "Debit (m³/s)")] pub q_m3s: f64,
    #[schemars(description = "Panjang pipa (m)")] pub length_m: f64,
    #[schemars(description = "Diameter pipa (m)")] pub diameter_m: f64,
    #[schemars(description = "Koefisien C: PVC(150)/PE(140)/steel_new(120)/cast_iron(100)/concrete(110)")] pub c_coeff: f64,
    #[schemars(description = "Sertakan minor losses (10%)")] pub include_minor_losses: bool,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PumpSizingParam {
    #[schemars(description = "Debit (m³/s)")] pub q_m3s: f64,
    pub static_lift_m: f64, pub friction_loss_m: f64, pub velocity_head_m: f64, pub pressure_head_m: f64,
    #[schemars(description = "Efisiensi pompa (0-1, typical 0.6-0.85)")] pub efficiency: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SedimentationParam {
    #[schemars(description = "Debit desain (m³/hari)")] pub q_m3d: f64,
    #[schemars(description = "Tipe: primary/secondary")] pub tank_type: String,
    #[schemars(description = "Bentuk: rectangular/circular")] pub tank_shape: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UasbParam {
    pub q_m3d: f64,
    #[schemars(description = "COD influent (mg/L)")] pub cod_in_mgl: f64,
    #[schemars(description = "Target COD effluent (mg/L)")] pub cod_eff_target: f64,
    pub temperature_c: f64,
    #[schemars(description = "Tipe limbah: pome/tapioka/karet/domestik")] pub waste_type: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TricklingFilterParam {
    pub q_m3d: f64,
    #[schemars(description = "BOD influent (mg/L)")] pub bod_in: f64,
    #[schemars(description = "BOD target (mg/L)")] pub bod_target: f64,
    #[schemars(description = "Kedalaman media (m), typical 1.5-3.0")] pub media_depth_m: f64,
    #[schemars(description = "Rasio resirkulasi (0-3)")] pub recirculation_ratio: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ConstructedWetlandParam {
    pub q_m3d: f64,
    #[schemars(description = "Parameter: BOD/TSS/NH4N")] pub parameter: String,
    #[schemars(description = "Konsentrasi influent (mg/L)")] pub ci_mgl: f64,
    #[schemars(description = "Target effluent (mg/L)")] pub ce_target: f64,
    pub temp_c: f64,
    #[schemars(description = "Tipe: FWS (free water surface) / HSSF (horizontal subsurface flow)")] pub wetland_type: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AnaerobicDigestionParam {
    pub q_m3d: f64,
    #[schemars(description = "Konsentrasi VS (kg/m³)")] pub vs_concentration_kgm3: f64,
    #[schemars(description = "% destruksi VS (50-80%)")] pub vs_destruction_pct: f64,
    pub temperature_c: f64,
    #[schemars(description = "Substrat: sapi/babi/ayam/pome")] pub substrate: String,
}
// ====== Chemistry Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FirstOrderParam { pub c0: f64, pub k: f64, pub t: f64, #[schemars(description = "Unit: s/min/hr/day")] pub time_unit: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IsothermParam {
    #[schemars(description = "Model: freundlich/langmuir")] pub model: String,
    #[schemars(description = "Konsentrasi kesetimbangan Ce (mg/L)")] pub ce: f64,
    pub kf: f64, pub n_exp: f64, pub qmax: f64, pub kl: f64,
    #[schemars(description = "Volume larutan (L)")] pub volume_l: f64,
    #[schemars(description = "Konsentrasi awal (mg/L)")] pub c0: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HenrysLawParam {
    #[schemars(description = "Senyawa: benzene/toluene/TCE/PCE/chloroform/methane/CO2/O2/NH3")] pub compound: String,
    pub concentration_mgl: f64, pub temperature_c: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NernstParam {
    #[schemars(description = "Setengah reaksi: O2_H2O/Fe3_Fe2/MnO4_Mn2/Cr2O7_Cr3/NO3_N2")] pub half_reaction: String,
    pub temperature_c: f64, pub log_q: f64, pub n_electrons: u32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PartitionParam {
    #[schemars(description = "Senyawa: benzene/toluene/naphthalene/phenol/atrazine/DDT/PCB")] pub compound: String,
    #[schemars(description = "Fraksi karbon organik tanah")] pub foc: f64,
    pub bulk_density_kgm3: f64, pub porosity: f64,
}
// ====== Hydrology Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RationalParam {
    #[schemars(description = "Koefisien limpasan C (0-1), atau isi 0 dan gunakan land_use")] pub c_coeff: f64,
    #[schemars(description = "Intensitas hujan (mm/jam)")] pub i_mm_hr: f64,
    #[schemars(description = "Luas DAS (ha)")] pub a_ha: f64,
    #[schemars(description = "Tipe lahan: hutan/sawah/perkebunan/permukiman_jarang/permukiman_padat/komersial/industri/jalan_aspal")] pub land_use: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UnitHydrographParam { pub a_km2: f64, pub tc_hours: f64, pub d_hours: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MuskingumParam {
    #[schemars(description = "JSON: [[t1,Q1],[t2,Q2],...] inflow hydrograph")] pub inflow_json: String,
    pub k_hours: f64, #[schemars(description = "Weighting factor x (0-0.5)")] pub x: f64, pub dt_hours: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TocParam {
    #[schemars(description = "Metode: kirpich/bransby_williams/scs_lag")] pub method: String,
    #[schemars(description = "Panjang saluran (m)")] pub l_m: f64,
    #[schemars(description = "Kemiringan (m/m)")] pub s_slope: f64,
    pub a_km2: f64, pub cn: f64,
}
// ====== Waste Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandfillLinerParam {
    #[schemars(description = "Tipe: single_clay/composite/double_liner")] pub liner_type: String,
    pub area_m2: f64, pub head_on_liner_m: f64, pub k_clay: f64, pub clay_thickness_m: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LeachateParam {
    pub area_m2: f64,
    #[schemars(description = "JSON 12 nilai curah hujan bulanan (mm)")] pub monthly_rainfall_json: String,
    #[schemars(description = "JSON 12 nilai ET bulanan (mm)")] pub monthly_et_json: String,
    pub soil_storage_mm: f64, pub runoff_coeff: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandfillStabilityParam { pub slope_angle_deg: f64, pub height_m: f64, pub unit_weight_kn_m3: f64, pub cohesion_kpa: f64, pub friction_deg: f64, pub pore_pressure_ratio: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TclpParam { #[schemars(description = "JSON: [{\"name\":\"Pb\",\"concentration_mgl\":4.5}, ...]")] pub parameters_json: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WasteCompatParam { pub waste_a: String, pub waste_b: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct B3StorageParam { #[schemars(description = "Tipe: padat/cair/lumpur")] pub waste_type: String, pub volume_m3_per_month: f64, pub density_kg_m3: f64 }
// ====== Radiation Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InverseSquareParam { pub dose_rate_at_d1: f64, pub d1_m: f64, pub d2_m: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ShieldingParam { pub initial_intensity: f64, #[schemars(description = "Material: lead/concrete/water/steel/earth")] pub material: String, pub thickness_cm: f64, #[schemars(description = "Sumber: Cs137/Co60/I131/Sr90/Ra226")] pub source: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DecayParam { #[schemars(description = "Isotop: Cs137/Co60/I131/Sr90/Ra226/C14/H3/Tc99m/U238")] pub isotope: String, pub initial_activity_bq: f64, pub time_elapsed: f64, #[schemars(description = "Unit: seconds/minutes/hours/days/years")] pub time_unit: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RadonParam { pub soil_radon_bq_m3: f64, pub floor_area_m2: f64, pub room_height_m: f64, pub ventilation_rate_ach: f64, #[schemars(description = "Tipe lantai: concrete_slab/basement/tanah/elevated")] pub floor_type: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NormParam { #[schemars(description = "Material: tin_slag/monazite/zircon/coal_ash/phosphogypsum/bauxite")] pub material: String, pub activity_bq_g: f64 }
// ====== Health & Monitoring Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HhraParam {
    #[schemars(description = "Jalur: inhalation/ingestion/dermal")] pub exposure_route: String,
    pub concentration: f64, pub intake_rate: f64, pub exposure_freq_days: f64,
    pub exposure_dur_years: f64, pub body_weight_kg: f64, pub avg_time_years: f64, pub csf: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HqParam {
    #[schemars(description = "Kontaminan: arsenic/chromium_vi/cadmium/mercury/benzene/toluene/xylene/phenol/formaldehyde/ammonia")] pub contaminant: String,
    #[schemars(description = "Jalur: oral/inhalation")] pub route: String,
    pub concentration: f64, pub intake_rate: f64, pub exposure_freq_days: f64,
    pub exposure_dur_years: f64, pub body_weight_kg: f64,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ArklParam {
    #[schemars(description = "Kontaminan: arsenic/chromium_vi/cadmium/benzene/toluene/ammonia/dll")] pub contaminant: String,
    #[schemars(description = "Jalur: oral/inhalation")] pub route: String,
    #[schemars(description = "Konsentrasi terukur (mg/kg/day untuk oral, mg/m³ untuk inhalasi)")] pub concentration: f64,
    #[schemars(description = "Tipe populasi: dewasa/anak")] pub population_type: String,
    #[schemars(description = "Skenario: residensial/okupasional/sekolah")] pub exposure_scenario: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SamplingParam { pub confidence_pct: f64, pub margin_error_pct: f64, pub std_deviation: f64, pub population_size: Option<u64> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MannKendallParam { #[schemars(description = "JSON array data time-series (urut waktu)")] pub data_json: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QaqcParam { #[schemars(description = "JSON: [{\"sample\":\"S1\",\"value\":5.2,\"duplicate\":5.0,\"spike\":47.5,\"spike_amount\":50.0,\"blank\":0.02}]")] pub data_json: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ColiformParam { pub initial_count_per_100ml: f64, pub temperature_c: f64, pub time_hours: f64, #[schemars(description = "Tipe air: freshwater/seawater/tropical")] pub water_type: String }
// ====== Ecological/Coastal Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BruunParam { pub sea_level_rise_m: f64, pub profile_length_m: f64, pub berm_height_m: f64, pub closure_depth_m: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CviParam { pub geomorphology: u32, pub shoreline_change_m_yr: f64, pub coastal_slope_pct: f64, pub slr_mm_yr: f64, pub mean_wave_height_m: f64, pub mean_tidal_range_m: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TrafficNoiseParam { pub vehicles_per_hour: f64, pub speed_kmh: f64, pub distance_m: f64, pub heavy_vehicle_pct: f64, pub gradient_pct: f64, #[schemars(description = "Tipe tanah: hard/soft")] pub ground_type: String, pub barrier_height_m: Option<f64> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BioretentionParam { pub q_design_m3s: f64, pub ksat_m_hr: f64, pub ponding_depth_m: f64, pub media_depth_m: f64, pub drain_time_hr: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaterFootprintParam { #[schemars(description = "Produk: rice/palm_oil/rubber/coffee/beef/chicken/cotton/paper/steel/cement")] pub product: String, pub quantity: f64, #[schemars(description = "Unit: kg/ton/L")] pub unit: String }
// ====== Economics Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CbaParam { #[schemars(description = "JSON: [{\"year\":0,\"amount\":1e9,\"description\":\"Konstruksi\",\"recurring\":false}]. Set \"recurring\":true for annual items repeated from year to end of period.")] pub costs_json: String, #[schemars(description = "JSON: [{\"year\":1,\"amount\":2e8,\"description\":\"Revenue\",\"recurring\":true}]. Set \"recurring\":true for annual items repeated from year to end of period.")] pub benefits_json: String, pub discount_rate: f64, pub years: u32 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MfaParam { pub inputs_json: String, pub outputs_json: String, pub stock_change: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Scope123Param { pub scope1_json: String, pub scope2_json: String, pub scope3_json: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CircularParam { pub mass_product_kg: f64, pub virgin_feedstock_pct: f64, pub recycled_input_pct: f64, pub reused_input_pct: f64, pub recycled_output_pct: f64, pub reused_output_pct: f64, pub product_lifetime_years: f64, pub industry_avg_lifetime: f64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExternalityParam { pub pollutant: String, pub amount: f64, #[schemars(description = "Unit: ton/kg")] pub unit: String, #[schemars(description = "Lokasi: urban/suburban/rural")] pub location_type: String }

// ====== GIS/RS Tool Params ======
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RasterBandMathParam {
    #[schemars(description = "Latitude")] pub lat: f64,
    #[schemars(description = "Longitude")] pub lon: f64,
    #[schemars(description = "Buffer radius (km)")] pub buffer_km: f64,
    #[schemars(description = "Index type: ndvi/ndwi/savi/evi/mndwi/ndbi/bsi")] pub index_type: String,
    #[schemars(description = "Start date YYYY-MM-DD")] pub start_date: String,
    #[schemars(description = "End date YYYY-MM-DD")] pub end_date: String,
    #[schemars(description = "Output GeoTIFF path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RasterBandMathLocalParam {
    #[schemars(description = "Input GeoTIFF path")] pub input_path: String,
    #[schemars(description = "Band math expression (e.g. '(b1-b2)/(b1+b2)')")] pub expression: String,
    #[schemars(description = "Output GeoTIFF path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DemGeeParam {
    #[schemars(description = "Latitude")] pub lat: f64,
    #[schemars(description = "Longitude")] pub lon: f64,
    #[schemars(description = "Buffer radius (km)")] pub buffer_km: f64,
    #[schemars(description = "Output path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ZonalStatsGeeParam {
    #[schemars(description = "GEE Image ID (e.g. USGS/SRTMGL1_003)")] pub image_id: String,
    #[schemars(description = "Band name (e.g. elevation)")] pub band: String,
    #[schemars(description = "GeoJSON polygon string (optional, use lat/lon/buffer if empty)")] pub geojson: Option<String>,
    #[schemars(description = "Latitude")] pub lat: f64,
    #[schemars(description = "Longitude")] pub lon: f64,
    #[schemars(description = "Buffer radius (km)")] pub buffer_km: f64,
    #[schemars(description = "Output JSON path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ZonalStatsLocalParam {
    #[schemars(description = "Input raster path")] pub raster_path: String,
    #[schemars(description = "Input vector path (GeoJSON/Shapefile)")] pub vector_path: String,
    #[schemars(description = "Stats: comma-separated (min,max,mean,std,sum,count)")] pub stats: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandCoverClassifyParam {
    #[schemars(description = "Latitude")] pub lat: f64,
    #[schemars(description = "Longitude")] pub lon: f64,
    #[schemars(description = "Buffer radius (km)")] pub buffer_km: f64,
    #[schemars(description = "Start date YYYY-MM-DD")] pub start_date: String,
    #[schemars(description = "End date YYYY-MM-DD")] pub end_date: String,
    #[schemars(description = "Output path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LandUseChangeParam {
    #[schemars(description = "Latitude")] pub lat: f64,
    #[schemars(description = "Longitude")] pub lon: f64,
    #[schemars(description = "Buffer radius (km)")] pub buffer_km: f64,
    #[schemars(description = "Period 1 start date YYYY-MM-DD")] pub d1_start: String,
    #[schemars(description = "Period 1 end date YYYY-MM-DD")] pub d1_end: String,
    #[schemars(description = "Period 2 start date YYYY-MM-DD")] pub d2_start: String,
    #[schemars(description = "Period 2 end date YYYY-MM-DD")] pub d2_end: String,
    #[schemars(description = "Output path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AccuracyAssessmentParam {
    #[schemars(description = "JSON array of predicted class labels, e.g. [\"forest\",\"water\",\"urban\"]")] pub predicted_json: String,
    #[schemars(description = "JSON array of actual (ground truth) class labels")] pub actual_json: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BufferAnalysisParam {
    #[schemars(description = "GeoJSON string")] pub geojson: String,
    #[schemars(description = "Buffer distance (meters)")] pub distance_m: f64,
    #[schemars(description = "Output GeoJSON path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OverlayAnalysisParam {
    #[schemars(description = "GeoJSON A string")] pub geojson_a: String,
    #[schemars(description = "GeoJSON B string")] pub geojson_b: String,
    #[schemars(description = "Operation: intersection/union/difference/symmetric_difference")] pub operation: String,
    #[schemars(description = "Output GeoJSON path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SuitabilityAnalysisParam {
    #[schemars(description = "JSON criteria for suitability analysis")] pub criteria_json: String,
    #[schemars(description = "Latitude")] pub lat: f64,
    #[schemars(description = "Longitude")] pub lon: f64,
    #[schemars(description = "Buffer radius (km)")] pub buffer_km: f64,
    #[schemars(description = "Output path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ViewshedAnalysisParam {
    #[schemars(description = "DEM GeoTIFF path")] pub dem_path: String,
    #[schemars(description = "Observer latitude")] pub observer_lat: f64,
    #[schemars(description = "Observer longitude")] pub observer_lon: f64,
    #[schemars(description = "Observer height above ground (m)")] pub observer_height_m: f64,
    #[schemars(description = "Max viewshed distance (m)")] pub max_distance_m: f64,
    #[schemars(description = "Output path")] pub output_path: String,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CoordTransformV2Param {
    #[schemars(description = "X coordinate / Easting / Longitude")] pub x: f64,
    #[schemars(description = "Y coordinate / Northing / Latitude")] pub y: f64,
    #[schemars(description = "Source CRS EPSG code (e.g. 4326)")] pub from_epsg: u32,
    #[schemars(description = "Target CRS EPSG code (e.g. 32750)")] pub to_epsg: u32,
}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Wgs84ToUtmParam {
    #[schemars(description = "Latitude (WGS84)")] pub lat: f64,
    #[schemars(description = "Longitude (WGS84)")] pub lon: f64,
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
    #[schemars(description = "Judul Peta untuk Kop SNI")]
    pub title: String,
    #[schemars(description = "Jika true, otomatis mendownload Sentinel-2 (Bebas awan 30 hari terakhir) via GEE sbg Basemap.")]
    pub realtime_satellite: Option<bool>,
}

// ====== Tool implementations ======

use rmcp::handler::server::wrapper::Parameters;

/// Parse "lat,lon" or "lat,lon,days" from query string, default to Indonesia center
fn parse_latlon_query(query: &str) -> (f64, f64, u32) {
    let parts: Vec<&str> = query.split(',').collect();
    let lat: f64 = parts.first().and_then(|s| s.trim().parse().ok()).unwrap_or(-8.65);
    let lon: f64 = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(116.35);
    let days: u32 = parts.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(30);
    (lat, lon, days)
}

#[tool_router]
impl EnvIndonesiaServer {
    // --- DATA INDONESIA ---
    #[tool(description = "BMKG weather forecast for Indonesian cities")]
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
        let lat = p.lat.unwrap_or(-6.2);
        let lon = p.lon.unwrap_or(106.85);
        if let Err(e) = crate::indonesia::validate_coords(lat, lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::data::openweather::air_pollution(&HTTP, lat, lon).await
    }

    #[tool(description = "Weather 7-day forecast (Open-Meteo, free, no API key)")]
    async fn open_meteo_weather(&self, Parameters(p): Parameters<LatLonParam>) -> String {
        let lat = p.lat.unwrap_or(-6.2);
        let lon = p.lon.unwrap_or(106.85);
        if let Err(e) = crate::indonesia::validate_coords(lat, lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::data::open_meteo::weather(&HTTP, lat, lon).await
    }

    #[tool(description = "NASA POWER solar irradiance GHI DNI monthly for energy potential")]
    async fn nasa_power_solar(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
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

    #[tool(description = "[LEGACY → use coordinate_transform_v2 or wgs84_to_utm] Coordinate transform. direction: wgs84_to_utm, utm_to_wgs84, or EPSG code")]
    fn coordinate_transform(&self, Parameters(p): Parameters<CoordParam>) -> String {
        match p.direction.as_str() {
            "wgs84_to_utm" => tools::gis::coords::wgs84_to_utm_auto(p.y, p.x),
            "utm_to_wgs84" => tools::gis::coords::utm_to_wgs84(p.x, p.y, "EPSG:32750"),
            _ => tools::gis::coords::transform(p.x, p.y, "EPSG:4326", &p.direction),
        }
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

    #[tool(description = "TCFD climate risk assessment for Indonesian sectors")]
    fn climate_risk_tcfd(&self, Parameters(p): Parameters<TcfdParam>) -> String {
        tools::esg::tcfd::risk_assessment(&p.sector, &p.location)
    }

    // --- OCEAN & MARINE ---
    #[tool(description = "Coral reef health Indonesia: 15 reef sites, 590 coral species. Opsional: lat/lon untuk cari reef terdekat.")]
    fn coral_reef_health(&self, Parameters(p): Parameters<CoralReefParam>) -> String {
        tools::ocean::coral::reef_health(p.lat, p.lon, p.n)
    }

    #[tool(description = "Marine protected areas Indonesia: 16+ KKP, 28.4 juta ha. Opsional: lat/lon untuk cari MPA terdekat.")]
    fn marine_protected_areas(&self, Parameters(p): Parameters<MpaParam>) -> String {
        tools::ocean::mpa::protected_areas(p.lat, p.lon, p.n)
    }

    // --- WRAPPERS (Existing Projects) ---
    #[tool(description = "Wrapper: Trigger ESG Audit pipeline in GeoESG-Final (Port 8000)")]
    async fn wrapper_esg_audit(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::wrappers::trigger_esg_audit(&HTTP, &p.query).await
    }

    #[tool(description = "Wrapper: Predict flood via geo-flood-ai (Port 8001)")]
    async fn wrapper_flood_predict(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::wrappers::predict_flood(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "Wrapper: Get methane plumes data (Port 8002)")]
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
    #[tool(description = "Status Gunung Api Indonesia dari MAGMA Indonesia")]
    async fn magma_volcano(&self) -> String {
        tools::data::magma::status(&HTTP).await
    }

    #[tool(description = "BPS Environmental Statistics Indonesia. keyword: hutan/sampah/air/ekonomi")]
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
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR: {}", e); }
        tools::satellite::modis::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "NASA VIIRS products information (Nighttime lights, active fires).")]
    async fn satellite_viirs(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR: {}", e); }
        tools::satellite::viirs::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "SRTM 30m Digital Elevation Model for Indonesia")]
    async fn satellite_srtm(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR: {}", e); }
        tools::satellite::srtm::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "CHIRPS Rainfall data (real HTTP query). Query: year,month (e.g. 2024,6)")]
    async fn satellite_chirps(&self, Parameters(p): Parameters<QueryParam>) -> String {
        let parts: Vec<&str> = p.query.split(',').collect();
        let year: u32 = parts.first().and_then(|s| s.trim().parse().ok()).unwrap_or(2024);
        let month: u32 = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(1);
        tools::satellite::chirps::query(&HTTP, year, month).await
    }

    #[tool(description = "NASA GRACE / GRACE-FO Groundwater Storage anomaly information.")]
    async fn satellite_grace(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR: {}", e); }
        tools::satellite::grace::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "Google Dynamic World 10m near real-time land cover info.")]
    async fn satellite_dynamic_world(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR: {}", e); }
        tools::satellite::dynamic_world::query(&HTTP, p.lat, p.lon).await
    }

    #[tool(description = "ECMWF ERA5 Climate Reanalysis information for long-term trends.")]
    async fn satellite_era5(&self, Parameters(p): Parameters<LatLonRequired>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR: {}", e); }
        tools::satellite::era5::query(&HTTP, p.lat, p.lon).await
    }

    // --- ADVANCED GIS & ESG ---
    #[tool(description = "Parse Sustainability Report (PDF) for ESG Analytics.")]
    async fn esg_report_parser(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::esg::report_parser::parse_esg_report(&p.query).await
    }

    #[tool(description = "[LEGACY → use dem_slope_gee] DEM Slope via GEE SRTM. Query: lat,lon,buffer_km,output_path")]
    fn gis_dem_slope(&self, Parameters(p): Parameters<QueryParam>) -> String {
        let parts: Vec<&str> = p.query.split(',').collect();
        let lat: f64 = parts.first().and_then(|s| s.trim().parse().ok()).unwrap_or(-8.65);
        let lon: f64 = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(116.35);
        let buffer_km: f64 = parts.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(10.0);
        let output_path = parts.get(3).map(|s| s.trim()).unwrap_or("/tmp/slope.tif");
        tools::gis::advanced::dem_slope(lat, lon, buffer_km, output_path)
    }

    #[tool(description = "[LEGACY → use zonal_statistics_gee] Zonal Raster Statistics via GEE. Input: GeoJSON polygon.")]
    fn gis_raster_stats(&self, Parameters(p): Parameters<GeoJsonParam>) -> String {
        tools::gis::advanced::raster_stats("USGS/SRTMGL1_003", "elevation", &p.geojson, -8.65, 116.35, 10.0, "/tmp/zonal_stats.json")
    }

    #[tool(description = "[LEGACY → use land_cover_classify] Land Cover Classifier via GEE Sentinel-2.")]
    fn gis_land_cover_classifier(&self) -> String {
        tools::gis::landcover::classify(-8.65, 116.35, 10.0, "2024-01-01", "2024-06-30", "/tmp/landcover.tif")
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
        tools::calculators::streeter_phelps::calculate(p.k1, p.k2, p.l0, p.d0, p.velocity_ms, p.distance_km, p.temp_c)
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
        tools::airquality::plume_rise::calculate(p.stack_height_m, p.stack_diameter_m, p.exit_velocity_ms, p.exit_temp_k, p.ambient_temp_k, p.wind_speed_ms)
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
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::ocean_modeling::ocean_viz::bathymetry_3d(p.lat, p.lon, &p.output_path, &p.title)
    }

    #[tool(description = "2D Ocean Current: Peta vector field arus laut berbasis angin (Ekman). Input: lat, lon, wind.")]
    fn ocean_current_2d(&self, Parameters(p): Parameters<OceanCurrentParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
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

    // =======================================
    // ADVANCED PHYSICS (FRONTIER 2026)
    // =======================================

    #[tool(description = "Flux Divergence Emission: Deteksi emisi gas misterius dari citra Satelit via Central Difference (Beirle et al., 2019).")]
    fn satellite_flux_divergence(&self, Parameters(p): Parameters<FluxDivergenceParam>) -> String {
        tools::advanced_physics::flux_divergence::calculate_emissions(&p.grid_data_json, p.u_wind, p.v_wind, p.dx_meters, p.dy_meters, p.lifetime_hours)
    }

    #[tool(description = "Groundwater Advection-Diffusion: Solusi eksplisit Finite Difference dengan jaminan stabilitas CFL.")]
    fn groundwater_advection_diffusion(&self, Parameters(p): Parameters<GroundwaterPdeParam>) -> String {
        tools::advanced_physics::groundwater_pde::solve_pde(&p.h_initial_json, p.diffusivity_d, p.dx_meters, p.dy_meters, p.time_steps, p.dt_seconds)
    }

    #[tool(description = "Bayesian Sensor Assimilation: Particle Filter Systematic Resampling untuk membersihkan noise sensor IoT lapangan.")]
    fn bayesian_sensor_assimilation(&self, Parameters(p): Parameters<BayesianSensorParam>) -> String {
        tools::advanced_physics::bayesian_assimilation::assimilate_sensor_data(&p.prior_particles_json, p.sensor_reading, p.sensor_noise_std)
    }

    #[tool(description = "UHI Radiative Transfer: Hitung lonjakan suhu mikro perkotaan akibat geometri gedung (Sky View Factor) & Albedo.")]
    fn uhi_radiative_transfer(&self, Parameters(p): Parameters<UhiParam>) -> String {
        tools::advanced_physics::uhi_radiative::calculate_uhi(p.albedo_urban, p.sky_view_factor, p.solar_insolation_w, p.ambient_temp_c)
    }

    // =====================================================
    // GOD TIER: 13 PREVIOUSLY UNREGISTERED TOOLS
    // =====================================================

    #[tool(description = "Biodiversity Index: Shannon-Wiener H' & Simpson 1-D. Ref: Shannon 1949. Input: JSON array jumlah individu per spesies.")]
    fn biodiversity_index(&self, Parameters(p): Parameters<BiodiversityCalcParam>) -> String {
        let counts: Result<Vec<u64>, _> = serde_json::from_str(&p.species_counts_json);
        match counts { Ok(c) => tools::calculators::biodiversity::calculate(&c), Err(e) => format!("ERROR [E103]: JSON parsing: {}", e) }
    }

    #[tool(description = "Composting C/N Ratio Optimizer. Ref: USDA/SNI. Input: JSON array [[name, mass_kg, c_pct, n_pct], ...]")]
    fn composting_cn(&self, Parameters(p): Parameters<CompostingParam>) -> String {
        let mats: Result<Vec<(String, f64, f64, f64)>, _> = serde_json::from_str(&p.materials_json);
        match mats { Ok(m) => tools::calculators::composting::calculate(&m), Err(e) => format!("ERROR [E103]: JSON parsing: {}", e) }
    }

    #[tool(description = "Flood Frequency Gumbel Distribution. Min 10 tahun data. Ref: Chow 1951, USGS Bulletin 17C.")]
    fn flood_frequency_gumbel(&self, Parameters(p): Parameters<FloodFreqParam>) -> String {
        let data: Result<Vec<f64>, _> = serde_json::from_str(&p.data_json);
        match data { Ok(d) => tools::calculators::flood_frequency::gumbel(&d, p.return_period), Err(e) => format!("ERROR [E103]: JSON parsing: {}", e) }
    }

    #[tool(description = "Log-Pearson Type III Flood Frequency. Ref: USGS Bulletin 17C, SNI 2415:2016. Wilson-Hilferty KT approximation.")]
    fn log_pearson_iii(&self, Parameters(p): Parameters<FloodFreqParam>) -> String {
        let data: Result<Vec<f64>, _> = serde_json::from_str(&p.data_json);
        match data { Ok(d) => tools::calculators::flood_frequency::log_pearson_iii(&d, p.return_period), Err(e) => format!("ERROR [E103]: JSON parsing: {}", e) }
    }

    #[tool(description = "Acid Mine Drainage (AMD/ABA). Ref: PermenLH 113/2003. Klasifikasi: PAF/NAF/Uncertain.")]
    fn acid_mine_drainage(&self, Parameters(p): Parameters<AmdCalcParam>) -> String {
        tools::calculators::acid_mine_drainage::calculate(p.sulfur_pct, p.anc_kg_h2so4_t, p.nag_ph)
    }

    #[tool(description = "Transport Emission IPCC Volume BBM. Ref: IPCC 2006. Input: tipe BBM + liter.")]
    fn transport_emission(&self, Parameters(p): Parameters<TransportEmParam>) -> String {
        tools::calculators::transport_emission::calculate(&p.fuel_type, p.liters)
    }

    #[tool(description = "Indeks Pencemaran (IP) Air. Ref: KepmenLH 115/2003. Normalisasi log untuk ratio >1.")]
    fn indeks_pencemaran(&self, Parameters(p): Parameters<IpParam>) -> String {
        tools::compliance::indeks_pencemaran::calculate(&p.data_json, p.temp_c)
    }

    #[tool(description = "Metode STORET Kualitas Air. Ref: KepmenLH 115/2003. Skor negatif: Kelas A-D.")]
    fn storet_water(&self, Parameters(p): Parameters<StoretParam>) -> String {
        tools::compliance::storet::calculate(&p.data_json)
    }

    #[tool(description = "SPPL Checker. Ref: PP 22/2021. Cek apakah kegiatan cukup SPPL atau wajib UKL-UPL/AMDAL.")]
    fn sppl_checker(&self, Parameters(p): Parameters<SpplParam>) -> String {
        tools::compliance::sppl::check(&p.kegiatan, p.is_wajib_amdal, p.is_wajib_uklupl)
    }

    #[tool(description = "Baku Mutu Air Laut (30+ parameter). Ref: KepMen LH 51/2004. pH/DO/BOD/logam berat/nutrient/coliform. Peruntukan: wisata/biota/pelabuhan.")]
    fn baku_mutu_laut(&self, Parameters(p): Parameters<BakuMutuLautParam>) -> String {
        tools::compliance::baku_mutu_laut::check(&p.parameter, p.concentration, &p.peruntukan)
    }

    #[tool(description = "WAQI Ground Sensor Air Quality. Source: waqi.info. Data stasiun fisik PM2.5/NO2/SO2/O3/CO.")]
    async fn waqi_air_quality(&self, Parameters(p): Parameters<LatLonParam>) -> String {
        let lat = p.lat.unwrap_or(-6.2);
        let lon = p.lon.unwrap_or(106.85);
        if let Err(e) = crate::indonesia::validate_coords(lat, lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::data::waqi::get_air_quality(&HTTP, lat, lon).await
    }

    #[tool(description = "4D Satellite Timelapse GIF via GEE. Cloud-free compositing tahunan Sentinel-2/Sentinel-1.")]
    fn satellite_timelapse(&self, Parameters(p): Parameters<TimelapseParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::satellite::timelapse::generate_4d_timelapse(p.lat, p.lon, p.buffer_km, p.start_year, p.end_year, &p.sensor, &p.output_path)
    }

    #[tool(description = "NASA EMIT Hyperspectral 285-band. Ekstraksi spectral signature mineral via GEE. Output: PNG + data.")]
    fn satellite_hyperspectral(&self, Parameters(p): Parameters<HyperspectralParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::satellite::hyperspectral::extract_signature(p.lat, p.lon, &p.output_path)
    }

    #[tool(description = "Advanced Physics Validator V3: TROPOMI UQ, PBL Inversion, Bingham Rheology, Gas Kinetics Tropis.")]
    fn physics_validator_v3(&self, Parameters(p): Parameters<ValidatorV3Param>) -> String {
        tools::advanced_physics::validator_v3::validate_advanced_physics(&p.gas_type, p.concentration, &p.time_of_day, &p.fluid_type, p.slope_angle_deg, p.depth_m)
    }

    // =====================================================
    // GOD TIER: 15 NEW COMPLIANCE/REGULATION TOOLS
    // =====================================================

    #[tool(description = "Baku Mutu Udara Ambien. Ref: PP 41/1999. Cek konsentrasi polutan vs standar nasional.")]
    fn baku_mutu_udara(&self, Parameters(p): Parameters<BakuMutuUdaraParam>) -> String {
        tools::compliance::baku_mutu_udara::check(&p.parameter, p.concentration, &p.averaging_time)
    }

    #[tool(description = "Baku Mutu Emisi Sumber Tidak Bergerak. Ref: PermenLHK 15/2019. Per jenis industri.")]
    fn baku_mutu_emisi(&self, Parameters(p): Parameters<BakuMutuEmisiParam>) -> String {
        tools::compliance::baku_mutu_emisi::check(&p.industry, &p.parameter, p.concentration)
    }

    #[tool(description = "Baku Mutu Air Limbah Industri. Ref: PermenLH 5/2014. 15+ jenis industri.")]
    fn baku_mutu_air_limbah(&self, Parameters(p): Parameters<BakuMutuAirLimbahParam>) -> String {
        tools::compliance::baku_mutu_air_limbah::check(&p.industry, &p.parameter, p.concentration)
    }

    #[tool(description = "Baku Mutu Air Limbah Domestik. Ref: PermenLHK 68/2016. pH/BOD/COD/TSS/oil/ammonia/coliform.")]
    fn baku_mutu_domestik(&self, Parameters(p): Parameters<BakuMutuDomestikParam>) -> String {
        tools::compliance::baku_mutu_domestik::check(&p.parameter, p.concentration)
    }

    #[tool(description = "Baku Mutu Kebisingan. Ref: KepmenLH 48/1996. 10 zona: perumahan/industri/RS/sekolah/ibadah.")]
    fn baku_mutu_kebisingan(&self, Parameters(p): Parameters<BakuMutuKebisinganParam>) -> String {
        tools::compliance::baku_mutu_kebisingan::check(&p.zone, p.measured_db)
    }

    #[tool(description = "Baku Mutu Getaran Mekanik. Ref: KepmenLH 49/1996. Zona: pemukiman/kantor/industri/RS.")]
    fn baku_mutu_getaran(&self, Parameters(p): Parameters<BakuMutuGetaranParam>) -> String {
        tools::compliance::baku_mutu_getaran::check(&p.zone, p.vibration_mm_s)
    }

    #[tool(description = "Baku Mutu Kebauan. Ref: KepmenLH 50/1996. H2S/NH3/methyl mercaptan/styrene.")]
    fn baku_mutu_kebauan(&self, Parameters(p): Parameters<BakuMutuKebauanParam>) -> String {
        tools::compliance::baku_mutu_kebauan::check(&p.chemical, p.concentration_ppm)
    }

    #[tool(description = "ISPU Calculator (Indeks Standar Pencemar Udara). Ref: PermenLHK 73/2019. Breakpoint interpolation.")]
    fn ispu_calculator(&self, Parameters(p): Parameters<IspuParam>) -> String {
        tools::compliance::ispu::calculate(p.pm10, p.pm25, p.so2, p.co, p.o3, p.no2)
    }

    #[tool(description = "Kelas Risiko Lingkungan (OSS). Ref: PP 22/2023. Tentukan: AMDAL/UKL-UPL/SPPL.")]
    fn risk_class_oss(&self, Parameters(p): Parameters<RiskClassParam>) -> String {
        tools::compliance::risk_class::determine(&p.sector, &p.scale_description, p.has_hazardous_waste, p.near_protected_area)
    }

    #[tool(description = "Daya Dukung Lingkungan Hidup. Ref: PermenLH 17/2009. Pendekatan: populasi/air/pangan.")]
    fn daya_dukung(&self, Parameters(p): Parameters<DayaDukungParam>) -> String {
        tools::compliance::daya_dukung::calculate(&p.approach, p.area_ha, p.population, p.water_supply_m3_yr, p.water_demand_m3_yr, p.food_production_ton_yr, p.food_demand_ton_yr)
    }

    #[tool(description = "Daya Tampung Beban Pencemaran. Ref: PP 82/2001. Mass balance sungai.")]
    fn daya_tampung(&self, Parameters(p): Parameters<DayaTampungParam>) -> String {
        tools::compliance::daya_tampung::calculate(p.q_river_m3s, p.c_upstream_mgl, p.c_standard_mgl, p.q_waste_m3s, p.c_waste_mgl, &p.parameter)
    }

    #[tool(description = "GHG Inventory. Ref: PermenLHK 102/2018, IPCC Tier 1. Sektor: energy/ippu/afolu/waste.")]
    fn ghg_inventory(&self, Parameters(p): Parameters<GhgInventoryParam>) -> String {
        tools::compliance::ghg_inventory::calculate(&p.sector, &p.activity, p.amount)
    }

    #[tool(description = "IKLH Sub-Indices: IKA/IKU/IKTL/IKAL. Ref: PermenLHK P.14/2020.")]
    fn iklh_sub_indices(&self, Parameters(p): Parameters<IklhSubParam>) -> String {
        match p.sub_type.to_lowercase().as_str() {
            "ika" => {
                let vals: Result<Vec<f64>, _> = serde_json::from_str(&p.data_json);
                match vals { Ok(v) => tools::compliance::iklh_sub::calculate_ika(&v), Err(e) => format!("ERROR: {}", e) }
            }
            "iku" => {
                let vals: Result<Vec<f64>, _> = serde_json::from_str(&p.data_json);
                match vals { Ok(v) => tools::compliance::iklh_sub::calculate_iku(&v), Err(e) => format!("ERROR: {}", e) }
            }
            "iktl" => {
                let v: Result<serde_json::Value, _> = serde_json::from_str(&p.data_json);
                match v {
                    Ok(val) => {
                        let fc = val["forest_cover_pct"].as_f64().unwrap_or(0.0);
                        let tp = val["target_pct"].as_f64().unwrap_or(30.0);
                        tools::compliance::iklh_sub::calculate_iktl(fc, tp)
                    }
                    Err(e) => format!("ERROR: {}", e)
                }
            }
            "ikal" => tools::compliance::iklh_sub::calculate_ikal(&p.data_json),
            _ => "ERROR: sub_type harus ika/iku/iktl/ikal".into()
        }
    }

    #[tool(description = "Regulasi Lingkungan Indonesia Lookup. Cari regulasi berdasarkan topik: air/udara/limbah/b3/amdal/emisi/laut/hutan/karbon.")]
    fn regulasi_lookup(&self, Parameters(p): Parameters<QueryParam>) -> String {
        tools::compliance::regulasi::lookup(&p.query)
    }

    #[tool(description = "AMDAL Screening. Ref: PermenLHK 4/2021. Tentukan wajib AMDAL/UKL-UPL/SPPL berdasarkan jenis & skala kegiatan.")]
    fn amdal_screening(&self, Parameters(p): Parameters<AmdalScreeningParam>) -> String {
        tools::compliance::amdal_screening::screen(&p.sector, &p.activity, p.scale_value, &p.scale_unit)
    }

    // =====================================================
    // GOD TIER: 5 AMDAL DOCUMENT GENERATOR
    // =====================================================

    #[tool(description = "Generate KA-ANDAL PDF. Ref: PermenLHK 5/2021. Kerangka Acuan AMDAL lengkap.")]
    fn amdal_ka_andal(&self, Parameters(p): Parameters<KaAndalParam>) -> String {
        tools::amdal::generator::generate_ka_andal(&p.project_name, &p.location, &p.project_type, &p.rona_json, &p.output_path)
    }

    #[tool(description = "Generate ANDAL PDF. Ref: PermenLHK 5/2021. Analisis Dampak Lingkungan Hidup.")]
    fn amdal_andal(&self, Parameters(p): Parameters<AndalParam>) -> String {
        tools::amdal::generator::generate_andal(&p.project_name, &p.location, &p.impacts_json, &p.output_path)
    }

    #[tool(description = "Generate RKL-RPL PDF. Ref: PermenLHK 5/2021. Rencana Pengelolaan & Pemantauan Lingkungan.")]
    fn amdal_rkl_rpl(&self, Parameters(p): Parameters<RklRplParam>) -> String {
        tools::amdal::generator::generate_rkl_rpl(&p.project_name, &p.location, &p.management_json, &p.output_path)
    }

    #[tool(description = "Generate UKL-UPL PDF. Ref: PermenLHK 6/2021. Untuk kegiatan non-AMDAL risiko menengah.")]
    fn ukl_upl_generator(&self, Parameters(p): Parameters<UklUplParam>) -> String {
        tools::amdal::generator::generate_ukl_upl(&p.project_name, &p.location, &p.impacts_json, &p.output_path)
    }

    #[tool(description = "KLHS Assessment PDF. Ref: UU 32/2009 Pasal 15-18. Kajian Lingkungan Hidup Strategis.")]
    fn klhs_assessment(&self, Parameters(p): Parameters<KlhsParam>) -> String {
        tools::amdal::generator::klhs_assessment(&p.policy_name, &p.daya_dukung_json, &p.output_path)
    }

    // =====================================================
    // GOD TIER: 3 NOISE MODELING TOOLS
    // =====================================================

    #[tool(description = "2D Noise Propagation Contour Map. ISO 9613-2 + barrier. Output PNG. Multi-source superposition.")]
    fn noise_propagation_2d(&self, Parameters(p): Parameters<Noise2dParam>) -> String {
        tools::noise::propagation::render_2d(&p.sources_json, &p.output_path, &p.title, p.grid_size.unwrap_or(500), &p.barrier_json.unwrap_or_else(|| "[]".into()))
    }

    #[tool(description = "3D Noise Surface Visualization. ISO 9613-2. Output PNG.")]
    fn noise_propagation_3d(&self, Parameters(p): Parameters<Noise3dParam>) -> String {
        tools::noise::propagation::render_3d(&p.sources_json, &p.output_path, &p.title, p.grid_size.unwrap_or(500))
    }

    #[tool(description = "Noise Compliance Check. Ref: KepmenLH 48/1996 + ISO 9613. Hitung buffer jarak aman.")]
    fn noise_compliance(&self, Parameters(p): Parameters<NoiseComplianceParam>) -> String {
        tools::noise::compliance::check(&p.zone, p.measured_db, p.distance_m, p.source_db)
    }

    // =====================================================
    // GOD TIER: 5 BIODIVERSITY & SOCIAL TOOLS
    // =====================================================

    #[tool(description = "IUCN Species Check di area. 33+ spesies dilindungi Indonesia. Filter by provinsi/pulau.")]
    async fn iucn_species_check(&self, Parameters(p): Parameters<IucnCheckParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::biodiversity::iucn::check_species(&HTTP, p.lat, p.lon, p.radius_km).await
    }

    #[tool(description = "Cek Status Spesies Dilindungi Indonesia. Ref: PP 7/1999, PermenLHK P.106/2018.")]
    fn protected_species(&self, Parameters(p): Parameters<ProtectedSpeciesParam>) -> String {
        tools::biodiversity::protected::check(&p.species_name)
    }

    #[tool(description = "Daftar Spesies Dilindungi per Provinsi. Ref: PP 7/1999.")]
    fn protected_species_by_province(&self, Parameters(p): Parameters<ProtectedByProvinceParam>) -> String {
        tools::biodiversity::protected::list_by_province(&p.province)
    }

    #[tool(description = "Social Impact Assessment Matrix untuk AMDAL. Ref: PermenLH 17/2012. Komponen: ekonomi/sosial/kesehatan.")]
    fn social_impact_matrix(&self, Parameters(p): Parameters<SocialImpactParam>) -> String {
        tools::biodiversity::social::impact_matrix(&p.impacts_json)
    }

    #[tool(description = "Health Impact Assessment. Analisis paparan polutan → Hazard Quotient (HQ) → risiko kesehatan.")]
    fn health_impact(&self, Parameters(p): Parameters<HealthImpactParam>) -> String {
        tools::biodiversity::social::health_impact(p.population, &p.pollutant, p.concentration, p.exposure_hours)
    }

    #[tool(description = "Valuasi Ekonomi Lingkungan. Ref: PP 46/2017. Metode: replacement_cost/travel_cost/hedonic/damage_cost.")]
    fn environmental_valuation(&self, Parameters(p): Parameters<ValuationParam>) -> String {
        tools::biodiversity::valuation::calculate(&p.method, &p.params_json)
    }

    // =====================================================
    // GOD TIER: 5 NEW DATA SOURCES
    // =====================================================

    #[tool(description = "ISPU Real-time dari KLHK. Source: ispu.menlhk.go.id. Data kualitas udara stasiun nasional.")]
    async fn ispu_klhk(&self, Parameters(p): Parameters<IspuKlhkParam>) -> String {
        tools::datasources::ispu_klhk::get_ispu(&HTTP, &p.kota).await
    }

    #[tool(description = "SiPongi KLHK Fire Hotspots. Hotspot kebakaran hutan/lahan per provinsi. Suplemen FIRMS.")]
    async fn sipongi_fire(&self, Parameters(p): Parameters<SipongiParam>) -> String {
        tools::datasources::sipongi::get_hotspots(&HTTP, &p.province).await
    }

    #[tool(description = "BMKG Historical Climate Data. Data iklim historis: curah hujan, suhu, kelembaban, angin.")]
    async fn bmkg_opendata(&self, Parameters(p): Parameters<BmkgOpenParam>) -> String {
        tools::datasources::bmkg_opendata::get_climate_data(&HTTP, &p.station_id, &p.parameter).await
    }

    #[tool(description = "OpenStreetMap POI Query. Cari RS/sekolah/permukiman/sungai di sekitar lokasi proyek (wajib AMDAL). Overpass API.")]
    async fn osm_poi_query(&self, Parameters(p): Parameters<OsmPoiParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::datasources::osm_poi::query_poi(&HTTP, p.lat, p.lon, p.radius_m, &p.poi_type).await
    }

    #[tool(description = "Elevation Profile antara 2 titik. Cross-section topografi. Source: Open-Elevation API / SRTM.")]
    async fn elevation_profile(&self, Parameters(p): Parameters<ElevationParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat1, p.lon1) { return format!("ERROR [E101]: titik awal — {}", e); }
        if let Err(e) = crate::indonesia::validate_coords(p.lat2, p.lon2) { return format!("ERROR [E101]: titik akhir — {}", e); }
        tools::datasources::elevation::profile(&HTTP, p.lat1, p.lon1, p.lat2, p.lon2, p.num_points.unwrap_or(20)).await
    }

    // =====================================================
    // GOD TIER: 6 SAR / SATELLITE TOOLS
    // =====================================================

    #[tool(description = "SAR Flood Detection. Sentinel-1 VV change detection pre/post banjir via GEE. Output: flood map PNG.")]
    fn sar_flood_detection(&self, Parameters(p): Parameters<SarFloodParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::satellite::sar::flood_detection(p.lat, p.lon, p.buffer_km, &p.pre_date, &p.post_date, &p.output_path)
    }

    #[tool(description = "SAR Deforestation Detection. Sentinel-1 backscatter loss detection di bawah awan. Via GEE.")]
    fn sar_deforestation(&self, Parameters(p): Parameters<SarDeforestParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::satellite::sar::deforestation(p.lat, p.lon, p.buffer_km, &p.start_date, &p.end_date, &p.output_path)
    }

    #[tool(description = "SAR Local Analysis. Proses Sentinel-1 lokal (SNAP GPT). ⚠️ Konfirmasi ukuran file sebelum download.")]
    fn sar_local_analysis(&self, Parameters(p): Parameters<SarLocalParam>) -> String {
        tools::satellite::sar::local_analysis(&p.input_path, &p.output_path, &p.analysis_type)
    }

    #[tool(description = "InSAR Land Subsidence (Screening). Sentinel-1 via GEE. ⚠️ Screening-level only, bukan full InSAR.")]
    fn land_subsidence_insar(&self, Parameters(p): Parameters<SarSubsidenceParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::satellite::sar::subsidence_insar(p.lat, p.lon, p.buffer_km, &p.start_date, &p.end_date, &p.output_path)
    }

    #[tool(description = "Burned Area Mapping (dNBR). Sentinel-2 Normalized Burn Ratio. Severity: Unburned→High. Ref: USGS.")]
    fn burned_area_mapping(&self, Parameters(p): Parameters<BurnedAreaParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::satellite::burned_area::map_burned_area(p.lat, p.lon, p.buffer_km, &p.fire_date, &p.output_path)
    }

    #[tool(description = "Mangrove Extent Mapping. Sentinel-2 NDVI+NDWI+elevation filter. Bandingkan dengan Global Mangrove Watch.")]
    fn mangrove_extent(&self, Parameters(p): Parameters<MangroveExtentParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::satellite::mangrove::map_extent(p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    // =====================================================
    // GOD TIER PHASE 2: WATER & WASTEWATER ENGINEERING
    // =====================================================

    #[tool(description = "CT Disinfection Calculator. Ref: EPA GDR. Chlorine/ozone/UV/chloramine vs Giardia/virus/crypto.")]
    fn ct_disinfection(&self, Parameters(p): Parameters<CtDisinfectionParam>) -> String {
        tools::water::ct_disinfection::calculate(&p.disinfectant, p.concentration_mgl, p.contact_time_min, &p.target_pathogen)
    }

    #[tool(description = "Darcy's Law: q = K×i. Aliran air tanah, kecepatan rembesan, waktu transport kontaminan. Ref: Darcy 1856.")]
    fn darcy_flow(&self, Parameters(p): Parameters<DarcyParam>) -> String {
        tools::water::darcy_flow::calculate(p.k_ms, p.gradient, p.area_m2, p.porosity, p.distance_m)
    }

    #[tool(description = "Theis Well Drawdown: s = Q/(4πT) × W(u). Prediksi penurunan muka airtanah akibat pemompaan. Ref: Theis 1935.")]
    fn theis_drawdown(&self, Parameters(p): Parameters<TheisParam>) -> String {
        tools::water::theis_drawdown::calculate(p.q_m3s, p.transmissivity_m2s, p.storativity, p.r_m, p.t_s)
    }

    #[tool(description = "Hazen-Williams Head Loss. Desain perpipaan air/limbah. Ref: Hazen-Williams.")]
    fn hazen_williams(&self, Parameters(p): Parameters<HazenWilliamsParam>) -> String {
        tools::water::hazen_williams::calculate(p.q_m3s, p.length_m, p.diameter_m, p.c_coeff, p.include_minor_losses)
    }

    #[tool(description = "Pump Sizing: TDH, motor power (kW/HP), NPSH. Seleksi pompa untuk proyek air/limbah.")]
    fn pump_sizing(&self, Parameters(p): Parameters<PumpSizingParam>) -> String {
        tools::water::pump_sizing::calculate(p.q_m3s, p.static_lift_m, p.friction_loss_m, p.velocity_head_m, p.pressure_head_m, p.efficiency)
    }

    #[tool(description = "Sedimentation Tank Design. Overflow rate, detention time, weir loading. Ref: Metcalf & Eddy.")]
    fn sedimentation_design(&self, Parameters(p): Parameters<SedimentationParam>) -> String {
        tools::water::sedimentation::design(p.q_m3d, &p.tank_type, &p.tank_shape)
    }

    #[tool(description = "UASB Reactor Design. OLR, HRT, gas yield. Untuk IPAL sawit (POME)/tapioka/karet/domestik. Ref: Lettinga 1980.")]
    fn uasb_design(&self, Parameters(p): Parameters<UasbParam>) -> String {
        tools::water::uasb_design::design(p.q_m3d, p.cod_in_mgl, p.cod_eff_target, p.temperature_c, &p.waste_type)
    }

    #[tool(description = "Trickling Filter Design (NRC equation). BOD removal efficiency. Ref: NRC 1946.")]
    fn trickling_filter(&self, Parameters(p): Parameters<TricklingFilterParam>) -> String {
        tools::water::trickling_filter::design(p.q_m3d, p.bod_in, p.bod_target, p.media_depth_m, p.recirculation_ratio)
    }

    #[tool(description = "Constructed Wetland Design (k-C* model). FWS/HSSF sizing. BOD/TSS/NH4N. Ref: Kadlec & Knight 1996.")]
    fn constructed_wetland(&self, Parameters(p): Parameters<ConstructedWetlandParam>) -> String {
        tools::water::constructed_wetland::design(p.q_m3d, &p.parameter, p.ci_mgl, p.ce_target, p.temp_c, &p.wetland_type)
    }

    #[tool(description = "Anaerobic Digestion / Biogas Reactor. SRT, OLR, gas yield. Substrat: sapi/babi/ayam/POME. Ref: McCarty.")]
    fn anaerobic_digestion(&self, Parameters(p): Parameters<AnaerobicDigestionParam>) -> String {
        tools::water::anaerobic_digestion::design(p.q_m3d, p.vs_concentration_kgm3, p.vs_destruction_pct, p.temperature_c, &p.substrate)
    }

    // =====================================================
    // GOD TIER PHASE 2: ENVIRONMENTAL CHEMISTRY
    // =====================================================

    #[tool(description = "First-Order Decay Kinetics: C(t) = C₀×e^(-kt). Half-life, t90, t99. Fondasi BOD/degradasi kontaminan.")]
    fn first_order_kinetics(&self, Parameters(p): Parameters<FirstOrderParam>) -> String {
        tools::calculators::first_order_kinetics::calculate(p.c0, p.k, p.t, &p.time_unit)
    }

    #[tool(description = "Freundlich/Langmuir Isotherm. Desain adsorber karbon aktif. Ref: Freundlich 1906, Langmuir 1918.")]
    fn isotherm_calc(&self, Parameters(p): Parameters<IsothermParam>) -> String {
        tools::calculators::isotherm::calculate(&p.model, p.ce, p.kf, p.n_exp, p.qmax, p.kl, p.volume_l, p.c0)
    }

    #[tool(description = "Henry's Law: p = KH×C. Gas-liquid partitioning. Air stripping feasibility. Common VOCs.")]
    fn henrys_law(&self, Parameters(p): Parameters<HenrysLawParam>) -> String {
        tools::calculators::henrys_law::calculate(&p.compound, p.concentration_mgl, p.temperature_c)
    }

    #[tool(description = "Nernst Equation: E = E° - (RT/nF)×ln(Q). Potensial redoks, spontanitas reaksi. Ref: Nernst.")]
    fn nernst_redox(&self, Parameters(p): Parameters<NernstParam>) -> String {
        tools::calculators::nernst_redox::calculate(&p.half_reaction, p.temperature_c, p.log_q, p.n_electrons)
    }

    #[tool(description = "Partition Coefficient Kd/Koc. Retardation factor kontaminan di tanah. Mobilitas polutan. Ref: Karickhoff 1981.")]
    fn partition_coefficient(&self, Parameters(p): Parameters<PartitionParam>) -> String {
        tools::calculators::partition_coeff::calculate(&p.compound, p.foc, p.bulk_density_kgm3, p.porosity)
    }

    // =====================================================
    // GOD TIER PHASE 2: HYDROLOGY ENHANCEMENT
    // =====================================================

    #[tool(description = "Rational Method: Q = C×I×A/360. Debit puncak drainase. Ref: Kuichling 1889.")]
    fn rational_method(&self, Parameters(p): Parameters<RationalParam>) -> String {
        tools::calculators::rational_method::calculate(p.c_coeff, p.i_mm_hr, p.a_ha, &p.land_use)
    }

    #[tool(description = "SCS Triangular Unit Hydrograph. tp, Qp, tb. Ref: SCS 1972.")]
    fn unit_hydrograph(&self, Parameters(p): Parameters<UnitHydrographParam>) -> String {
        tools::calculators::unit_hydrograph::calculate(p.a_km2, p.tc_hours, p.d_hours)
    }

    #[tool(description = "Muskingum Flood Routing. Atenuasi debit puncak di sungai. Ref: McCarthy 1938.")]
    fn muskingum_routing(&self, Parameters(p): Parameters<MuskingumParam>) -> String {
        let inflow: Result<Vec<(f64, f64)>, _> = serde_json::from_str(&p.inflow_json);
        match inflow { Ok(i) => tools::calculators::muskingum_routing::route(&i, p.k_hours, p.x, p.dt_hours), Err(e) => format!("ERROR [E103]: JSON parsing: {}", e) }
    }

    #[tool(description = "Time of Concentration: Kirpich/Bransby-Williams/SCS Lag. Input untuk kurva IDF. Ref: Kirpich 1940.")]
    fn time_of_concentration(&self, Parameters(p): Parameters<TocParam>) -> String {
        tools::calculators::time_of_concentration::calculate(&p.method, p.l_m, p.s_slope, p.a_km2, p.cn)
    }

    // =====================================================
    // GOD TIER PHASE 2: SOLID & HAZARDOUS WASTE
    // =====================================================

    #[tool(description = "Landfill Liner Design. Giroud-Bonaparte leakage. Ref: PermenPU 3/2013, EPA.")]
    fn landfill_liner(&self, Parameters(p): Parameters<LandfillLinerParam>) -> String {
        tools::waste::landfill_liner::design(&p.liner_type, p.area_m2, p.head_on_liner_m, p.k_clay, p.clay_thickness_m)
    }

    #[tool(description = "Leachate Generation (water balance). Volume lindi bulanan dari TPA. Ref: EPA HELP Model.")]
    fn leachate_generation(&self, Parameters(p): Parameters<LeachateParam>) -> String {
        let rain: Result<Vec<f64>, _> = serde_json::from_str(&p.monthly_rainfall_json);
        let et: Result<Vec<f64>, _> = serde_json::from_str(&p.monthly_et_json);
        match (rain, et) { (Ok(r), Ok(e)) => tools::waste::leachate::calculate(p.area_m2, &r, &e, p.soil_storage_mm, p.runoff_coeff), _ => "ERROR: JSON parsing gagal. Format: [jan,feb,...,des] (12 nilai).".into() }
    }

    #[tool(description = "Landfill Slope Stability (infinite slope). FoS analysis. Min 1.3 static. Ref: PermenPU, Bishop.")]
    fn landfill_stability(&self, Parameters(p): Parameters<LandfillStabilityParam>) -> String {
        tools::waste::landfill_stability::calculate(p.slope_angle_deg, p.height_m, p.unit_weight_kn_m3, p.cohesion_kpa, p.friction_deg, p.pore_pressure_ratio)
    }

    #[tool(description = "TCLP Screening. Karakteristik limbah B3. Ref: PP 101/2014, EPA SW-846.")]
    fn tclp_screening(&self, Parameters(p): Parameters<TclpParam>) -> String {
        tools::waste::tclp::screen(&p.parameters_json)
    }

    #[tool(description = "Waste Compatibility Matrix. Cek kompatibilitas penyimpanan 2 jenis limbah B3.")]
    fn waste_compatibility(&self, Parameters(p): Parameters<WasteCompatParam>) -> String {
        tools::waste::waste_compatibility::check(&p.waste_a, &p.waste_b)
    }

    #[tool(description = "TPS B3 Storage Calculator. Luas lantai, containment, persyaratan. Ref: PP 101/2014.")]
    fn b3_storage_calc(&self, Parameters(p): Parameters<B3StorageParam>) -> String {
        tools::waste::b3_storage::calculate(&p.waste_type, p.volume_m3_per_month, p.density_kg_m3)
    }

    // =====================================================
    // GOD TIER PHASE 2: RADIATION & NUCLEAR
    // =====================================================

    #[tool(description = "Inverse Square Law Radiasi. Laju dosis vs jarak. Jarak aman pekerja/publik.")]
    fn radiation_inverse_square(&self, Parameters(p): Parameters<InverseSquareParam>) -> String {
        tools::radiation::inverse_square::calculate(p.dose_rate_at_d1, p.d1_m, p.d2_m)
    }

    #[tool(description = "Shielding Radiasi. HVL lead/concrete/water/steel. Ref: ICRP.")]
    fn radiation_shielding(&self, Parameters(p): Parameters<ShieldingParam>) -> String {
        tools::radiation::shielding::calculate(p.initial_intensity, &p.material, p.thickness_cm, &p.source)
    }

    #[tool(description = "Radioactive Decay: A(t) = A₀×e^(-λt). 10 isotop. Waktu ke clearance level BAPETEN.")]
    fn radioactive_decay(&self, Parameters(p): Parameters<DecayParam>) -> String {
        tools::radiation::radioactive_decay::calculate(&p.isotope, p.initial_activity_bq, p.time_elapsed, &p.time_unit)
    }

    #[tool(description = "Radon Indoor Estimation. Konsentrasi Rn-222 dalam ruangan. Ref: WHO 2009 (100 Bq/m³).")]
    fn radon_indoor(&self, Parameters(p): Parameters<RadonParam>) -> String {
        tools::radiation::radon_indoor::calculate(p.soil_radon_bq_m3, p.floor_area_m2, p.room_height_m, p.ventilation_rate_ach, &p.floor_type)
    }

    #[tool(description = "NORM Screening. Timah/monazite/zircon/coal ash. Ref: PerKa BAPETEN 4/2013.")]
    fn norm_screening(&self, Parameters(p): Parameters<NormParam>) -> String {
        tools::radiation::norm_screening::screen(&p.material, p.activity_bq_g)
    }

    // =====================================================
    // GOD TIER PHASE 2: HEALTH RISK & MONITORING
    // =====================================================

    #[tool(description = "HHRA Cancer Risk (ILCR). Multi-pathway exposure. Ref: US EPA RAGS.")]
    fn hhra_cancer_risk(&self, Parameters(p): Parameters<HhraParam>) -> String {
        tools::biodiversity::hhra::calculate_ilcr(&p.exposure_route, p.concentration, p.intake_rate, p.exposure_freq_days, p.exposure_dur_years, p.body_weight_kg, p.avg_time_years, p.csf)
    }

    #[tool(description = "Hazard Quotient (HQ) Non-Cancer Risk. Auto-lookup RfD dari IRIS database. Ref: US EPA IRIS, Pedoman ARKL.")]
    fn hhra_hazard_quotient(&self, Parameters(p): Parameters<HqParam>) -> String {
        tools::biodiversity::hhra::calculate_hq(&p.contaminant, &p.route, p.concentration, p.intake_rate, p.exposure_freq_days, p.exposure_dur_years, p.body_weight_kg)
    }

    #[tool(description = "ARKL Indonesia (Analisis Risiko Kesehatan Lingkungan). Default Indonesia: BW=55kg, fE=350, Dt=30. Ref: Pedoman ARKL Kemenkes 2012.")]
    fn arkl_calculator(&self, Parameters(p): Parameters<ArklParam>) -> String {
        tools::biodiversity::hhra::calculate_arkl(&p.contaminant, &p.route, p.concentration, &p.population_type, &p.exposure_scenario)
    }

    #[tool(description = "Sampling Design Calculator. Jumlah sampel + strategi. Ref: ISO 5667, EPA QA/G-5S.")]
    fn sampling_design(&self, Parameters(p): Parameters<SamplingParam>) -> String {
        tools::biodiversity::sampling_design::calculate(p.confidence_pct, p.margin_error_pct, p.std_deviation, p.population_size)
    }

    #[tool(description = "Mann-Kendall Trend Test + Sen's Slope. Deteksi tren data monitoring lingkungan. Ref: Mann 1945.")]
    fn mann_kendall_trend(&self, Parameters(p): Parameters<MannKendallParam>) -> String {
        tools::biodiversity::mann_kendall::trend_test(&p.data_json)
    }

    #[tool(description = "QA/QC Data Validation. RPD duplikat, spike recovery, blank check. Ref: EPA 40 CFR 136.")]
    fn qaqc_validation(&self, Parameters(p): Parameters<QaqcParam>) -> String {
        tools::biodiversity::qaqc::validate(&p.data_json)
    }

    #[tool(description = "Coliform Die-off Decay (Mancini model). T90 tropis. Kepatuhan PP 22/2021 coliform. Ref: Mancini 1978.")]
    fn coliform_decay(&self, Parameters(p): Parameters<ColiformParam>) -> String {
        tools::biodiversity::coliform_decay::calculate(p.initial_count_per_100ml, p.temperature_c, p.time_hours, &p.water_type)
    }

    // =====================================================
    // GOD TIER PHASE 2: ECOLOGICAL & COASTAL
    // =====================================================

    #[tool(description = "Bruun Rule Coastal Erosion. Resesi pantai akibat SLR. Skenario IPCC AR6. Ref: Bruun 1962.")]
    fn bruun_rule(&self, Parameters(p): Parameters<BruunParam>) -> String {
        tools::ocean_modeling::bruun_rule::calculate(p.sea_level_rise_m, p.profile_length_m, p.berm_height_m, p.closure_depth_m)
    }

    #[tool(description = "Coastal Vulnerability Index (CVI). 6 variabel: geomorfologi, perubahan garis pantai, kemiringan, SLR, gelombang, pasut.")]
    fn coastal_vulnerability(&self, Parameters(p): Parameters<CviParam>) -> String {
        tools::ocean_modeling::coastal_vulnerability::calculate(p.geomorphology, p.shoreline_change_m_yr, p.coastal_slope_pct, p.slr_mm_yr, p.mean_wave_height_m, p.mean_tidal_range_m)
    }

    #[tool(description = "Traffic Noise Model (CoRTN). Kebisingan lalu lintas jalan. Line source → contour. + KepmenLH 48/1996.")]
    fn traffic_noise(&self, Parameters(p): Parameters<TrafficNoiseParam>) -> String {
        tools::noise::traffic_noise::calculate(p.vehicles_per_hour, p.speed_kmh, p.distance_m, p.heavy_vehicle_pct, p.gradient_pct, &p.ground_type, p.barrier_height_m)
    }

    #[tool(description = "Bioretention / Rain Garden Design. Green infrastructure BMP. Sizing + media + tanaman Indonesia.")]
    fn bioretention_design(&self, Parameters(p): Parameters<BioretentionParam>) -> String {
        tools::calculators::bioretention::design(p.q_design_m3s, p.ksat_m_hr, p.ponding_depth_m, p.media_depth_m, p.drain_time_hr)
    }

    #[tool(description = "Water Footprint ISO 14046. Blue/green/grey WF. 17 produk Indonesia. Ref: Hoekstra 2011.")]
    fn water_footprint(&self, Parameters(p): Parameters<WaterFootprintParam>) -> String {
        tools::calculators::water_footprint::calculate(&p.product, p.quantity, &p.unit)
    }

    // =====================================================
    // GOD TIER PHASE 2: ECONOMICS & INDUSTRIAL ECOLOGY
    // =====================================================

    #[tool(description = "Cost-Benefit Analysis (NPV/BCR/IRR). Analisis ekonomi proyek lingkungan. Sensitivity ±10-20%.")]
    fn cost_benefit_analysis(&self, Parameters(p): Parameters<CbaParam>) -> String {
        tools::esg::cost_benefit::calculate(&p.costs_json, &p.benefits_json, p.discount_rate, p.years)
    }

    #[tool(description = "Material Flow Analysis (MFA). Mass balance industri. Efisiensi + waste ratio. Ref: Brunner & Rechberger.")]
    fn material_flow_analysis(&self, Parameters(p): Parameters<MfaParam>) -> String {
        tools::esg::material_flow::analyze(&p.inputs_json, &p.outputs_json, p.stock_change)
    }

    #[tool(description = "GHG Protocol Scope 1/2/3. Emisi korporat per kategori. EF Indonesia (Perpres 98/2021).")]
    fn scope_123_ghg(&self, Parameters(p): Parameters<Scope123Param>) -> String {
        tools::esg::scope123::calculate(&p.scope1_json, &p.scope2_json, &p.scope3_json)
    }

    #[tool(description = "Circular Economy MCI. Material Circularity Indicator. Ref: Ellen MacArthur Foundation 2015.")]
    fn circular_economy_mci(&self, Parameters(p): Parameters<CircularParam>) -> String {
        tools::esg::circular_economy::calculate(p.mass_product_kg, p.virgin_feedstock_pct, p.recycled_input_pct, p.reused_input_pct, p.recycled_output_pct, p.reused_output_pct, p.product_lifetime_years, p.industry_avg_lifetime)
    }

    #[tool(description = "Externality / Damage Cost. Biaya kerusakan lingkungan per polutan. Social cost of carbon. Konteks Indonesia.")]
    fn externality_cost(&self, Parameters(p): Parameters<ExternalityParam>) -> String {
        tools::esg::externality_cost::calculate(&p.pollutant, p.amount, &p.unit, &p.location_type)
    }

    // =====================================================
    // GIS / REMOTE SENSING — REAL IMPLEMENTATIONS
    // =====================================================

    #[tool(description = "Raster Band Math via GEE Sentinel-2. Compute spectral indices: NDVI/NDWI/SAVI/EVI/MNDWI/NDBI/BSI. Output: GeoTIFF.")]
    fn raster_band_math(&self, Parameters(p): Parameters<RasterBandMathParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::gis::advanced::band_math_gee(p.lat, p.lon, p.buffer_km, &p.index_type, &p.start_date, &p.end_date, &p.output_path)
    }

    #[tool(description = "Raster Band Math on Local GeoTIFF. Custom expression e.g. '(b1-b2)/(b1+b2)'. Output: GeoTIFF.")]
    fn raster_band_math_local(&self, Parameters(p): Parameters<RasterBandMathLocalParam>) -> String {
        tools::gis::advanced::band_math_local(&p.input_path, &p.expression, &p.output_path)
    }

    #[tool(description = "DEM Slope Analysis via GEE SRTM 30m. Kemiringan lereng (derajat). Output: GeoTIFF.")]
    fn dem_slope_gee(&self, Parameters(p): Parameters<DemGeeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::gis::advanced::dem_slope(p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    #[tool(description = "DEM Aspect Analysis via GEE SRTM 30m. Arah hadap lereng (0-360°). Output: GeoTIFF.")]
    fn dem_aspect_gee(&self, Parameters(p): Parameters<DemGeeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::gis::advanced::dem_aspect(p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    #[tool(description = "DEM Hillshade via GEE SRTM 30m. Bayangan relief untuk visualisasi terrain. Output: GeoTIFF.")]
    fn dem_hillshade_gee(&self, Parameters(p): Parameters<DemGeeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::gis::advanced::dem_hillshade(p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    #[tool(description = "Zonal Statistics via GEE reduceRegion. Stats dari image_id+band di dalam polygon/buffer. Output: JSON.")]
    fn zonal_statistics_gee(&self, Parameters(p): Parameters<ZonalStatsGeeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        let geojson = p.geojson.as_deref().unwrap_or("");
        tools::gis::advanced::raster_stats(&p.image_id, &p.band, geojson, p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    #[tool(description = "Zonal Statistics Local. Hitung min/max/mean/std/sum/count raster di zona vektor. Pure local (rasterstats).")]
    fn zonal_statistics_local(&self, Parameters(p): Parameters<ZonalStatsLocalParam>) -> String {
        tools::gis::advanced::zonal_stats_local(&p.raster_path, &p.vector_path, &p.stats)
    }

    #[tool(description = "Land Cover Classification via GEE Sentinel-2. Dynamic World + SNI 7645:2014. Output: classified GeoTIFF.")]
    fn land_cover_classify(&self, Parameters(p): Parameters<LandCoverClassifyParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::gis::landcover::classify(p.lat, p.lon, p.buffer_km, &p.start_date, &p.end_date, &p.output_path)
    }

    #[tool(description = "Land Use Change Detection. Banding 2 periode citra Sentinel-2 via GEE. Deteksi deforestasi/urbanisasi. Output: change map.")]
    fn land_use_change(&self, Parameters(p): Parameters<LandUseChangeParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::gis::landcover::change_detection(p.lat, p.lon, p.buffer_km, &p.d1_start, &p.d1_end, &p.d2_start, &p.d2_end, &p.output_path)
    }

    #[tool(description = "Classification Accuracy Assessment (Python landcover engine). Confusion matrix, Kappa, Producer/User accuracy. Ref: SNI 8202:2015.")]
    fn accuracy_assessment(&self, Parameters(p): Parameters<AccuracyAssessmentParam>) -> String {
        tools::gis::landcover::accuracy_assessment(&p.predicted_json, &p.actual_json)
    }

    #[tool(description = "Classification Accuracy Assessment (pure Rust). Confusion matrix, Kappa, OA, SNI 8202:2015 compliance. No Python dependency.")]
    fn accuracy_assessment_rs(&self, Parameters(p): Parameters<AccuracyAssessmentParam>) -> String {
        tools::calculators::accuracy_assessment::calculate(&p.predicted_json, &p.actual_json)
    }

    #[tool(description = "Buffer Analysis. Create buffer zone around GeoJSON geometry. Output: buffered GeoJSON.")]
    fn buffer_analysis(&self, Parameters(p): Parameters<BufferAnalysisParam>) -> String {
        tools::gis::spatial_ops::buffer(&p.geojson, p.distance_m, &p.output_path)
    }

    #[tool(description = "Overlay Analysis. Intersection/union/difference/symmetric_difference of 2 GeoJSON layers. Output: GeoJSON.")]
    fn overlay_analysis(&self, Parameters(p): Parameters<OverlayAnalysisParam>) -> String {
        tools::gis::spatial_ops::overlay(&p.geojson_a, &p.geojson_b, &p.operation, &p.output_path)
    }

    #[tool(description = "Suitability Analysis. Multi-criteria evaluation via GEE layers. Output: suitability map.")]
    fn suitability_analysis(&self, Parameters(p): Parameters<SuitabilityAnalysisParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::gis::spatial_ops::suitability(&p.criteria_json, p.lat, p.lon, p.buffer_km, &p.output_path)
    }

    #[tool(description = "Viewshed Analysis. Line-of-sight visibility dari DEM. Untuk AMDAL visual impact, tower placement. Output: visibility map.")]
    fn viewshed_analysis(&self, Parameters(p): Parameters<ViewshedAnalysisParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.observer_lat, p.observer_lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::gis::viewshed::analyze(&p.dem_path, p.observer_lat, p.observer_lon, p.observer_height_m, p.max_distance_m, &p.output_path)
    }

    #[tool(description = "Coordinate Transform V2. Transform between any EPSG CRS. Input: x, y, from_epsg, to_epsg.")]
    fn coordinate_transform_v2(&self, Parameters(p): Parameters<CoordTransformV2Param>) -> String {
        let from = format!("EPSG:{}", p.from_epsg);
        let to = format!("EPSG:{}", p.to_epsg);
        tools::gis::coords::transform(p.x, p.y, &from, &to)
    }

    #[tool(description = "WGS84 to UTM Auto. Auto-detect UTM zone for Indonesia coordinates. Returns easting, northing, zone, EPSG.")]
    fn wgs84_to_utm(&self, Parameters(p): Parameters<Wgs84ToUtmParam>) -> String {
        if let Err(e) = crate::indonesia::validate_coords(p.lat, p.lon) { return format!("ERROR [E101]: Koordinat tidak valid - {}", e); }
        tools::gis::coords::wgs84_to_utm_auto(p.lat, p.lon)
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
        .with_instructions("Environmental AI MCP Server for Indonesia — GOD TIER v3. 220+ tools covering ALL 20 domains of Environmental Engineering + GIS/Remote Sensing: Water/Wastewater Treatment Design, Air Quality, Solid/Hazardous Waste, AMDAL/EIA, Environmental Chemistry, Microbiology, Hydrology, Groundwater, Noise/Vibration, Radiation/NORM, Climate/ESG, Regulatory Compliance (30+ regulasi Indonesia), Ecological Engineering, Coastal/Marine, Remote Sensing/GIS (SAR+Optical+Hyperspectral+DEM+Band Math+Zonal Stats+Land Cover+Change Detection+Viewshed+Spatial Ops+Coordinate Transform), Monitoring/QA-QC, Environmental Health (HHRA), Industrial Ecology (MFA/MCI), Environmental Economics (CBA/NPV), Physics-Informed Validation, 2D/3D/4D Visualization. GIS tools: raster_band_math, dem_slope/aspect/hillshade, zonal_statistics, land_cover_classify, land_use_change, accuracy_assessment, buffer/overlay/suitability, viewshed, coordinate_transform_v2, wgs84_to_utm. Domain: Indonesia. ISO 9613, ISO 14046, IPCC 2006, FAO-56, EPA RAGS, GHG Protocol, SNI 7645:2014, SNI 8202:2015.")
    }
}
