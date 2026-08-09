/// Compliance Limit Checker (satellite-referenced)
///
/// NOTE: Despite the satellite references, THIS TOOL performs NO atmospheric
/// correction, NO background subtraction, NO plume fitting, and NO emission
/// quantification. It is a limit-checker: it compares a caller-supplied
/// measured_value against a regulatory_limit and reports exceedance + sanctions.
/// The "satellite_source" string is only echoed as a label; no satellite data is
/// retrieved or processed. The plume-fitting evidence chain shown below is the
/// methodology described in the references, NOT something this tool executes.
///
/// Literature Reference (NOT this tool's performance):
///   ESA/Copernicus TROPOMI plume-fitting & emission quantification (real method).
///   PP 22/2021; Permen LH 6/2026 (Indonesian regulatory framework).
///   Those pipelines require external satellite processing not done here.
pub fn assess(facility_name: &str, lat: f64, lon: f64, parameter: &str, measured_value: f64, regulatory_limit: f64, satellite_source: &str) -> String {
    let mut out = String::from("=== Compliance Limit Checker (satellite-referenced) ===\n");
    out.push_str("NOTE: No satellite/plume-fitting runs here — this is a limit comparison only.\n");
    out.push_str("Literature Reference (NOT this tool's performance):\n");
    out.push_str("  ESA TROPOMI plume fitting; PP 22/2021; Permen LH 6/2026\n\n");
    let sensor = match satellite_source.to_lowercase().as_str() {
        s if s.contains("tropomi") || s.contains("sentinel-5p") || s.contains("s5p") => "Sentinel-5P TROPOMI (5.5×3.5km, daily)",
        s if s.contains("sentinel-2") || s.contains("s2") => "Sentinel-2 MSI (10m, 5-day)",
        s if s.contains("landsat") => "Landsat 8/9 (30m, 8-day)",
        s if s.contains("modis") => "MODIS (250-1000m, daily)",
        s if s.contains("sentinel-1") || s.contains("s1") || s.contains("sar") => "Sentinel-1 SAR (10m, 6-12day)",
        _ => "Multi-sensor",
    };
    let compliant = measured_value <= regulatory_limit;
    let exceedance_pct = if regulatory_limit > 0.0 { (measured_value / regulatory_limit - 1.0) * 100.0 } else { 0.0 };
    let evidence_strength = if (measured_value - regulatory_limit).abs() / regulatory_limit > 0.2 {"Strong"} else {"Marginal"};
    out.push_str(&format!("Facility: {} ({:.4}, {:.4})\n", facility_name, lat, lon));
    out.push_str(&format!("Parameter: {}, Satellite: {}\n", parameter, sensor));
    out.push_str(&format!("Measured: {:.4}, Limit: {:.4}\n\n", measured_value, regulatory_limit));
    out.push_str("-- Compliance Assessment --\n\n");
    out.push_str(&format!("  Status: {}\n", if compliant {"✅ COMPLIANT"} else {"❌ VIOLATION DETECTED"}));
    if !compliant {
        out.push_str(&format!("  Exceedance: {:.1}%\n", exceedance_pct));
        out.push_str(&format!("  Evidence strength: {}\n\n", evidence_strength));
    } else {
        out.push('\n');
    }
    out.push_str("-- Sanki Berbasis Risiko (Permen LH 6/2026) --\n");
    if !compliant {
        out.push_str("  1. Teguran tertulis (30 hari perbaikan)\n");
        out.push_str("  2. Paksaan pemerintah (stop produksi)\n");
        out.push_str(&format!("  3. Denda: {:.0} × {:.4} × durasi (max Rp3M)\n", measured_value, regulatory_limit));
        out.push_str("  4. Pencabutan izin\n\n");
    } else {
        out.push_str("  No sanksi — maintain compliance\n\n");
    }
    out.push_str("-- Evidence Chain (reference methodology; NOT executed by this tool) --\n");
    out.push_str("  Satellite data → atmospheric correction → background subtraction\n");
    out.push_str("  → plume fitting → emission quantification → legal evidence\n");
    out.push_str("  (This tool only compares measured_value vs limit; chain above is external.)\n\n");
    out.push_str("-- DATA ACCESS --\n");
    out.push_str("  Copernicus Open Access Hub (free)\n");
    out.push_str("  GEE (Google Earth Engine) for processing\n");
    out.push_str("  Ref: ESA; PP 22/2021; Permen LH 6/2026\n");
    out
}
