#!/usr/bin/env python3
"""Spatial Analysis Engine — buffer, overlay, suitability
Uses geopandas for vector ops, GEE for raster suitability analysis
"""
import sys, json, os
import numpy as np


def buffer_analysis(geojson_str, distance_m, output_path):
    """Buffer GeoJSON geometry by distance in meters"""
    import geopandas as gpd
    from shapely.geometry import shape, mapping
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt

    geojson = json.loads(geojson_str)

    # Build GeoDataFrame
    if geojson.get('type') == 'FeatureCollection':
        gdf = gpd.GeoDataFrame.from_features(geojson['features'], crs='EPSG:4326')
    elif geojson.get('type') == 'Feature':
        gdf = gpd.GeoDataFrame.from_features([geojson], crs='EPSG:4326')
    else:
        gdf = gpd.GeoDataFrame(geometry=[shape(geojson)], crs='EPSG:4326')

    # Estimate UTM zone from centroid
    centroid = gdf.geometry.union_all().centroid
    utm_zone = int((centroid.x + 180) / 6) + 1
    hemisphere = 'north' if centroid.y >= 0 else 'south'
    epsg_utm = 32600 + utm_zone if hemisphere == 'north' else 32700 + utm_zone

    # Reproject to UTM, buffer, reproject back
    gdf_utm = gdf.to_crs(epsg=epsg_utm)
    buffered_utm = gdf_utm.buffer(distance_m)
    buffered_gdf = gpd.GeoDataFrame(geometry=buffered_utm, crs=f'EPSG:{epsg_utm}')
    buffered_wgs = buffered_gdf.to_crs(epsg=4326)

    # Compute area in UTM
    area_m2 = buffered_utm.area.sum()
    area_ha = area_m2 / 1e4
    area_km2 = area_m2 / 1e6

    # Save GeoJSON output
    geojson_out = output_path.replace('.png', '.geojson')
    buffered_wgs.to_file(geojson_out, driver='GeoJSON')

    # PNG visualization
    fig, ax = plt.subplots(1, 1, figsize=(10, 10))
    buffered_wgs.plot(ax=ax, alpha=0.3, color='blue', edgecolor='blue', linewidth=2,
                      label=f'Buffer {distance_m}m')
    gdf.plot(ax=ax, color='red', edgecolor='red', linewidth=2, label='Original')

    try:
        import contextily as ctx
        buffered_web = buffered_wgs.to_crs(epsg=3857)
        gdf_web = gdf.to_crs(epsg=3857)
        ax.clear()
        buffered_web.plot(ax=ax, alpha=0.3, color='blue', edgecolor='blue', linewidth=2,
                          label=f'Buffer {distance_m}m')
        gdf_web.plot(ax=ax, color='red', edgecolor='red', linewidth=2, label='Original')
        ctx.add_basemap(ax, source=ctx.providers.OpenStreetMap.Mapnik)
    except Exception:
        pass  # no basemap fallback

    ax.legend()
    ax.set_title(f'Buffer Analysis ({distance_m}m)')
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight')
    plt.close()

    print(f"SUCCESS: Buffer analysis completed. Output: {output_path}")
    print(f"GeoJSON: {geojson_out}")
    print(f"Buffer distance: {distance_m}m")
    print(f"Area: {area_ha:.2f} ha ({area_km2:.4f} km2)")
    print(f"CRS: UTM Zone {utm_zone}{hemisphere[0].upper()} (EPSG:{epsg_utm}) → WGS84")
    print(f"Features: {len(gdf)} input → {len(buffered_wgs)} buffered")


def overlay_analysis(geojson1_str, geojson2_str, operation, output_path):
    """Overlay 2 GeoJSON layers: intersection/union/difference/symmetric_difference"""
    import geopandas as gpd
    from shapely.geometry import shape
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt

    def load_gdf(gj_str):
        gj = json.loads(gj_str)
        if gj.get('type') == 'FeatureCollection':
            return gpd.GeoDataFrame.from_features(gj['features'], crs='EPSG:4326')
        elif gj.get('type') == 'Feature':
            return gpd.GeoDataFrame.from_features([gj], crs='EPSG:4326')
        else:
            return gpd.GeoDataFrame(geometry=[shape(gj)], crs='EPSG:4326')

    gdf1 = load_gdf(geojson1_str)
    gdf2 = load_gdf(geojson2_str)

    valid_ops = ['intersection', 'union', 'difference', 'symmetric_difference']
    if operation not in valid_ops:
        print(f"ERROR: Operation '{operation}' tidak dikenal. Gunakan: {', '.join(valid_ops)}")
        return

    # Estimate UTM for area calculation
    centroid = gdf1.geometry.union_all().centroid
    utm_zone = int((centroid.x + 180) / 6) + 1
    hemisphere = 'north' if centroid.y >= 0 else 'south'
    epsg_utm = 32600 + utm_zone if hemisphere == 'north' else 32700 + utm_zone

    # Perform overlay
    result = gpd.overlay(gdf1, gdf2, how=operation)

    if len(result) == 0:
        print(f"SUCCESS: Overlay ({operation}) produced empty result — no overlap")
        return

    # Area in UTM
    result_utm = result.to_crs(epsg=epsg_utm)
    area_m2 = result_utm.geometry.area.sum()
    area_ha = area_m2 / 1e4

    # Save GeoJSON
    geojson_out = output_path.replace('.png', '.geojson')
    result.to_file(geojson_out, driver='GeoJSON')

    # PNG visualization
    fig, axes = plt.subplots(1, 2, figsize=(16, 8))

    # Left: inputs
    gdf1.plot(ax=axes[0], alpha=0.4, color='blue', edgecolor='blue', label='Layer 1')
    gdf2.plot(ax=axes[0], alpha=0.4, color='red', edgecolor='red', label='Layer 2')
    axes[0].legend()
    axes[0].set_title('Input Layers')

    # Right: result
    result.plot(ax=axes[1], alpha=0.5, color='green', edgecolor='darkgreen')
    axes[1].set_title(f'Result: {operation}')

    try:
        import contextily as ctx
        for ax_item in axes:
            gdf1_web = gdf1.to_crs(epsg=3857)
            bounds = gdf1_web.total_bounds
            ax_item.set_xlim(bounds[0], bounds[2])
            ax_item.set_ylim(bounds[1], bounds[3])
    except Exception:
        pass

    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight')
    plt.close()

    print(f"SUCCESS: Overlay ({operation}) completed. Output: {output_path}")
    print(f"GeoJSON: {geojson_out}")
    print(f"Result features: {len(result)}")
    print(f"Result area: {area_ha:.2f} ha ({area_m2/1e6:.4f} km2)")
    print(f"Input 1: {len(gdf1)} features | Input 2: {len(gdf2)} features")


def suitability_analysis(lat, lon, buffer_km, criteria_json, output_path):
    """Weighted overlay suitability analysis using GEE raster layers
    criteria_json: [{"image_id": "...", "band": "...", "weight": 0.3,
                     "min": 0, "max": 45, "invert": false, "label": "Slope"}, ...]
    """
    import ee
    ee.Initialize()
    import requests
    import matplotlib
    matplotlib.use('Agg')

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)
    criteria = json.loads(criteria_json)

    if not criteria:
        print("ERROR: criteria list kosong")
        return

    # Validate weights sum to ~1.0
    total_weight = sum(c['weight'] for c in criteria)
    if abs(total_weight - 1.0) > 0.01:
        print(f"ERROR: Total weight harus 1.0, didapat {total_weight:.2f}")
        return

    # Normalize each criterion to 0-1, apply weight, sum
    weighted_sum = None
    for crit in criteria:
        img = ee.Image(crit['image_id']).select(crit['band']).clip(roi)
        vmin = crit.get('min', 0)
        vmax = crit.get('max', 1)

        # Normalize to 0-1
        normalized = img.subtract(vmin).divide(vmax - vmin).clamp(0, 1)

        # Invert if lower values are better
        if crit.get('invert', False):
            normalized = ee.Image(1).subtract(normalized)

        weighted = normalized.multiply(crit['weight'])

        if weighted_sum is None:
            weighted_sum = weighted
        else:
            weighted_sum = weighted_sum.add(weighted)

    suitability = weighted_sum.rename('suitability')

    # Statistics
    stats = suitability.reduceRegion(
        reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True)
            .combine(ee.Reducer.max(), '', True).combine(ee.Reducer.stdDev(), '', True),
        geometry=roi, scale=30, maxPixels=1e9
    ).getInfo()

    # Classify suitability
    classified = suitability.expression(
        "(b('suitability') >= 0.8) ? 5 : "
        "(b('suitability') >= 0.6) ? 4 : "
        "(b('suitability') >= 0.4) ? 3 : "
        "(b('suitability') >= 0.2) ? 2 : 1"
    )

    # Class areas
    pixel_area = ee.Image.pixelArea()
    class_areas = pixel_area.addBands(classified).reduceRegion(
        reducer=ee.Reducer.sum().group(groupField=1, groupName='class'),
        geometry=roi, scale=30, maxPixels=1e9
    ).getInfo()

    # GeoTIFF download
    tif_path = output_path.replace('.png', '.tif')
    url = suitability.getDownloadURL({
        'scale': 30, 'region': roi, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
    })
    r = requests.get(url, timeout=60)
    with open(tif_path, 'wb') as f:
        f.write(r.content)

    # Thumbnail PNG
    thumb_url = suitability.getThumbURL({
        'region': roi, 'dimensions': 800,
        'min': 0, 'max': 1,
        'palette': ['red', 'orange', 'yellow', 'lightgreen', 'darkgreen']
    })
    img_data = requests.get(thumb_url, timeout=30).content
    with open(output_path, 'wb') as f:
        f.write(img_data)

    # Output
    mean_v = stats.get('suitability_mean', 'N/A')
    min_v = stats.get('suitability_min', 'N/A')
    max_v = stats.get('suitability_max', 'N/A')
    std_v = stats.get('suitability_stdDev', 'N/A')

    print(f"SUCCESS: Suitability analysis completed. Output: {output_path}")
    print(f"GeoTIFF: {tif_path}")
    if isinstance(mean_v, (int, float)):
        print(f"Stats: mean={mean_v:.4f}, min={min_v:.4f}, max={max_v:.4f}, std={std_v:.4f}")
    else:
        print(f"Stats: {stats}")

    print(f"\nCriteria used ({len(criteria)}):")
    for c in criteria:
        inv = " (inverted)" if c.get('invert') else ""
        print(f"  - {c.get('label', c['band'])}: weight={c['weight']}, range=[{c.get('min',0)}-{c.get('max',1)}]{inv}")

    suitability_labels = {5: 'Sangat Sesuai (S1)', 4: 'Sesuai (S2)', 3: 'Cukup Sesuai (S3)',
                          2: 'Kurang Sesuai (N1)', 1: 'Tidak Sesuai (N2)'}
    total_area = sum(g['sum'] for g in class_areas.get('groups', []))
    print(f"\nKelas Kesesuaian:")
    print(f"{'Kelas':<25} {'Area (ha)':<12} {'%':<6}")
    print("-" * 45)
    for g in sorted(class_areas.get('groups', []), key=lambda x: x['class'], reverse=True):
        cls = g['class']
        area_ha = g['sum'] / 1e4
        pct = (g['sum'] / total_area * 100) if total_area > 0 else 0
        label = suitability_labels.get(cls, f'Class {cls}')
        print(f"{label:<25} {area_ha:<12.1f} {pct:<6.1f}")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("ERROR: Usage: spatial_engine.py <mode> [args...]")
        print("Modes: buffer, overlay, suitability")
        sys.exit(1)

    mode = sys.argv[1]
    try:
        if mode == 'buffer':
            # args: geojson_str distance_m output_path
            buffer_analysis(sys.argv[2], float(sys.argv[3]), sys.argv[4])
        elif mode == 'overlay':
            # args: geojson1_str geojson2_str operation output_path
            overlay_analysis(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5])
        elif mode == 'suitability':
            # args: lat lon buffer_km criteria_json output_path
            suitability_analysis(float(sys.argv[2]), float(sys.argv[3]),
                                 float(sys.argv[4]), sys.argv[5], sys.argv[6])
        else:
            print(f"ERROR: Unknown mode '{mode}'")
    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc(file=sys.stderr)
