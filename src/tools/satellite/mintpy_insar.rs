/// MintPy InSAR — SBAS/PSI Time Series Displacement
///
/// This tool emits the ISCE2/MintPy command sequence and reports published
/// subsidence rates. It does NOT run InSAR processing itself.
///
/// Verified references:
///   Yunjun et al. 2019 (MintPy v1); Berardino et al. 2002 (SBAS)
///   Umarhadi & Siegert 2026, Environ. Res. Commun. 8:025025,
///     DOI 10.1088/2515-7620/ae43ab — SBAS InSAR + Random Forest Regression
///     over the ex-Mega Rice Project peatlands, Central Kalimantan
///
/// Python package: mintpy (SBAS processing Sentinel-1 SLC → displacement)
use std::process::Command;

pub fn assess(
    lat: f64, lon: f64,
    date_start: &str, date_end: &str,
    bbox_km: f64,
) -> String {
    let mut out = String::from("=== MintPy InSAR (SBAS Displacement) ===\n");
    out.push_str("Ref: Yunjun et al. 2019 (MintPy); Berardino et al. 2002 (SBAS)\n");
    out.push_str("     Umarhadi & Siegert 2026, DOI 10.1088/2515-7620/ae43ab (gambut Kalimantan)\n");
    out.push_str("CATATAN: tool ini menghasilkan urutan perintah dan angka pustaka;\n");
    out.push_str("         pemrosesan InSAR tidak dijalankan di sini.\n\n");

    // Convert lat/lon + bbox to geographic bounds
    let half_lat = bbox_km / 111.0 / 2.0;
    let half_lon = bbox_km / (111.0 * lat.to_radians().cos()) / 2.0;
    let south = lat - half_lat;
    let north = lat + half_lat;
    let west = lon - half_lon;
    let east = lon + half_lon;

    out.push_str(&format!("Center: ({:.4}, {:.4})\n", lat, lon));
    out.push_str(&format!("BBox: {:.1}km → S={:.4} N={:.4} W={:.4} E={:.4}\n", bbox_km, south, north, west, east));
    out.push_str(&format!("Date range: {} to {}\n\n", date_start, date_end));

    // Check if mintpy is installed
    let check = Command::new("python3")
        .arg("-c")
        .arg("import mintpy; print(mintpy.__version__)")
        .output();

    match check {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            out.push_str(&format!("MintPy version: {}\n\n", ver));

            // Generate processing script
            out.push_str("═══ SBAS PROCESSING PIPELINE ═══\n\n");
            out.push_str("Step 1: Download Sentinel-1 SLC scenes\n");
            out.push_str(&format!("  Area: S={:.4} N={:.4} W={:.4} E={:.4}\n", south, north, west, east));
            out.push_str(&format!("  Date: {} to {}\n", date_start, date_end));
            out.push_str("  Source: Copernicus Open Access Hub / ASF Vertex\n");
            let year_start: u32 = date_start.split('-').next().unwrap_or("2024").parse().unwrap_or(2024);
            let year_end: u32 = date_end.split('-').next().unwrap_or("2025").parse().unwrap_or(2025);
            let scene_count = (year_end.saturating_sub(year_start) * 30) as i32;
            out.push_str(&format!("  Scene count: ~{} (12-day repeat)\n\n", scene_count));

            out.push_str("Step 2: Coregistration + Interferogram stack\n");
            out.push_str("  Tool: ISCE2 / GAMMA / SNAP\n");
            out.push_str("  Perpendicular baseline < 200m\n\n");

            out.push_str("Step 3: MintPy SBAS time series\n");
            out.push_str("  smallbaselineApp.py --generate-config\n");
            out.push_str(&format!("  mintpy.select.subset.lalo = [{:.4}:{:.4}:{:.4}:{:.4}]\n", south, north, west, east));
            out.push_str(&format!("  mintpy.compute.cluster = local\n"));
            out.push_str(&format!("  mintpy.compute.numWorker = 4\n\n"));

            out.push_str("Step 4: Quality assessment\n");
            out.push_str("  Average spatial coherence > 0.3\n");
            out.push_str("  Residual RMS < 2mm (sub-million precision)\n\n");

            out.push_str("═══ EXPECTED OUTPUTS ═══\n\n");
            out.push_str("  velocity.png    — Displacement rate map (mm/yr)\n");
            out.push_str("  timeseries.h5  — Time series displacement\n");
            out.push_str("  temporalCoherence.png — Quality map\n\n");

            // Attempt to run mintpy (if data exists)
            let script = "
import sys, os, json
try:
    from mintpy import view, tsview
    from osgeo import gdal
except ImportError:
    print(json.dumps({'error': 'mintpy or gdal not fully installed'}))
    sys.exit(0)

# Check for existing velocity file
paths = ['/tmp/mintpy_run/velocity.h5', '/tmp/mintpy_run/geo_velocity.h5']
for p in paths:
    if os.path.exists(p):
        from mintpy.utils import readfile
        ds, atr = readfile.read(p)
        result = {
            'velocity_file': p,
            'mean_velocity_mm_yr': float(ds.mean()),
            'max_velocity_mm_yr': float(ds.max()),
            'min_velocity_mm_yr': float(ds.min()),
            'std_velocity_mm_yr': float(ds.std()),
            'shape': list(ds.shape),
        }
        print(json.dumps(result))
        sys.exit(0)

print(json.dumps({'error': 'No velocity file found. Run SBAS pipeline first.'}))
";

            let run = Command::new("python3")
                .arg("-c").arg(script)
                .output();

            if let Ok(o) = run {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if v.get("error").is_none() {
                        let mean = v.get("mean_velocity_mm_yr").and_then(|x| x.as_f64()).unwrap_or(0.0);
                        let max_v = v.get("max_velocity_mm_yr").and_then(|x| x.as_f64()).unwrap_or(0.0);
                        let min_v = v.get("min_velocity_mm_yr").and_then(|x| x.as_f64()).unwrap_or(0.0);
                        let std_v = v.get("std_velocity_mm_yr").and_then(|x| x.as_f64()).unwrap_or(0.0);

                        out.push_str("═══ DISPLACEMENT RESULTS ═══\n\n");
                        out.push_str(&format!("  Mean displacement rate:  {:.1} mm/yr\n", mean));
                        out.push_str(&format!("  Max (uplift):            {:.1} mm/yr\n", max_v));
                        out.push_str(&format!("  Min (subsidence):        {:.1} mm/yr\n", min_v));
                        out.push_str(&format!("  Std (variability):       {:.1} mm/yr\n\n", std_v));

                        if min_v < -50.0 {
                            out.push_str("  🔴 Severe subsidence (>50mm/yr) — Critical!\n");
                        } else if min_v < -20.0 {
                            out.push_str("  🟠 Significant subsidence (20-50mm/yr)\n");
                        } else if min_v < -10.0 {
                            out.push_str("  🟡 Moderate subsidence (10-20mm/yr)\n");
                        } else {
                            out.push_str("  🟢 Stable (<10mm/yr)\n");
                        }

                        if max_v > 10.0 {
                            out.push_str("  ⬆️ Uplift detected (>10mm/yr) — Possible tectonic/fluid injection\n");
                        }
                    }
                }
            }

            out.push_str("\n── Limitations (honest) ──\n");
            out.push_str("  • SBAS requires persistent scatterers (urban/rock) — forest/peat poor\n");
            out.push_str("  • LOS only — true vertical = LOS / cos(incidence)\n");
            out.push_str("  • Atmospheric artifacts (troposphere) can mimic subsidence\n");
            out.push_str("  • Processing takes 30-60 min for full pipeline\n");
            out.push_str("  • For Indonesia: use Sentinel-1 (free) or ALOS-2 (L-band, better vegetation)\n");
        }
        _ => {
            out.push_str("⚠️ MintPy tidak terinstall.\n");
            out.push_str("Install: conda install -c conda-forge mintpy isce2\n");
            out.push_str("Or: pip install mintpy (limited)\n\n");

            // Provide manual instructions
            out.push_str("═══ MANUAL PROCESSING GUIDE ═══\n\n");
            out.push_str("1. Download Sentinel-1 SLC:\n");
            out.push_str(&format!("   Area: S={:.4} N={:.4} W={:.4} E={:.4}\n", south, north, west, east));
            out.push_str(&format!("   Date: {} to {}\n", date_start, date_end));
            out.push_str("   Source: https://search.asf.alaska.edu\n\n");
            out.push_str("2. Process with ISCE2:\n");
            out.push_str("   stackSentinel.py -s slc_dir -b 'lat_min lat_max lon_min lon_max' -d dem -a aux_cal -o orbits\n\n");
            out.push_str("3. Run MintPy:\n");
            out.push_str("   smallbaselineApp.py  # in project directory\n\n");
            out.push_str("4. View results:\n");
            out.push_str("   view.py velocity.h5  # displacement rate map\n\n");

            // Published subsidence rates for Indonesia. Every entry must be
            // traceable to a citation whose reported value it actually matches.
            //
            // The Central Kalimantan peatland entry previously read -50 mm/yr
            // attributed to Umarhadi 2026. That paper reports mean rates of
            // -1.72 +/- 1.57 cm/yr (Block B) and -1.55 +/- 2.27 cm/yr (Block C),
            // i.e. roughly -17 mm/yr. The -50 figure does not appear in it.
            out.push_str("═══ LAJU PENURUNAN TANAH TERPUBLIKASI (Indonesia) ═══\n\n");
            out.push_str("Nilai di bawah adalah angka yang DILAPORKAN dalam pustaka,\n");
            out.push_str("bukan hasil pengukuran tool ini pada AOI Anda.\n\n");

            // (label, rate mm/yr, kind, description incl. source)
            let known: [(&str, f64, &str, &str); 6] = [
                ("Jakarta", -36.0, "maksimum",
                 "Ekstraksi air tanah (Science Advances 2024, Java InSAR 2017-2023: maks 3.6 cm/yr)"),
                ("Semarang", -80.0, "maksimum",
                 "Penurunan pesisir, terkontrol sesar (Science Advances 2024: maks 8 cm/yr)"),
                ("Pekalongan", -100.0, "maksimum",
                 "Hotspot terparah di Jawa (Science Advances 2024: maks 10 cm/yr)"),
                ("Bandung", -30.0, "maksimum",
                 "Penurunan tanah, air tanah industri — sumber belum diverifikasi ulang"),
                ("Kalimantan Tengah (gambut, ex-MRP Blok B)", -17.2, "rata-rata",
                 "Konsolidasi gambut terdrainase; SBAS InSAR + Random Forest Regression. \
                  Umarhadi & Siegert 2026, Environ. Res. Commun. 8:025025, \
                  DOI 10.1088/2515-7620/ae43ab: -1.72 +/- 1.57 cm/yr"),
                ("Kalimantan Tengah (gambut, ex-MRP Blok C)", -15.5, "rata-rata",
                 "Idem, Blok C: -1.55 +/- 2.27 cm/yr (Umarhadi & Siegert 2026)"),
            ];

            for (city, rate, kind, desc) in &known {
                let indicator = if *rate < -50.0 {
                    "[SANGAT TINGGI]"
                } else if *rate < -20.0 {
                    "[TINGGI]"
                } else if *rate < -10.0 {
                    "[SEDANG]"
                } else {
                    "[RENDAH]"
                };
                out.push_str(&format!(
                    "  {} {}: {:.1} mm/yr ({})\n      {}\n",
                    indicator, city, rate, kind, desc
                ));
            }
            out.push_str(
                "\n  Perhatikan pembeda 'maksimum' vs 'rata-rata': nilai maksimum adalah\n  \
                 kecepatan titik terburuk, rata-rata adalah nilai area. Keduanya tidak\n  \
                 sebanding secara langsung.\n",
            );
            out.push_str(
                "\n  Prediktor kunci penurunan gambut menurut Umarhadi & Siegert 2026:\n  \
                 kedalaman gambut, kejadian kebakaran terakhir, dan jarak ke tepi gambut.\n",
            );
        }
    }

    out
}
