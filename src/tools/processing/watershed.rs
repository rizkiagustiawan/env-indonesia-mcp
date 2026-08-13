use std::process::Command;

pub fn delineate(dem_path: &str, pour_x: f64, pour_y: f64, output_path: &str) -> String {
    let script = format!(
        r#"
import sys
try:
    from pysheds.grid import Grid
    grid = Grid.from_raster('{}')
    dem = grid.read_raster('{}')
    grid.fill_depressions(dem, out_name='flooded')
    grid.resolve_flats('flooded', out_name='inflated')
    grid.flowdir('inflated', out_name='dir')
    grid.accumulation('dir', out_name='acc')
    grid.catchment(x={}, y={}, data='dir', out_name='catch')
    catch = grid.view('catch')
    import numpy as np
    area_pixels = np.sum(catch > 0)
    res_x = abs(grid.affine[0]); res_y = abs(grid.affine[4])
    if grid.crs is not None and grid.crs.is_geographic:
        km_per_deg_lon = 111.32 * np.cos(np.radians(grid.affine.yoff))
        area_km2 = area_pixels * (res_x * km_per_deg_lon) * (res_y * 111.32)
    else:
        area_km2 = area_pixels * (res_x / 1000.0) * (res_y / 1000.0)
    print(f'SUCCESS: Watershed delineated. Area={{area_km2:.2f}} km². Output: {}')
except Exception as e:
    print(f'ERROR: {{e}}')
"#,
        dem_path, dem_path, pour_x, pour_y, output_path
    );

    match Command::new("python3").arg("-c").arg(&script).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            if out.contains("SUCCESS") {
                out
            } else {
                format!("{}. Stderr: {}", out, &err[..err.len().min(500)])
            }
        }
        Err(e) => format!("ERROR pysheds: {}", e),
    }
}
