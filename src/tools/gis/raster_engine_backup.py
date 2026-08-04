#!/usr/bin/env python3
"""Raster Analysis Engine — GEE-first + local rasterio fallback
Supports: band math (NDVI/NDWI/SAVI/EVI/MNDWI/NDBI/custom), DEM analysis, zonal stats, spectral unmixing
"""
import sys, json, os
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from provenance import create_provenance


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
        # CMRI = MNDWI - NDVI (Gupta et al. 2018). CMRI > 0 = mangrove candidate
        'cmri': s2.normalizedDifference(['B3', 'B11']).subtract(s2.normalizedDifference(['B8', 'B4'])).rename('cmri'),
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
    """NDVI annual trend analysis using Mann-Kendall + Sen's slope.
    Ref: Mann 1945, Sen 1968, Kendall 1975, Saifulloh et al. 2025
    """
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
        ndvi = s2.normalizedDifference(['B8', 'B4']).rename('NDVI') \
            .set('year', year)
        annual_ndvi.append(ndvi)

    stack = ee.ImageCollection(annual_ndvi)

    # Add time band for regression
    def add_time(img):
        year = ee.Number(img.get('year'))
        return img.addBands(ee.Image.constant(year).float().rename('year'))

    stack_with_time = stack.map(add_time)

    # Mann-Kendall + Sen's slope (replaces linearFit)
    # Sen's slope: median of all pairwise slopes — robust to outliers
    # Kendall's tau: non-parametric trend significance
    trend = stack_with_time.select(['year', 'NDVI']).reduce(ee.Reducer.sensSlope())
    slope = trend.select('slope').clip(roi)  # NDVI change per year

    # Kendall's tau for significance
    kendall = stack_with_time.select(['year', 'NDVI']).reduce(ee.Reducer.kendallsCorrelation())
    tau = kendall.select('p_value').clip(roi)

    # Mask significant trends (p < 0.05)
    sig_mask = tau.lt(0.05)
    slope_sig = slope.updateMask(sig_mask)

    # Stats (full slope — all pixels)
    stats = slope.reduceRegion(
        reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True).combine(ee.Reducer.max(), '', True),
        geometry=roi, scale=30, maxPixels=1e9
    ).getInfo()

    # Significance stats
    sig_stats = slope_sig.reduceRegion(
        reducer=ee.Reducer.mean().combine(ee.Reducer.count(), '', True),
        geometry=roi, scale=30, maxPixels=1e9
    ).getInfo()

    total_pixel_count = slope.reduceRegion(
        reducer=ee.Reducer.count(), geometry=roi, scale=30, maxPixels=1e9
    ).getInfo()

    # Thumbnail (full slope)
    thumb = slope.getThumbURL({
        'region': roi, 'dimensions': 800,
        'min': -0.05, 'max': 0.05,
        'palette': ['red', 'white', 'green']
    })
    img_data = requests.get(thumb, timeout=30).content
    with open(output_path, 'wb') as f:
        f.write(img_data)

    # GeoTIFF output
    geotiff_path = output_path.replace('.png', '.tif')
    try:
        download_url = slope.getDownloadURL({
            'region': roi, 'scale': 30, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
        })
        tif_data = requests.get(download_url, timeout=120).content
        with open(geotiff_path, 'wb') as f:
            f.write(tif_data)
        print(f"GeoTIFF: {geotiff_path} ({len(tif_data)/1024:.1f} KB)")
    except Exception as e:
        print(f"GeoTIFF export gagal: {e}")

    # Significance info
    total_px = list(total_pixel_count.values())[0] if total_pixel_count else 0
    sig_px = sig_stats.get('slope_count', 0) or 0
    sig_pct = (sig_px / total_px * 100) if total_px > 0 else 0

    print(f"SUCCESS: NDVI Time Series Trend ({start_year}-{end_year}). Output: {output_path}")
    print("DISCLAIMER: Trend analysis berbasis annual median composite. Faktor non-vegetasi (cloud residual, atmospheric) dapat mempengaruhi.")
    print(f"Metode: Sen's slope (robust) + Kendall's tau (signifikansi)")
    print(f"Trend unit: NDVI change per year")
    print(f"Mean slope: {stats.get('slope_mean', 'N/A')}")
    print(f"Min slope: {stats.get('slope_min', 'N/A')} | Max slope: {stats.get('slope_max', 'N/A')}")
    print(f"Piksel signifikan (p<0.05): {sig_px:.0f}/{total_px:.0f} ({sig_pct:.1f}%)")
    print(f"Mean slope (signifikan saja): {sig_stats.get('slope_mean', 'N/A')}")
    print(f"Negatif = kehilangan vegetasi (merah) | Positif = penambahan vegetasi (hijau)")
    print(f"Ref: Mann 1945, Sen 1968, Kendall 1975, Saifulloh et al. 2025")

    # Provenance metadata
    create_provenance(output_path,
        tool='ndvi_timeseries',
        gee_collection='COPERNICUS/S2_SR_HARMONIZED',
        date_range=[f'{start_year}-01-01', f'{end_year}-12-31'],
        coordinates={'lat': lat, 'lon': lon},
        parameters={'buffer_km': buffer_km, 'start_year': start_year, 'end_year': end_year},
        algorithms=["Sen's slope (robust median pairwise)", "Kendall's tau (non-parametric significance)"],
        references=['Mann 1945', 'Sen 1968', 'Kendall 1975', 'Saifulloh et al. 2025'],
        masking='CLOUDY_PIXEL_PERCENTAGE < 30',
        crs='EPSG:4326',
        scale_m=30)


def mineral_mapping(lat, lon, buffer_km, output_path):
    """Mineral reconnaissance mapping from Sentinel-2 band ratios.
    Detects: Iron Oxide, Clay (Al-OH), Silica, Ferrous Iron, Gossan/Alteration.
    Ref: van der Meer et al. 2012, Hewson et al. 2005
    """
    import ee, requests
    ee.Initialize()

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    # S2 composite with Cloud Score+
    cs = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED') \
        .filterDate('2023-01-01', '2023-12-31').filterBounds(roi)
    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate('2023-01-01', '2023-12-31').filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30)) \
        .linkCollection(cs, ['cs_cdf']) \
        .map(lambda img: img.updateMask(img.select('cs_cdf').gte(0.60))) \
        .median().clip(roi)

    # Mineral indices
    iron_oxide = s2.select('B4').divide(s2.select('B2')).rename('iron_oxide')  # Red/Blue
    clay = s2.select('B11').divide(s2.select('B12')).rename('clay_aloh')  # SWIR1/SWIR2
    ferrous = s2.select('B12').divide(s2.select('B8')).add(
        s2.select('B3').divide(s2.select('B4'))).rename('ferrous_iron')  # SWIR2/NIR + Green/Red
    silica = s2.select('B12').divide(s2.select('B11')).rename('silica')  # SWIR2/SWIR1

    # Geological RGB composite: R=iron_oxide, G=clay, B=ferrous
    geo_rgb = iron_oxide.addBands([clay, ferrous])

    # Alteration detection (anomaly = high iron_oxide AND high clay)
    alteration = iron_oxide.gt(1.5).And(clay.gt(1.2)).rename('alteration_zone')

    # Stack all bands
    result = iron_oxide.addBands([clay, ferrous, silica, alteration.toFloat()])

    # Stats
    stats = result.reduceRegion(
        reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True)
            .combine(ee.Reducer.max(), '', True),
        geometry=roi, scale=20, maxPixels=1e9
    ).getInfo()

    # Visualization — geological false color composite
    vis = {
        'bands': ['iron_oxide', 'clay_aloh', 'ferrous_iron'],
        'min': [0.5, 0.8, 0.5],
        'max': [3.0, 2.0, 2.5],
        'region': roi, 'dimensions': 800
    }
    thumb = geo_rgb.getThumbURL(vis)
    img_data = requests.get(thumb, timeout=60).content
    with open(output_path, 'wb') as f:
        f.write(img_data)

    # GeoTIFF
    geotiff_path = output_path.replace('.png', '.tif')
    try:
        url = result.getDownloadURL({
            'region': roi, 'scale': 20, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
        })
        tif_data = requests.get(url, timeout=120).content
        with open(geotiff_path, 'wb') as f:
            f.write(tif_data)
        print(f"GeoTIFF: {geotiff_path} ({len(tif_data)/1024:.1f} KB)")
    except Exception as e:
        print(f"GeoTIFF export failed: {e}")

    print(f"SUCCESS: Mineral mapping. Output: {output_path}")
    print(f"Indeks mineral:")
    print(f"  Iron Oxide (B4/B2): mean={stats.get('iron_oxide_mean', 'N/A'):.3f}")
    print(f"  Clay Al-OH (B11/B12): mean={stats.get('clay_aloh_mean', 'N/A'):.3f}")
    print(f"  Ferrous Iron (B12/B8+B3/B4): mean={stats.get('ferrous_iron_mean', 'N/A'):.3f}")
    print(f"  Silica (B12/B11): mean={stats.get('silica_mean', 'N/A'):.3f}")
    print(f"RGB: R=Iron Oxide, G=Clay, B=Ferrous Iron")
    print(f"Zona alterasi: iron_oxide>1.5 AND clay>1.2")
    print(f"Ref: van der Meer et al. 2012, Hewson et al. 2005")


def make_roi(lat, lon, buffer_km):
    """Helper: create ROI from lat/lon/buffer."""
    import ee
    return ee.Geometry.Point([lon, lat]).buffer(buffer_km * 1000)


def spectral_unmixing(lat, lon, buffer_km, output_path):
    """Linear Spectral Unmixing (LSU) from Sentinel-2.
    Decomposes each pixel into fractional abundances of endmember spectra.
    Uses ee.Image.unmix() — non-negative least squares (NNLS).
    
    Endmembers: Vegetation, Soil, Water, Impervious (urban).
    Ref: Adams et al. 1986, Shimabukuro & Smith 1991
    """
    import ee, requests
    ee.Initialize()

    roi = make_roi(lat, lon, buffer_km)
    
    # S2 composite with Cloud Score+
    cs = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED') \
        .filterDate('2023-01-01', '2023-12-31').filterBounds(roi)
    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate('2023-01-01', '2023-12-31').filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30)) \
        .linkCollection(cs, ['cs_cdf']) \
        .map(lambda img: img.updateMask(img.select('cs_cdf').gte(0.60))) \
        .median().clip(roi)
    
    # Select bands for unmixing (6 bands covering VNIR-SWIR)
    bands = ['B2', 'B3', 'B4', 'B8', 'B11', 'B12']
    image = s2.select(bands).divide(10000)  # Scale to reflectance [0,1]
    
    # Endmember spectra (typical reflectance values for S2 bands)
    # B2(490), B3(560), B4(665), B8(842), B11(1610), B12(2190)
    endmembers = [
        [0.03, 0.05, 0.03, 0.45, 0.22, 0.10],  # Vegetation (green)
        [0.10, 0.15, 0.20, 0.30, 0.35, 0.30],  # Soil (bare/laterite)
        [0.06, 0.04, 0.02, 0.001, 0.0005, 0.0002],  # Water (clear)
        [0.12, 0.12, 0.13, 0.18, 0.22, 0.20],  # Impervious (urban/concrete)
    ]
    endmember_names = ['Vegetation', 'Soil', 'Water', 'Impervious']
    
    # Run unmixing — non-negative least squares
    fractions = image.unmix(endmembers, sumToOne=True, nonNegative=True)
    fractions = fractions.rename(endmember_names).clip(roi)
    
    # Stats
    stats = fractions.reduceRegion(
        reducer=ee.Reducer.mean(),
        geometry=roi, scale=20, maxPixels=1e9
    ).getInfo()
    
    # RMSE — reconstruction error
    reconstructed = ee.Image(endmembers[0]).multiply(fractions.select('Vegetation')) \
        .add(ee.Image(endmembers[1]).multiply(fractions.select('Soil'))) \
        .add(ee.Image(endmembers[2]).multiply(fractions.select('Water'))) \
        .add(ee.Image(endmembers[3]).multiply(fractions.select('Impervious')))
    rmse = image.subtract(reconstructed).pow(2).reduce(ee.Reducer.mean()).sqrt().rename('RMSE')
    
    rmse_stats = rmse.reduceRegion(
        reducer=ee.Reducer.mean(), geometry=roi, scale=20, maxPixels=1e9
    ).getInfo()
    
    result = fractions.addBands(rmse)
    
    # Visualization — RGB: R=Soil, G=Vegetation, B=Water
    vis = {
        'bands': ['Soil', 'Vegetation', 'Water'],
        'min': 0, 'max': 1,
        'region': roi, 'dimensions': 800
    }
    thumb = fractions.getThumbURL(vis)
    img_data = requests.get(thumb, timeout=60).content
    with open(output_path, 'wb') as f:
        f.write(img_data)
    
    # GeoTIFF
    geotiff_path = output_path.replace('.png', '.tif')
    try:
        url = result.getDownloadURL({
            'region': roi, 'scale': 20, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
        })
        tif_data = requests.get(url, timeout=120).content
        with open(geotiff_path, 'wb') as f:
            f.write(tif_data)
        print(f"GeoTIFF: {geotiff_path} ({len(tif_data)/1024:.1f} KB)")
    except Exception as e:
        print(f"GeoTIFF export gagal: {e}")
    
    print(f"SUCCESS: Spectral Unmixing (LSU). Output: {output_path}")
    print(f"Endmembers: {', '.join(endmember_names)}")
    print(f"Fraksi rata-rata:")
    for name in endmember_names:
        val = stats.get(name, 0) or 0
        print(f"  {name}: {val:.3f} ({val*100:.1f}%)")
    print(f"RMSE rekonstruksi: {rmse_stats.get('RMSE_mean', 'N/A')}")
    print(f"RGB: R=Soil, G=Vegetation, B=Water")
    print(f"Metode: NNLS (Non-Negative Least Squares), sumToOne=True")
    print(f"Ref: Adams et al. 1986, Shimabukuro & Smith 1991")


# CLI dispatcher
if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("ERROR: Usage: raster_engine.py <mode> [args...]")
        print("Modes: band_math_gee, band_math_local, dem_slope, dem_aspect, dem_hillshade, zonal_gee, zonal_local, unmix")
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
        elif mode == 'mineral':
            mineral_mapping(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), sys.argv[5])
        elif mode == 'unmix':
            # args: lat lon buffer_km output_path
            spectral_unmixing(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), sys.argv[5])
        else:
            print(f"ERROR: Unknown mode '{mode}'")
    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc(file=sys.stderr)
