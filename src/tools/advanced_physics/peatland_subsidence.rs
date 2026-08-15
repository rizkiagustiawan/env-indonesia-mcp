use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Peatland Subsidence & Carbon Emission Model (Tropical Peat)
/// Ref: Hooijer et al. (2012), PP 71/2014, PP 57/2016
/// Calculates subsidence (cm) and CO2 oxidation (t/ha/yr) based on Groundwater Level (GWL).

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PeatlandSubsidenceParam {
    pub gwl_m: f64,
    pub duration_years: f64,
    pub area_ha: f64,
}

pub fn calculate_peatland_subsidence(param: &PeatlandSubsidenceParam) -> String {
    let mut out = String::from("=== Peatland Subsidence & CO2 Emission Model ===\n");
    out.push_str("Ref: Hooijer et al. (2012), PP 71/2014, PP 57/2016\n\n");

    let gwl = param.gwl_m;
    
    // Regulatory check: PP 71/2014 TMAT max 0.4m (40 cm) below surface
    let is_legal = gwl <= 0.4;
    let compliance_status = if is_legal {
        "MEMENUHI SYARAT (TMAT <= 40 cm)"
    } else {
        "PELANGGARAN HUKUM (TMAT > 40 cm) - PP 71/2014"
    };

    out.push_str(&format!("Groundwater Level (TMAT) : {:.2} m di bawah permukaan\n", gwl));
    out.push_str(&format!("Durasi Drainase          : {:.1} tahun\n", param.duration_years));
    out.push_str(&format!("Luas Area                : {:.1} ha\n", param.area_ha));
    out.push_str(&format!("Status Kepatuhan         : {}\n\n", compliance_status));

    // Physics Model (Hooijer et al., 2012 for Southeast Asian tropical peat)
    // 1. Initial consolidation happens in year 1.
    // 2. Linear oxidation/subsidence thereafter.
    // Conservative operational formula: 
    // Subsidence = ~5 cm/yr per 1 meter of drainage
    // CO2 Emission = ~91 tons/ha/yr per 1 meter of drainage
    
    let annual_subsidence_cm = if gwl > 0.0 { gwl * 5.0 } else { 0.0 };
    let total_subsidence_cm = annual_subsidence_cm * param.duration_years;
    
    let annual_co2_per_ha = if gwl > 0.0 { gwl * 91.0 } else { 0.0 };
    let total_annual_co2 = annual_co2_per_ha * param.area_ha;
    let cumulative_co2 = total_annual_co2 * param.duration_years;

    out.push_str("Proyeksi Fisika & Emisi Karbon:\n");
    out.push_str(&format!("  Laju Subsidence        : {:.2} cm/tahun\n", annual_subsidence_cm));
    out.push_str(&format!("  Total Subsidence ({} thn) : {:.2} cm\n", param.duration_years, total_subsidence_cm));
    out.push_str(&format!("  Emisi CO2 (Oksidasi)   : {:.2} ton CO2/ha/tahun\n", annual_co2_per_ha));
    out.push_str(&format!("  Total Emisi Tahunan    : {:.2} ton CO2/tahun\n", total_annual_co2));
    out.push_str(&format!("  Emisi Kumulatif        : {:.2} ton CO2\n\n", cumulative_co2));

    out.push_str("Dampak Lanjutan & Mitigasi:\n");
    if total_subsidence_cm > 50.0 {
        out.push_str("  [!] BAHAYA: Subsidence > 50 cm berisiko memicu banjir permanen (rob) jika dekat pantai/sungai.\n");
    }
    if !is_legal {
        out.push_str("  [!] MITIGASI WAJIB: Lakukan pembasahan kembali (rewetting) dengan sekat kanal (canal blocking) untuk menaikkan muka air tanah ke < 40 cm.\n");
    }

    out.push_str("\n");
    let payload = crate::result_contract::ScientificResult::new(
        "Peatland_Cumulative_CO2_tons",
        cumulative_co2,
        "tons",
    );
    out.push_str(&serde_json::to_string(&payload).unwrap_or_default());

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peatland_legal() {
        let p = PeatlandSubsidenceParam {
            gwl_m: 0.3, // 30 cm, legal
            duration_years: 10.0,
            area_ha: 100.0,
        };
        let out = calculate_peatland_subsidence(&p);
        assert!(out.contains("MEMENUHI SYARAT"));
        assert!(out.contains("2730.00 ton CO2/tahun")); // 0.3 * 91 * 100
    }

    #[test]
    fn test_peatland_illegal() {
        let p = PeatlandSubsidenceParam {
            gwl_m: 0.8, // 80 cm, illegal
            duration_years: 5.0,
            area_ha: 100.0,
        };
        let out = calculate_peatland_subsidence(&p);
        assert!(out.contains("PELANGGARAN HUKUM"));
        assert!(out.contains("4.00 cm/tahun")); // 0.8 * 5
    }
}
