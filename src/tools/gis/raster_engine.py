#!/usr/bin/env python3
"""Raster Analysis Engine — GEE-first + local rasterio fallback
Supports: band math (NDVI/NDWI/SAVI/EVI/MNDWI/NDBI/custom), DEM analysis, zonal stats
"""
import sys, json, os
import numpy as np


def band_math_gee(lat, lon, buffer_km, index_type, start_date, end_date, output_path):
    """Compute vegetation/water indices from Sentinel-2 via GEE"""
    import ee
    ee.Initialize()

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    # Cloud Score+ (Google, 2023) — ML-based, superior to SCL for tropical regions
    csPlus = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED')
    QA_BAND = 'cs_cdf'
    CLEAR_THRESHOLD = 0.60

    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate(start_date, end_date) \
        .filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30)) \
        .linkCollection(csPlus, [QA_BAND]) \
        .map(lambda img: img.updateMask(img.select(QA_BAND).gte(CLEAR_THRESHOLD))) \
        .median()

    # Compute index
    indices = {
        'ndvi': s2.normalizedDifference(['B8', 'B4']),
        'ndwi': s2.normalizedDifference(['B3', 'B8']),
        'mndwi': s2.normalizedDifference(['B3', 'B11']),
        'savi': s2.expression('((NIR-RED)/(NIR+RED+5000))*1.5',
                              {'NIR': s2.select('B8'), 'RED': s2.select('B4')}),
        'evi': s2.expression('2.5*((NIR-RED)/(NIR+6*RED-7.5*BLUE+10000))',
                             {'NIR': s2.select('B8'), 'RED': s2.select('B4'),
                              'BLUE': s2.select('B2')}),
        'ndbi': s2.normalizedDifference(['B11', 'B8']),
        'bsi': s2.expression(
            '((SWIR+RED)-(NIR+BLUE))/((SWIR+RED)+(NIR+BLUE))',
            {'SWIR': s2.select('B11'), 'RED': s2.select('B4'),
             'NIR': s2.select('B8'), 'BLUE': s2.select('B2')}),
        # CMRI = NDVI - NDWI (Gupta et al. 2018). CMRI > 0 = mangrove candidate
        'cmri': s2.normalizedDifference(['B8', 'B4']).subtract(s2.normalizedDifference(['B3', 'B8'])).rename('cmri'),
    }

    idx_lower = index_type.lower()
    if idx_lower not in indices:
        print(f"ERROR: Index '{index_type}' tidak dikenal. Tersedia: {', '.join(indices.keys())}")
        return

    result = indices[idx_lower].rename(idx_lower).clip(roi)

    # Statistics
    stats = result.reduceRegion(
        reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True)
            .combine(ee.Reducer.max(), '', True).combine(ee.Reducer.stdDev(), '', True),
        geometry=roi, scale=10, maxPixels=1e9
    ).getInfo()

    # Download GeoTIFF
    url = result.getDownloadURL({
        'scale': 10, 'region': roi, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
    })
    import requests
    r = requests.get(url, timeout=60)
    tif_path = output_path.replace('.png', '.tif')
    with open(tif_path, 'wb') as f:
        f.write(r.content)

    # Visualization PNG
    import matplotlib
    matplotlib.use('Agg')

    thumb_url = result.getThumbURL({
        'region': roi, 'dimensions': 800,
        'min': -1 if idx_lower in ('ndvi', 'ndwi', 'mndwi') else -0.5,
        'max': 1 if idx_lower in ('ndvi', 'ndwi', 'mndwi') else 0.5,
        'palette': (['red', 'yellow', 'green']
                     if idx_lower in ('ndvi', 'savi', 'evi')
                     else ['brown', 'white', 'blue'])
    })

    img_data = requests.get(thumb_url, timeout=30).content
    with open(output_path, 'wb') as f:
        f.write(img_data)

    # Format stats
    mean_val = stats.get(f'{idx_lower}_mean', 'N/A')
    min_val = stats.get(f'{idx_lower}_min', 'N/A')
    max_val = stats.get(f'{idx_lower}_max', 'N/A')
    std_val = stats.get(f'{idx_lower}_stdDev', 'N/A')

    print(f"SUCCESS: {index_type.upper()} computed. Output: {output_path}")
    print(f"GeoTIFF: {tif_path}")
    if isinstance(mean_val, (int, float)):
        print(f"Stats: mean={mean_val:.4f}, min={min_val:.4f}, max={max_val:.4f}, std={std_val:.4f}")
    else:
        print(f"Stats: {stats}")
    print(f"Resolution: 10m (Sentinel-2)")
    print(f"Period: {start_date} to {end_date}")
    print(f"Cloud masking: Cloud Score+ (cs_cdf >= {CLEAR_THRESHOLD}, ML-based)")


def band_math_local(input_path, expression, output_path):
    """Compute band math on local GeoTIFF using rasterio+numpy"""
    import rasterio

    with rasterio.open(input_path) as src:
        bands = {}
        for i in range(1, src.count + 1):
            bands[f'B{i}'] = src.read(i).astype(float)

        # Safe eval with numpy
        result = eval(expression, {"__builtins__": {}, "np": np, **bands})

        profile = src.profile.copy()
        profile.update(count=1, dtype='float32', nodata=-9999)

        with rasterio.open(output_path, 'w', **profile) as dst:
            dst.write(result.astype(np.float32), 1)

    print(f"SUCCESS: Band math computed. Output: {output_path}")
    print(f"Expression: {expression}")
    print(f"Stats: mean={np.nanmean(result):.4f}, min={np.nanmin(result):.4f}, max={np.nanmax(result):.4f}")


def dem_analysis_gee(lat, lon, buffer_km, analysis_type, output_path):
    """Compute slope/aspect/hillshade from SRTM via GEE"""
    import ee
    ee.Initialize()
    import requests

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)
    srtm = ee.Image('USGS/SRTMGL1_003')

    if analysis_type == 'slope':
        result = ee.Terrain.slope(srtm).clip(roi)
        palette = ['green', 'yellow', 'orange', 'red', 'darkred']
        vis = {'min': 0, 'max': 45, 'palette': palette}
        unit = 'degrees'
    elif analysis_type == 'aspect':
        result = ee.Terrain.aspect(srtm).clip(roi)
        palette = ['red', 'yellow', 'green', 'cyan', 'blue', 'magenta', 'red']
        vis = {'min': 0, 'max': 360, 'palette': palette}
        unit = 'degrees'
    elif analysis_type == 'hillshade':
        result = ee.Terrain.hillshade(srtm).clip(roi)
        vis = {'min': 0, 'max': 255}
        palette = None
        unit = '0-255'
    else:
        print(f"ERROR: analysis_type '{analysis_type}' tidak dikenal. Gunakan: slope/aspect/hillshade")
        return

    # Stats
    stats = result.reduceRegion(
        reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True)
            .combine(ee.Reducer.max(), '', True),
        geometry=roi, scale=30, maxPixels=1e9
    ).getInfo()

    # Download GeoTIFF
    url = result.getDownloadURL({
        'scale': 30, 'region': roi, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
    })
    r = requests.get(url, timeout=60)
    tif_path = output_path.replace('.png', '.tif')
    with open(tif_path, 'wb') as f:
        f.write(r.content)

    # Thumbnail PNG
    if palette:
        vis['palette'] = palette
    thumb = result.getThumbURL({**vis, 'region': roi, 'dimensions': 800})
    img = requests.get(thumb, timeout=30).content
    with open(output_path, 'wb') as f:
        f.write(img)

    band_key = list(stats.keys())[0].rsplit('_', 1)[0] if stats else analysis_type
    mean_v = stats.get(f'{band_key}_mean',
                       stats.get('slope_mean',
                       stats.get('aspect_mean',
                       stats.get('hillshade_mean', 'N/A'))))
    min_v = stats.get(f'{band_key}_min', 'N/A')
    max_v = stats.get(f'{band_key}_max', 'N/A')

    print(f"SUCCESS: DEM {analysis_type} computed. Output: {output_path}")
    print(f"GeoTIFF: {tif_path}")
    print(f"Stats: mean={mean_v}, min={min_v}, max={max_v} ({unit})")
    print(f"DEM: SRTM 30m | CRS: EPSG:4326")


def zonal_stats_gee(lat, lon, buffer_km, image_id, band, geojson_str, output_path):
    """Zonal statistics using GEE reduceRegion"""
    import ee
    ee.Initialize()

    geojson = json.loads(geojson_str)
    if geojson.get('type') == 'FeatureCollection':
        features = [ee.Feature(ee.Geometry(f['geometry'])) for f in geojson['features']]
        fc = ee.FeatureCollection(features)
    else:
        fc = ee.FeatureCollection([ee.Feature(ee.Geometry(geojson))])

    image = ee.Image(image_id).select(band)

    stats = image.reduceRegions(
        collection=fc,
        reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True)
            .combine(ee.Reducer.max(), '', True).combine(ee.Reducer.stdDev(), '', True)
            .combine(ee.Reducer.count(), '', True).combine(ee.Reducer.sum(), '', True),
        scale=30
    ).getInfo()

    print(f"SUCCESS: Zonal statistics computed for {len(stats['features'])} zones")
    print(f"Image: {image_id} | Band: {band}")
    for i, feat in enumerate(stats['features']):
        props = feat['properties']
        print(f"\nZone {i+1}:")
        for k, v in props.items():
            if isinstance(v, (int, float)):
                print(f"  {k}: {v:.4f}" if isinstance(v, float) else f"  {k}: {v}")


def zonal_stats_local(raster_path, vector_path, stats_list):
    """Zonal statistics using rasterstats on local files"""
    from rasterstats import zonal_stats as zs
    import geopandas as gpd

    if os.path.exists(vector_path):
        gdf = gpd.read_file(vector_path)
    else:
        gdf = gpd.GeoDataFrame.from_features(json.loads(vector_path))
    results = zs(gdf, raster_path, stats=stats_list)

    print(f"SUCCESS: Local zonal statistics computed for {len(results)} zones")
    for i, r in enumerate(results):
        print(f"\nZone {i+1}:")
        for k, v in r.items():
            if v is not None:
                print(f"  {k}: {v:.4f}" if isinstance(v, float) else f"  {k}: {v}")


def topo_correction(lat, lon, buffer_km, start_date, end_date, output_path):
    """C-correction topographic normalization for Sentinel-2.
    Ref: Teillet et al. 1982. Applied only where slope > 5 degrees.
    """
    import ee, math, requests
    ee.Initialize()

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    # S2 composite (single image for consistent solar angle)
    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate(start_date, end_date).filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 20)) \
        .sort('CLOUDY_PIXEL_PERCENTAGE').first()

    if s2 is None:
        print("ERROR: No S2 image found for this period")
        return

    # Solar angles from metadata
    solar_zenith = ee.Number(s2.get('MEAN_SOLAR_ZENITH_ANGLE'))
    solar_azimuth = ee.Number(s2.get('MEAN_SOLAR_AZIMUTH_ANGLE'))

    deg2rad = ee.Number(math.pi / 180)
    theta_s = solar_zenith.multiply(deg2rad)
    phi_s = solar_azimuth.multiply(deg2rad)

    # DEM terrain
    dem = ee.Image('USGS/SRTMGL1_003')
    slope_rad = ee.Terrain.slope(dem).multiply(deg2rad)
    aspect_rad = ee.Terrain.aspect(dem).multiply(deg2rad)

    # cos(i) = illumination angle
    cos_i = (slope_rad.cos().multiply(theta_s.cos())
             .add(slope_rad.sin().multiply(theta_s.sin())
                  .multiply(phi_s.subtract(aspect_rad).cos())))
    cos_i = cos_i.rename('cos_i')

    # Mask self-shadowed and flat areas
    cos_i = cos_i.updateMask(cos_i.gt(0))
    slope_mask = ee.Terrain.slope(dem).gt(5)  # only correct slopes > 5 deg

    cos_theta_s = ee.Image.constant(theta_s.cos())

    # C-correction per band
    bands_to_correct = ['B2', 'B3', 'B4', 'B8', 'B11', 'B12']
    corrected_bands = []

    for band in bands_to_correct:
        # Regression: reflectance = a + b * cos(i)
        regression = cos_i.addBands(s2.select(band)).reduceRegion(
            reducer=ee.Reducer.linearFit(),
            geometry=roi, scale=30, maxPixels=1e6, bestEffort=True
        )

        b_coeff = ee.Number(regression.get('scale'))
        a_coeff = ee.Number(regression.get('offset'))
        c_coeff = a_coeff.divide(b_coeff).max(0)  # clamp C >= 0

        # L_corrected = L * (cos(theta_s) + C) / (cos(i) + C)
        corrected = s2.select(band) \
            .multiply(cos_theta_s.add(c_coeff)) \
            .divide(cos_i.add(c_coeff)) \
            .rename(band)

        # Only apply where slope > 5 degrees
        corrected = corrected.where(slope_mask.Not(), s2.select(band))
        corrected_bands.append(corrected)

    result = ee.Image(corrected_bands).clip(roi)

    # Download
    url = result.getDownloadURL({'scale': 10, 'region': roi, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'})
    r = requests.get(url, timeout=120)
    tif_path = output_path.replace('.png', '.tif')
    with open(tif_path, 'wb') as f:
        f.write(r.content)

    print(f"SUCCESS: Topographic C-correction applied. Output: {tif_path}")
    print(f"Bands corrected: {', '.join(bands_to_correct)}")
    print(f"Solar zenith: {solar_zenith.getInfo():.1f}\u00b0")
    print(f"Correction applied only where slope > 5\u00b0")
    print(f"Ref: Teillet et al. 1982")


def ndvi_timeseries(lat, lon, buffer_km, start_year, end_year, output_path):
    """NDVI annual trend analysis. Ref: Saifulloh et al. 2025"""
    import ee, requests
    ee.Initialize()

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    # Annual NDVI composites
    years = list(range(start_year, end_year + 1))
    annual_ndvi = []

    for year in years:
        s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
            .filterDate(f'{year}-01-01', f'{year}-12-31') \
            .filterBounds(roi) \
            .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30)) \
            .median()
        ndvi = s2.normalizedDifference(['B8', 'B4']).rename(f'NDVI_{year}')
        annual_ndvi.append(ndvi)

    # Stack and compute linear trend
    stack = ee.ImageCollection(annual_ndvi)

    # Add time band for regression
    def add_time(img):
        year = ee.Number.parse(img.bandNames().get(0).slice(-4))
        return img.addBands(ee.Image.constant(year).float().rename('year'))

    stack_with_time = stack.map(add_time)

    # Linear fit: NDVI = a + b * year
    trend = stack_with_time.select(['year', stack.first().bandNames().get(0)]) \
        .reduce(ee.Reducer.linearFit())

    slope = trend.select('scale').clip(roi)  # NDVI change per year

    # Stats
    stats = slope.reduceRegion(
        reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True).combine(ee.Reducer.max(), '', True),
        geometry=roi, scale=30, maxPixels=1e9
    ).getInfo()

    # Thumbnail
    thumb = slope.getThumbURL({
        'region': roi, 'dimensions': 800,
        'min': -0.05, 'max': 0.05,
        'palette': ['red', 'white', 'green']
    })
    img_data = requests.get(thumb, timeout=30).content
    with open(output_path, 'wb') as f:
        f.write(img_data)

    print(f"SUCCESS: NDVI Time Series Trend ({start_year}-{end_year}). Output: {output_path}")
    print(f"Trend unit: NDVI change per year")
    print(f"Mean slope: {stats.get('scale_mean', 'N/A')}")
    print(f"Negative = vegetation loss (red) | Positive = vegetation gain (green)")
    print(f"Ref: Saifulloh et al. 2025")


# CLI dispatcher
if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("ERROR: Usage: raster_engine.py <mode> [args...]")
        print("Modes: band_math_gee, band_math_local, dem_slope, dem_aspect, dem_hillshade, zonal_gee, zonal_local")
        sys.exit(1)

    mode = sys.argv[1]
    try:
        if mode == 'band_math_gee':
            # args: lat lon buffer_km index_type start_date end_date output_path
            band_math_gee(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                          sys.argv[5], sys.argv[6], sys.argv[7], sys.argv[8])
        elif mode == 'band_math_local':
            # args: input_path expression output_path
            band_math_local(sys.argv[2], sys.argv[3], sys.argv[4])
        elif mode in ('dem_slope', 'dem_aspect', 'dem_hillshade'):
            # args: lat lon buffer_km output_path
            dem_analysis_gee(float(sys.argv[2]), float(sys.argv[3]),
                             float(sys.argv[4]), mode.replace('dem_', ''), sys.argv[5])
        elif mode == 'zonal_gee':
            # args: lat lon buffer_km image_id band geojson_str output_path
            zonal_stats_gee(float(sys.argv[2]), float(sys.argv[3]),
                            float(sys.argv[4]), sys.argv[5], sys.argv[6],
                            sys.argv[7], sys.argv[8])
        elif mode == 'zonal_local':
            # args: raster_path vector_path stats_list(comma-sep)
            zonal_stats_local(sys.argv[2], sys.argv[3], sys.argv[4].split(','))
        elif mode == 'topo_correct':
            # args: lat lon buffer_km start_date end_date output_path
            topo_correction(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                            sys.argv[5], sys.argv[6], sys.argv[7])
        elif mode == 'ndvi_timeseries':
            # args: lat lon buffer_km start_year end_year output_path
            ndvi_timeseries(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                            int(sys.argv[5]), int(sys.argv[6]), sys.argv[7])
        else:
            print(f"ERROR: Unknown mode '{mode}'")
    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc(file=sys.stderr)
