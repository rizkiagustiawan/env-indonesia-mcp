use std::process::Command;

pub fn info(path: &str) -> String {
    match Command::new("gdalinfo").arg(path).arg("-json").output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout);
            if out.is_empty() {
                format!("ERROR: gdalinfo gagal untuk {}", path)
            } else {
                format!("=== GeoTIFF Info ===\n{}", &out[..out.len().min(4000)])
            }
        }
        Err(e) => format!("ERROR: gdalinfo tidak ditemukan: {}", e),
    }
}

pub fn crop(input: &str, output: &str, bbox: &str) -> String {
    // bbox format: "minlon minlat maxlon maxlat"
    let parts: Vec<&str> = bbox.split_whitespace().collect();
    if parts.len() != 4 {
        return "ERROR: bbox harus 4 angka: minlon minlat maxlon maxlat".into();
    }

    match Command::new("gdalwarp")
        .args(&["-te", parts[0], parts[1], parts[2], parts[3]])
        .args(&["-overwrite", input, output])
        .output()
    {
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            if o.status.success() {
                format!("SUCCESS: Crop selesai → {}", output)
            } else {
                format!("ERROR: {}", err)
            }
        }
        Err(e) => format!("ERROR: gdalwarp: {}", e),
    }
}

pub fn reproject(input: &str, output: &str, target_crs: &str) -> String {
    match Command::new("gdalwarp")
        .args(&["-t_srs", target_crs, "-overwrite", input, output])
        .output()
    {
        Ok(o) => {
            if o.status.success() {
                format!("SUCCESS: Reproject ke {} → {}", target_crs, output)
            } else {
                format!("ERROR: {}", String::from_utf8_lossy(&o.stderr))
            }
        }
        Err(e) => format!("ERROR: {}", e),
    }
}
