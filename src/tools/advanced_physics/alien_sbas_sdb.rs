use std::process::Command;
use serde::{Deserialize, Serialize};
use rmcp::schemars::{self, JsonSchema};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SbasPeatParam {
    #[schemars(description = "Subsidence velocity in mm/yr (e.g. from Sentinel-1 SBAS)")]
    pub subsidence_mm_yr: f64,
    #[schemars(description = "Water table drawdown depth in meters")]
    pub water_table_m: f64,
    #[schemars(description = "Area of the peatland zone in hectares")]
    pub area_ha: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SdbParam {
    #[schemars(description = "Remote sensing reflectance in Blue band (Sentinel-2 B2)")]
    pub r_rs_blue: f64,
    #[schemars(description = "Remote sensing reflectance in Green band (Sentinel-2 B3)")]
    pub r_rs_green: f64,
    #[schemars(description = "Remote sensing reflectance in Red band (Sentinel-2 B4)")]
    pub r_rs_red: f64,
    #[schemars(description = "Diffuse attenuation coefficient Kd for Blue band (turbidity proxy)")]
    pub kd_blue: f64,
    #[schemars(description = "Diffuse attenuation coefficient Kd for Green band (turbidity proxy)")]
    pub kd_green: f64,
}

pub fn invert_peat_thickness(p: &SbasPeatParam) -> String {
    let script = "src/tools/advanced_physics/alien_sbas_sdb.py";
    let out = Command::new("python3")
        .arg(script)
        .arg("peat")
        .arg(p.subsidence_mm_yr.to_string())
        .arg(p.water_table_m.to_string())
        .arg(p.area_ha.to_string())
        .output();
    match out {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn invert_bathymetry(p: &SdbParam) -> String {
    let script = "src/tools/advanced_physics/alien_sbas_sdb.py";
    let out = Command::new("python3")
        .arg(script)
        .arg("sdb")
        .arg(p.r_rs_blue.to_string())
        .arg(p.r_rs_green.to_string())
        .arg(p.r_rs_red.to_string())
        .arg(p.kd_blue.to_string())
        .arg(p.kd_green.to_string())
        .output();
    match out {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
