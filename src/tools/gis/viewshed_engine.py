#!/usr/bin/env python3
"""Viewshed Analysis Engine — local DEM-based line-of-sight
Ray-casting from observer point to each cell within max distance
"""
import sys, json, os
import numpy as np


def viewshed(dem_path, obs_lat, obs_lon, obs_height, max_distance, output_path):
    """Compute viewshed from observer location on DEM raster

    Args:
        dem_path: Path to DEM GeoTIFF
        obs_lat, obs_lon: Observer coordinates (WGS84)
        obs_height: Observer height above ground (meters)
        max_distance: Maximum view distance (meters)
        output_path: Output path for visibility raster PNG
    """
    import rasterio
    from rasterio.transform import rowcol
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt

    # Read DEM
    with rasterio.open(dem_path) as src:
        dem = src.read(1).astype(np.float64)
        transform = src.transform
        crs = src.crs
        profile = src.profile.copy()
        nodata = src.nodata
        nrows, ncols = dem.shape

    # Handle nodata
    if nodata is not None:
        dem[dem == nodata] = np.nan

    # Observer pixel location
    obs_row, obs_col = rowcol(transform, obs_lon, obs_lat)
    if obs_row < 0 or obs_row >= nrows or obs_col < 0 or obs_col >= ncols:
        print(f"ERROR: Observer ({obs_lat}, {obs_lon}) diluar batas DEM")
        return

    obs_elev = dem[obs_row, obs_col]
    if np.isnan(obs_elev):
        print(f"ERROR: Observer berada pada pixel nodata")
        return

    obs_total_elev = obs_elev + obs_height

    # Pixel resolution in meters (approximate for geographic CRS)
    pixel_size_x = abs(transform.a)
    pixel_size_y = abs(transform.e)

    # Convert degrees to meters (approximate)
    if crs and crs.is_geographic:
        lat_rad = np.radians(obs_lat)
        m_per_deg_lat = 111320.0
        m_per_deg_lon = 111320.0 * np.cos(lat_rad)
        res_x_m = pixel_size_x * m_per_deg_lon
        res_y_m = pixel_size_y * m_per_deg_lat
    else:
        res_x_m = pixel_size_x
        res_y_m = pixel_size_y

    avg_res_m = (res_x_m + res_y_m) / 2.0

    # Max distance in pixels
    max_dist_px = int(max_distance / avg_res_m)

    # Visibility array: 1=visible, 0=not visible
    visibility = np.zeros((nrows, ncols), dtype=np.uint8)
    visibility[obs_row, obs_col] = 1  # Observer always visible

    # Earth curvature + atmospheric refraction correction
    # Reference: standard geodetic surveying, k=0.13 (atmospheric refraction coefficient)
    EARTH_RADIUS = 6_371_000  # meters
    REFRACTION_K = 0.13

    # Determine bounding box of analysis
    r_min = max(0, obs_row - max_dist_px)
    r_max = min(nrows - 1, obs_row + max_dist_px)
    c_min = max(0, obs_col - max_dist_px)
    c_max = min(ncols - 1, obs_col + max_dist_px)

    # Ray-casting: for each target cell, trace line-of-sight
    for tr in range(r_min, r_max + 1):
        for tc in range(c_min, c_max + 1):
            if tr == obs_row and tc == obs_col:
                continue

            # Distance check
            dr = tr - obs_row
            dc = tc - obs_col
            dist_px = np.sqrt(dr * dr + dc * dc)
            dist_m = dist_px * avg_res_m

            if dist_m > max_distance:
                continue

            target_elev = dem[tr, tc]
            if np.isnan(target_elev):
                continue

            # Apply earth curvature + refraction correction to target
            curvature_correction_target = (dist_m ** 2) / (2 * EARTH_RADIUS) * (1 - REFRACTION_K)
            adjusted_target_elev = target_elev - curvature_correction_target

            # Angle from observer to target (curvature-corrected)
            target_angle = np.arctan2(adjusted_target_elev - obs_total_elev, dist_m)

            # Trace ray: check intermediate cells using Bresenham-like stepping
            n_steps = max(abs(dr), abs(dc))
            is_visible = True

            for step in range(1, n_steps):
                frac = step / n_steps
                ir = int(obs_row + dr * frac)
                ic = int(obs_col + dc * frac)

                if ir < 0 or ir >= nrows or ic < 0 or ic >= ncols:
                    is_visible = False
                    break

                inter_elev = dem[ir, ic]
                if np.isnan(inter_elev):
                    continue

                inter_dist_m = np.sqrt((ir - obs_row) ** 2 + (ic - obs_col) ** 2) * avg_res_m
                if inter_dist_m < 1e-6:
                    continue

                # Apply earth curvature + refraction correction to intermediate point
                curvature_correction_inter = (inter_dist_m ** 2) / (2 * EARTH_RADIUS) * (1 - REFRACTION_K)
                adjusted_inter_elev = inter_elev - curvature_correction_inter

                inter_angle = np.arctan2(adjusted_inter_elev - obs_total_elev, inter_dist_m)

                if inter_angle > target_angle:
                    is_visible = False
                    break

            if is_visible:
                visibility[tr, tc] = 1

    # Count stats
    total_cells = (r_max - r_min + 1) * (c_max - c_min + 1)
    visible_cells = int(visibility.sum())
    visible_area_m2 = visible_cells * res_x_m * res_y_m
    visible_area_ha = visible_area_m2 / 1e4

    # Save visibility GeoTIFF
    tif_path = output_path.replace('.png', '.tif')
    profile.update(count=1, dtype='uint8', nodata=0)
    with rasterio.open(tif_path, 'w', **profile) as dst:
        dst.write(visibility, 1)

    # PNG visualization
    fig, axes = plt.subplots(1, 2, figsize=(16, 8))

    # Left: DEM with observer
    im0 = axes[0].imshow(dem, cmap='terrain',
                         extent=[transform.c, transform.c + transform.a * ncols,
                                 transform.f + transform.e * nrows, transform.f])
    axes[0].plot(obs_lon, obs_lat, 'r*', markersize=15, label=f'Observer ({obs_height}m)')
    axes[0].legend()
    axes[0].set_title(f'DEM (elev: {np.nanmin(dem):.0f}-{np.nanmax(dem):.0f}m)')
    plt.colorbar(im0, ax=axes[0], label='Elevation (m)')

    # Right: Viewshed
    vis_display = np.ma.masked_where(visibility == 0, visibility)
    axes[1].imshow(dem, cmap='Greys', alpha=0.5,
                   extent=[transform.c, transform.c + transform.a * ncols,
                           transform.f + transform.e * nrows, transform.f])
    axes[1].imshow(vis_display, cmap='Greens', alpha=0.6,
                   extent=[transform.c, transform.c + transform.a * ncols,
                           transform.f + transform.e * nrows, transform.f])
    axes[1].plot(obs_lon, obs_lat, 'r*', markersize=15)
    axes[1].set_title(f'Viewshed (visible: {visible_cells} cells)')

    plt.suptitle(f'Viewshed Analysis — max distance: {max_distance}m')
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight')
    plt.close()

    print(f"SUCCESS: Viewshed analysis completed. Output: {output_path}")
    print(f"GeoTIFF: {tif_path}")
    print(f"Observer: ({obs_lat}, {obs_lon}) at {obs_height}m above ground")
    print(f"Observer elevation: {obs_elev:.1f}m + {obs_height}m = {obs_total_elev:.1f}m")
    print(f"Max distance: {max_distance}m ({max_dist_px} pixels)")
    print(f"DEM resolution: ~{avg_res_m:.1f}m | Size: {ncols}x{nrows}")
    print(f"Visible cells: {visible_cells} / {total_cells} ({visible_cells/total_cells*100:.1f}%)")
    print(f"Visible area: {visible_area_ha:.2f} ha ({visible_area_m2/1e6:.4f} km2)")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("ERROR: Usage: viewshed_engine.py <dem_path> <lat> <lon> <observer_height_m> <max_distance_m> <output_path>")
        sys.exit(1)

    try:
        viewshed(
            dem_path=sys.argv[1],
            obs_lat=float(sys.argv[2]),
            obs_lon=float(sys.argv[3]),
            obs_height=float(sys.argv[4]),
            max_distance=float(sys.argv[5]),
            output_path=sys.argv[6]
        )
    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc(file=sys.stderr)
