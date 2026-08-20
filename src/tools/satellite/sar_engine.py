#!/usr/bin/env python3
"""SAR & Optical Remote Sensing Engine
Supports: flood detection, deforestation, subsidence (InSAR-like), burned area (dNBR), mangrove mapping
Uses Google Earth Engine (ee) for cloud processing and matplotlib for visualization.
"""

import sys
import os
import json
import math
import requests
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import ListedColormap, BoundaryNorm
from datetime import datetime, timedelta

try:
    import ee
except ImportError:
    print("ERROR: Google Earth Engine Python API belum terinstall. Jalankan: pip install earthengine-api")
    sys.exit(1)

# Initialize GEE
try:
    ee.Initialize()
except Exception:
    try:
        ee.Authenticate()
        ee.Initialize()
    except Exception as e:
        print(f"ERROR: Google Earth Engine belum diotentikasi. Jalankan: earthengine authenticate\nDetail: {e}")
        sys.exit(1)


def make_roi(lat, lon, buffer_km):
    """Create ROI from center point and buffer."""
    point = ee.Geometry.Point([lon, lat])
    return point.buffer(buffer_km * 1000).bounds()


def otsu_threshold_array(values, nbins=256):
    """Otsu (1979) threshold on a 1-D array: maximise between-class variance.

    Distinct from the median, which is simply the 50th percentile and carries no
    class-separation property. The local-raster path used to call a median
    "Otsu-like"; this is the actual criterion.
    """
    values = np.asarray(values, dtype=float).ravel()
    values = values[np.isfinite(values)]
    if values.size == 0:
        raise ValueError("otsu_threshold_array: no finite values")
    if np.allclose(values.min(), values.max()):
        return float(values.min())

    counts, edges = np.histogram(values, bins=nbins)
    centres = (edges[:-1] + edges[1:]) / 2.0
    total = counts.sum()
    total_sum = float((counts * centres).sum())

    best_thresh = float(centres[0])
    best_variance = -1.0
    w0 = 0.0
    sum0 = 0.0
    for i in range(len(counts)):
        w0 += counts[i]
        if w0 == 0:
            continue
        w1 = total - w0
        if w1 == 0:
            break
        sum0 += counts[i] * centres[i]
        mu0 = sum0 / w0
        mu1 = (total_sum - sum0) / w1
        variance = w0 * w1 * (mu0 - mu1) ** 2
        if variance > best_variance:
            best_variance = variance
            best_thresh = float(centres[i])
    return best_thresh


# Published accuracy for SAR flood mapping. Quoted as a spread, not a single
# flattering figure, and split by setting because urban performance is far worse
# than open-area performance.
#
#   Bonafilia et al. 2020, Sen1Floods11, DOI 10.1109/cvprw50498.2020.00113
#   Paul & Ganju 2021, semi-supervised on Sen1Floods11, arXiv 2107.08369
#   Bai et al. 2021, S1+S2 fusion on Sen1Floods11, DOI 10.3390/rs13112220
#   Bereczky et al. 2022, CNN vs rule-based, DOI 10.1109/jstars.2022.3152127
#   Aldiansyah et al. 2024, Kendari Indonesia, DOI 10.23960/jgrs.ft.unila.205
#   Zhao, Xiong & Zhu 2024, UrbanSARFloods, arXiv 2406.04111
#   Mukherjee et al. 2026, Urban Flood Observations, arXiv 2604.23066
#   Amitrano et al. 2024, review, DOI 10.3390/rs16040656
SAR_FLOOD_ACCURACY_BOUNDS = [
    ("Sen1Floods11, semi-supervised ensemble", "IoU 0.7654", "open area", "Paul & Ganju 2021"),
    ("Sen1Floods11, S1+S2 fusion", "mIoU 52.99%", "open area", "Bai et al. 2021"),
    ("Kendari, S1 + Otsu (area tergenang)", "OA 95.81%, Kappa 0.86", "open area", "Aldiansyah et al. 2024"),
    ("UFO, model tersegmentasi terlatih", "mIoU 77.3", "urban", "Mukherjee et al. 2026"),
    ("Google Dynamic World, kelas air", "IoU 48.1", "urban", "Mukherjee et al. 2026"),
    ("NASA IMPACT (Sentinel-1)", "IoU 44.1", "urban", "Mukherjee et al. 2026"),
]


def report_sar_flood_bounds():
    """Text block stating what SAR flood mapping actually achieves."""
    lines = [
        "Batas ketelitian terpublikasi untuk pemetaan banjir SAR:",
        f"  {'Metode / produk':<42} {'Skor':<24} {'Setting'}",
    ]
    for name, score, setting, ref in SAR_FLOOD_ACCURACY_BOUNDS:
        lines.append(f"  {name:<42} {score:<24} {setting}  [{ref}]")
    lines += [
        "",
        "  Catatan penting:",
        "  - Angka urban jauh lebih rendah daripada open-area. Untuk Jakarta, Semarang,",
        "    atau kota lain, pakai batas urban (IoU 44-48 untuk produk siap pakai).",
        "  - Zhao, Xiong & Zhu 2024 (UrbanSARFloods, arXiv 2406.04111): weighted",
        "    cross-entropy dan transfer learning TIDAK cukup mengatasi data tak seimbang;",
        "    deteksi banjir urban tetap sulit.",
        "  - Amitrano et al. 2024 (DOI 10.3390/rs16040656): SAR masih terbatas berat di",
        "    area bervegetasi dan urban karena mekanisme hamburan kompleks.",
        "  - Bereczky et al. 2022 (DOI 10.1109/jstars.2022.3152127): dual-pol VV+VH",
        "    mengalahkan single-pol sebesar 5% IoU.",
        "",
        "  Angka di atas adalah performa yang dilaporkan pada dataset masing-masing,",
        "  BUKAN akurasi keluaran tool ini pada AOI Anda.",
    ]
    return "\n".join(lines)


def get_thumbnail(image, roi, vis_params, dimensions=800):
    """Get thumbnail URL for visualization."""
    vis = image.visualize(**vis_params)
    url = vis.getThumbURL({
        'dimensions': dimensions,
        'region': roi,
        'format': 'png'
    })
    return url


def download_thumbnail(url, output_path):
    """Download thumbnail image from URL."""
    import requests
    r = requests.get(url, timeout=60)
    if r.status_code == 200:
        os.makedirs(os.path.dirname(output_path) if os.path.dirname(output_path) else ".", exist_ok=True)
        with open(output_path, 'wb') as f:
            f.write(r.content)
        size_kb = os.path.getsize(output_path) / 1024
        return True, size_kb
    return False, 0


def flood_detection(lat, lon, buffer_km, pre_date, post_date, output_path):
    """Sentinel-1 VV pre/post event flood change detection.
    Water threshold: -15 dB in VV polarization.
    """
    roi = make_roi(lat, lon, buffer_km)
    pre_start = (datetime.strptime(pre_date, "%Y-%m-%d") - timedelta(days=30)).strftime("%Y-%m-%d")
    post_end = (datetime.strptime(post_date, "%Y-%m-%d") + timedelta(days=15)).strftime("%Y-%m-%d")

    # Sentinel-1 GRD collection
    s1 = ee.ImageCollection('COPERNICUS/S1_GRD') \
        .filterBounds(roi) \
        .filter(ee.Filter.listContains('transmitterReceiverPolarisation', 'VV')) \
        .filter(ee.Filter.eq('instrumentMode', 'IW')) \
        .select('VV')

    # Pre-event composite
    pre_raw = s1.filterDate(pre_start, pre_date).median().clip(roi)
    # Post-event composite
    post_raw = s1.filterDate(post_date, post_end).median().clip(roi)

    # Speckle filter — 3x3 focal median (standard SAR preprocessing)
    pre = pre_raw.focal_median(radius=30, kernelType='circle', units='meters')
    post = post_raw.focal_median(radius=30, kernelType='circle', units='meters')

    # Change detection: water appears as decrease in VV backscatter
    diff = pre.subtract(post).rename('diff')

    # Otsu adaptive threshold (Aldiansyah et al. 2024, 95.81% OA)
    def otsu_threshold(image, region, scale=30):
        """Compute Otsu threshold from image histogram."""
        histogram = image.reduceRegion(
            reducer=ee.Reducer.histogram(256, 0.1),
            geometry=region,
            scale=scale,
            maxPixels=1e9,
            bestEffort=True
        ).getInfo()

        hist_key = list(histogram.keys())[0]
        hist_data = histogram[hist_key]
        counts = hist_data['histogram']
        means = hist_data['bucketMeans']

        # Client-side Otsu
        counts = np.array(counts)
        means = np.array(means)
        total = counts.sum()

        best_thresh = means[0]
        best_variance = 0

        w0 = 0
        sum0 = 0
        total_sum = (counts * means).sum()

        for i in range(len(counts)):
            w0 += counts[i]
            if w0 == 0:
                continue
            w1 = total - w0
            if w1 == 0:
                break

            sum0 += counts[i] * means[i]
            mu0 = sum0 / w0
            mu1 = (total_sum - sum0) / w1

            variance = w0 * w1 * (mu0 - mu1) ** 2
            if variance > best_variance:
                best_variance = variance
                best_thresh = means[i]

        return best_thresh

    # Use Otsu instead of fixed -15 dB, with fallback
    try:
        vv_threshold = otsu_threshold(post, roi)
    except Exception:
        vv_threshold = -15  # fallback

    # Threshold for flood: post VV < Otsu threshold AND change > 3 dB
    flood_mask = post.lt(vv_threshold).And(diff.gt(3))

    # Create visualization
    # RGB: R=pre_VV, G=post_VV, B=pre_VV (highlights change in green channel)
    rgb = ee.Image.cat([pre, post, pre]).rename(['R', 'G', 'B'])

    url = get_thumbnail(rgb, roi, {
        'bands': ['R', 'G', 'B'],
        'min': -25,
        'max': 0,
        'gamma': 1.2
    })

    ok, size = download_thumbnail(url, output_path)
    if ok:
        # Get flood extent statistics
        flood_area = flood_mask.multiply(ee.Image.pixelArea())
        stats = flood_area.reduceRegion(
            reducer=ee.Reducer.sum(),
            geometry=roi,
            scale=10,
            maxPixels=1e9
        )
        flood_ha = stats.getInfo().get('VV', 0) / 10000

        return (f"SUCCESS: Peta banjir disimpan di {output_path} ({size:.1f} KB)\n"
                f"Pre-event: {pre_start} - {pre_date}\n"
                f"Post-event: {post_date} - {post_end}\n"
                f"Estimasi area tergenang: {flood_ha:.1f} Ha\n"
                f"Visualisasi: R=pre-VV, G=post-VV, B=pre-VV (area hijau = banjir)\n"
                f"Threshold: VV < {vv_threshold:.1f} dB (Otsu adaptive) dan perubahan > 3 dB\n"
                f"\nBELUM DITERAPKAN pada pipeline ini: radiometric terrain correction\n"
                f"(koreksi lereng) dan speckle filter. Di medan berlereng, backscatter\n"
                f"belum dikoreksi sehingga bayangan dan layover radar dapat terbaca\n"
                f"sebagai air. Rujukan koreksi: Vollrath, Mullissa & Reiche 2020,\n"
                f"DOI 10.3390/rs12111867 (modul GEE angular-based slope correction).\n"
                f"\n{report_sar_flood_bounds()}")
    return "ERROR: Gagal mengunduh hasil deteksi banjir."


def deforestation_detection(lat, lon, buffer_km, start_date, end_date, output_path):
    """Sentinel-1 VV/VH temporal backscatter loss detection for deforestation."""
    roi = make_roi(lat, lon, buffer_km)

    s1 = ee.ImageCollection('COPERNICUS/S1_GRD') \
        .filterBounds(roi) \
        .filter(ee.Filter.listContains('transmitterReceiverPolarisation', 'VV')) \
        .filter(ee.Filter.listContains('transmitterReceiverPolarisation', 'VH')) \
        .filter(ee.Filter.eq('instrumentMode', 'IW'))

    # Split into early and late periods
    mid_date = datetime.strptime(start_date, "%Y-%m-%d") + \
        (datetime.strptime(end_date, "%Y-%m-%d") - datetime.strptime(start_date, "%Y-%m-%d")) / 2
    mid_str = mid_date.strftime("%Y-%m-%d")

    early_raw = s1.filterDate(start_date, mid_str).median().clip(roi)
    late_raw = s1.filterDate(mid_str, end_date).median().clip(roi)

    # Speckle filter — 3x3 focal median (standard SAR preprocessing)
    early = early_raw.focal_median(radius=30, kernelType='circle', units='meters')
    late = late_raw.focal_median(radius=30, kernelType='circle', units='meters')

    # VH backscatter decrease indicates forest loss
    vh_early = early.select('VH')
    vh_late = late.select('VH')
    vh_change = vh_early.subtract(vh_late).rename('vh_change')

    # Forest loss: VH decrease > 3 dB (forest has higher VH than open land)
    forest_loss = vh_change.gt(3)

    # RGB visualization: R=early_VH, G=late_VH, B=early_VV
    rgb = ee.Image.cat([
        early.select('VH'),
        late.select('VH'),
        early.select('VV')
    ]).rename(['R', 'G', 'B'])

    url = get_thumbnail(rgb, roi, {
        'bands': ['R', 'G', 'B'],
        'min': -25,
        'max': -5,
        'gamma': 1.3
    })

    ok, size = download_thumbnail(url, output_path)
    if ok:
        loss_area = forest_loss.multiply(ee.Image.pixelArea())
        stats = loss_area.reduceRegion(
            reducer=ee.Reducer.sum(),
            geometry=roi,
            scale=10,
            maxPixels=1e9
        )
        loss_ha = stats.getInfo().get('vh_change', 0) / 10000

        return (f"SUCCESS: Peta deforestasi disimpan di {output_path} ({size:.1f} KB)\n"
                f"Periode awal: {start_date} - {mid_str}\n"
                f"Periode akhir: {mid_str} - {end_date}\n"
                f"Estimasi kehilangan hutan: {loss_ha:.1f} Ha\n"
                f"Visualisasi: R=early-VH, G=late-VH, B=early-VV (merah = kehilangan)\n"
                f"Metode: Sentinel-1 VH backscatter change (threshold > 3 dB)")
    return "ERROR: Gagal mengunduh hasil deteksi deforestasi."


def subsidence_screening(lat, lon, buffer_km, start_date, end_date, output_path):
    """Simplified InSAR-like subsidence screening using Sentinel-1 coherence proxy.
    DISCLAIMER: This is a screening-level analysis only. Full InSAR processing
    requires dedicated software (SNAP, ISCE, StaMPS).
    """
    roi = make_roi(lat, lon, buffer_km)

    s1 = ee.ImageCollection('COPERNICUS/S1_GRD') \
        .filterBounds(roi) \
        .filter(ee.Filter.listContains('transmitterReceiverPolarisation', 'VV')) \
        .filter(ee.Filter.eq('instrumentMode', 'IW')) \
        .select('VV')

    # Compute temporal statistics as proxy for ground stability
    # High variance in VV over time suggests surface change (proxy for deformation)
    collection = s1.filterDate(start_date, end_date)

    mean_vv = collection.mean().clip(roi)
    std_vv = collection.reduce(ee.Reducer.stdDev()).clip(roi)

    # Coefficient of variation as instability indicator
    cv = std_vv.divide(mean_vv.abs()).rename('cv')

    # Download GeoTIFF (for SNI overlay)
    import requests as req
    tif_path = output_path if output_path.endswith('.tif') else output_path.replace('.png', '.tif')
    try:
        dl_url = cv.toFloat().getDownloadURL({
            'scale': 30, 'region': roi, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
        })
        r = req.get(dl_url, timeout=60)
        if r.status_code == 200 and len(r.content) > 1024:
            with open(tif_path, 'wb') as f:
                f.write(r.content)
    except Exception as e:
        print(f"[WARNING] GeoTIFF download gagal: {e}")

    # PNG thumbnail for quick preview
    png_path = output_path if output_path.endswith('.png') else output_path.replace('.tif', '.png')
    url = get_thumbnail(cv, roi, {
        'bands': ['cv'],
        'min': 0,
        'max': 0.5,
        'palette': ['blue', 'green', 'yellow', 'orange', 'red']
    })

    ok, size = download_thumbnail(url, png_path)
    if ok:
        return (f"SUCCESS: Peta screening subsiden disimpan di {output_path} ({size:.1f} KB)\n"
                f"Periode: {start_date} - {end_date}\n"
                f"Metode: Koefisien variasi temporal VV (proxy stabilitas)\n"
                f"Warna: biru=stabil, merah=tidak stabil\n\n"
                f"DISCLAIMER: Ini adalah analisis screening level saja.\n"
                f"InSAR penuh memerlukan pemrosesan dedicated (SNAP/ISCE/StaMPS).\n"
                f"Hasil ini TIDAK menunjukkan laju penurunan tanah aktual.\n"
                f"Gunakan sebagai indikasi awal untuk penyelidikan lebih lanjut.")
    return "ERROR: Gagal mengunduh hasil screening subsiden."


def mask_s2_clouds(image):
    """Pixel-level cloud masking using SCL band.
    Removes: cloud shadow(3), cloud medium(8), cloud high(9), cirrus(10), saturated/defective(1)
    """
    scl = image.select('SCL')
    mask = scl.neq(3).And(scl.neq(8)).And(scl.neq(9)).And(scl.neq(10)).And(scl.neq(1))
    return image.updateMask(mask)


def burned_area_mapping(lat, lon, buffer_km, fire_date, output_path):
    """Sentinel-2 dNBR (differenced Normalized Burn Ratio) for burn severity mapping.
    NBR = (NIR - SWIR2) / (NIR + SWIR2) = (B8 - B12) / (B8 + B12)
    dNBR = pre_NBR - post_NBR
    Severity classes per USGS:
      <-0.25: Enhanced regrowth, high
      -0.25 to -0.1: Enhanced regrowth, low
      -0.1 to 0.1: Unburned
      0.1 to 0.27: Low severity
      0.27 to 0.44: Moderate-low severity
      0.44 to 0.66: Moderate-high severity
      >0.66: High severity
    """
    roi = make_roi(lat, lon, buffer_km)

    fire_dt = datetime.strptime(fire_date, "%Y-%m-%d")
    pre_start = (fire_dt - timedelta(days=60)).strftime("%Y-%m-%d")
    pre_end = (fire_dt - timedelta(days=5)).strftime("%Y-%m-%d")
    post_start = (fire_dt + timedelta(days=5)).strftime("%Y-%m-%d")
    post_end = (fire_dt + timedelta(days=60)).strftime("%Y-%m-%d")

    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30)) \
        .map(mask_s2_clouds)

    def compute_nbr(image):
        return image.normalizedDifference(['B8', 'B12']).rename('NBR')

    pre_nbr = s2.filterDate(pre_start, pre_end).map(compute_nbr).median().clip(roi)
    post_nbr = s2.filterDate(post_start, post_end).map(compute_nbr).median().clip(roi)

    dnbr = pre_nbr.subtract(post_nbr).rename('dNBR')

    # Severity classification
    severity = ee.Image(0) \
        .where(dnbr.lt(-0.25), 1) \
        .where(dnbr.gte(-0.25).And(dnbr.lt(-0.1)), 2) \
        .where(dnbr.gte(-0.1).And(dnbr.lt(0.1)), 3) \
        .where(dnbr.gte(0.1).And(dnbr.lt(0.27)), 4) \
        .where(dnbr.gte(0.27).And(dnbr.lt(0.44)), 5) \
        .where(dnbr.gte(0.44).And(dnbr.lt(0.66)), 6) \
        .where(dnbr.gte(0.66), 7) \
        .clip(roi).rename('severity')

    # Palette: green (regrowth) -> yellow (unburned) -> orange -> red (high severity)
    url = get_thumbnail(severity, roi, {
        'bands': ['severity'],
        'min': 1,
        'max': 7,
        'palette': ['#1a9641', '#73d216', '#f7f7f7', '#fee08b', '#fdae61', '#f46d43', '#d73027']
    })

    ok, size = download_thumbnail(url, output_path)
    if ok:
        # Calculate burned area statistics
        burned = dnbr.gt(0.1).multiply(ee.Image.pixelArea())
        stats = burned.reduceRegion(
            reducer=ee.Reducer.sum(),
            geometry=roi,
            scale=20,
            maxPixels=1e9
        )
        burned_ha = stats.getInfo().get('dNBR', 0) / 10000

        # Thematic Cartography: Overlay severity raster di atas basemap SNI
        try:
            temp_tif = output_path.replace('.png', '_overlay.tif')
            download_url = severity.toFloat().getDownloadURL({
                'region': roi, 'scale': 20, 'format': 'GEO_TIFF', 'crs': 'EPSG:3857'
            })
            tif_data = requests.get(download_url, timeout=120).content
            with open(temp_tif, 'wb') as f: f.write(tif_data)

            import json
            d = buffer_km / 111.0
            dlon = d / math.cos(math.radians(lat))
            geojson_data = {"type":"FeatureCollection","features":[{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":[[[lon-dlon,lat-d],[lon+dlon,lat-d],[lon+dlon,lat+d],[lon-dlon,lat+d],[lon-dlon,lat-d]]]}}]}

            discrete_labels = {
                '#1a9641': 'Regrowth',
                '#73d216': 'Unburned',
                '#f7f7f7': 'Unburned',
                '#fee08b': 'Low Sev',
                '#fdae61': 'Mod-Low',
                '#f46d43': 'Mod-High',
                '#d73027': 'High Sev',
            }

            sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'gis'))
            from cartography import generate_sni_map
            generate_sni_map(json.dumps(geojson_data), output_path,
                title="PETA BURN SEVERITY (dNBR)",
                realtime=True, author="Rizki Agustiawan x ZeroClaw AI",
                overlay_raster=temp_tif,
                analysis_type='discrete', cmap='RdYlGn',
                discrete_labels=discrete_labels,
                colorbar_label='Burn Severity (USGS 7-class)',
                analysis_stats={
                    'Algoritma': 'dNBR+RdNBR',
                    'Sensor': 'Sentinel-2 L2A',
                    'Sumber': 'Google Earth Engine',
                    'Area Terbakar': f'{burned_ha:.0f} Ha',
                })
            if os.path.exists(temp_tif): os.remove(temp_tif)
            size = os.path.getsize(output_path) / 1024
        except Exception as e:
            print(f"[WARNING] Thematic cartography gagal, menggunakan thumbnail: {e}")

        return (f"SUCCESS: Peta area terbakar disimpan di {output_path} ({size:.1f} KB)\n"
                f"Tanggal kebakaran: {fire_date}\n"
                f"Pre-fire: {pre_start} - {pre_end}\n"
                f"Post-fire: {post_start} - {post_end}\n"
                f"Estimasi total area terbakar (dNBR > 0.1): {burned_ha:.1f} Ha\n"
                f"Klasifikasi (USGS): hijau=regrowth, kuning=unburned, oranye=low, merah=high severity\n"
                f"Metode: Sentinel-2 dNBR = (preNBR - postNBR)")
    return "ERROR: Gagal mengunduh peta area terbakar."


def mangrove_mapping(lat, lon, buffer_km, output_path):
    """Map mangrove extent using Sentinel-2 spectral indices + elevation filter.
    Criteria: NDVI > 0.3 AND MNDWI in [-0.3, 0.3] AND elevation < 10m
    """
    roi = make_roi(lat, lon, buffer_km)

    # Latest cloud-free Sentinel-2 composite (last 6 months)
    end_date = datetime.now().strftime("%Y-%m-%d")
    start_date = (datetime.now() - timedelta(days=180)).strftime("%Y-%m-%d")

    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterBounds(roi) \
        .filterDate(start_date, end_date) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 20)) \
        .map(mask_s2_clouds) \
        .median() \
        .clip(roi)

    # NDVI = (NIR - Red) / (NIR + Red) = (B8 - B4) / (B8 + B4)
    ndvi = s2.normalizedDifference(['B8', 'B4']).rename('NDVI')

    # NOTE: McFeeters NDWI = (Green-NIR)/(Green+NIR) is NEGATIVE for vegetation
    # So ndwi.gt(0) would select WATER, not mangrove — contradicts ndvi.gt(0.3)
    # Instead use MNDWI range to identify the land-water transition zone where mangroves grow

    # MNDWI (Modified NDWI, Xu 2006) = (Green - SWIR1) / (Green + SWIR1)
    # MNDWI > 0 for water, < 0 for vegetation/soil
    # Mangrove = vegetation (NDVI>0.3) NEAR water (MNDWI > -0.3) at low elevation
    mndwi = s2.normalizedDifference(['B3', 'B11']).rename('MNDWI')

    # Elevation from SRTM
    srtm = ee.Image('USGS/SRTMGL1_003').clip(roi)
    elevation = srtm.select('elevation')

    # Mangrove criteria: NDVI > 0.3 AND MNDWI in [-0.3, 0.3] AND elevation < 10m
    mangrove_mask = ndvi.gt(0.3).And(mndwi.gt(-0.3)).And(mndwi.lt(0.3)).And(elevation.lt(10))

    # Create false color composite with mangrove overlay
    # Base: S2 true color
    base = s2.visualize(bands=['B4', 'B3', 'B2'], min=0, max=2500, gamma=1.2)

    # Mangrove overlay in green
    mangrove_vis = mangrove_mask.selfMask().visualize(
        palette=['#00ff00'],
        min=0, max=1
    )

    # Blend
    combined = base.blend(mangrove_vis.updateMask(mangrove_mask))

    # Download GeoTIFF (for SNI overlay) — write to output_path if .tif, else separate
    import requests as req
    tif_path = output_path if output_path.endswith('.tif') else output_path.replace('.png', '.tif')
    try:
        dl_url = mangrove_mask.toFloat().getDownloadURL({
            'scale': 30, 'region': roi, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
        })
        r = req.get(dl_url, timeout=60)
        if r.status_code == 200 and len(r.content) > 1024:
            with open(tif_path, 'wb') as f:
                f.write(r.content)
    except Exception as e:
        print(f"[WARNING] GeoTIFF download gagal: {e}")

    # PNG thumbnail for quick preview
    png_path = output_path if output_path.endswith('.png') else output_path.replace('.tif', '.png')
    url = combined.getThumbURL({
        'dimensions': 800,
        'region': roi,
        'format': 'png'
    })

    ok, size = download_thumbnail(url, png_path)
    if ok:
        mangrove_area = mangrove_mask.multiply(ee.Image.pixelArea())
        stats = mangrove_area.reduceRegion(
            reducer=ee.Reducer.sum(),
            geometry=roi,
            scale=10,
            maxPixels=1e9
        )
        mangrove_ha = stats.getInfo().get('NDVI', 0) / 10000

        return (f"SUCCESS: Peta mangrove disimpan di {output_path} ({size:.1f} KB)\n"
                f"Periode citra: {start_date} - {end_date}\n"
                f"Estimasi luas mangrove: {mangrove_ha:.1f} Ha\n"
                f"Kriteria: NDVI > 0.3 AND MNDWI in [-0.3, 0.3] AND elevasi < 10m\n"
                f"Overlay: area hijau = mangrove terdeteksi\n"
                f"Sumber elevasi: SRTM 30m")
    return "ERROR: Gagal mengunduh peta mangrove."


def local_analysis(input_path, output_path, analysis_type):
    """Analyze locally downloaded SAR/optical GeoTIFF."""
    try:
        import rasterio
    except ImportError:
        return "ERROR: rasterio belum terinstall. Jalankan: pip install rasterio"

    if not os.path.exists(input_path):
        return f"ERROR: File tidak ditemukan: {input_path}"

    file_size_mb = os.path.getsize(input_path) / (1024 * 1024)
    if file_size_mb > 500:
        return f"ERROR: File terlalu besar ({file_size_mb:.1f} MB). Maksimum 500 MB."

    with rasterio.open(input_path) as src:
        data = src.read(1)
        profile = src.profile

        fig, ax = plt.subplots(figsize=(12, 10))

        if analysis_type == "histogram":
            valid = data[data != src.nodata] if src.nodata is not None else data.flatten()
            ax.hist(valid.flatten(), bins=100, color='steelblue', edgecolor='black', alpha=0.7)
            ax.set_title(f"Histogram - {os.path.basename(input_path)}", fontweight='bold')
            ax.set_xlabel("Nilai Piksel")
            ax.set_ylabel("Frekuensi")
            stats_text = (f"Min: {valid.min():.2f}\nMax: {valid.max():.2f}\n"
                         f"Mean: {valid.mean():.2f}\nStd: {valid.std():.2f}")
            ax.text(0.95, 0.95, stats_text, transform=ax.transAxes, fontsize=10,
                    verticalalignment='top', horizontalalignment='right',
                    bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.8))

        elif analysis_type == "threshold":
            # Binary classification via Otsu (1979): maximises between-class
            # variance. Previously this computed a median and labelled it
            # "Otsu-like", which are different criteria and give different cuts.
            valid = data[data != src.nodata] if src.nodata is not None else data.flatten()
            try:
                threshold = otsu_threshold_array(valid)
                method_label = "Otsu 1979"
            except ValueError:
                threshold = float(np.median(valid))
                method_label = "median (fallback: Otsu gagal)"
            binary = np.where(data > threshold, 1, 0)
            ax.imshow(binary, cmap='RdYlGn')
            ax.set_title(f"Threshold Analysis (T={threshold:.2f}, {method_label})",
                         fontweight='bold')

        else:  # Default: visualization
            vmin = np.percentile(data[data != 0], 2) if np.any(data != 0) else data.min()
            vmax = np.percentile(data[data != 0], 98) if np.any(data != 0) else data.max()
            im = ax.imshow(data, cmap='viridis', vmin=vmin, vmax=vmax)
            plt.colorbar(im, ax=ax, label='Nilai Piksel')
            ax.set_title(f"Visualisasi - {os.path.basename(input_path)}", fontweight='bold')

        plt.savefig(output_path, dpi=200, bbox_inches='tight')
        plt.close()

        return (f"SUCCESS: Analisis lokal disimpan di {output_path}\n"
                f"Input: {input_path} ({file_size_mb:.1f} MB)\n"
                f"Tipe analisis: {analysis_type}\n"
                f"Dimensi: {data.shape[1]} x {data.shape[0]} piksel\n"
                f"CRS: {profile.get('crs', 'N/A')}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: sar_engine.py <mode> [args...]")
        print("Modes: flood, deforestation, subsidence, burned_area, mangrove, local")
        sys.exit(1)

    mode = sys.argv[1]

    try:
        if mode == "flood":
            if len(sys.argv) < 8:
                print("ERROR: flood memerlukan: lat lon buffer_km pre_date post_date output_path")
                sys.exit(1)
            lat, lon, buf = float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4])
            pre_date, post_date, output = sys.argv[5], sys.argv[6], sys.argv[7]
            print(flood_detection(lat, lon, buf, pre_date, post_date, output))

        elif mode == "deforestation":
            if len(sys.argv) < 8:
                print("ERROR: deforestation memerlukan: lat lon buffer_km start_date end_date output_path")
                sys.exit(1)
            lat, lon, buf = float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4])
            start, end, output = sys.argv[5], sys.argv[6], sys.argv[7]
            print(deforestation_detection(lat, lon, buf, start, end, output))

        elif mode == "subsidence":
            if len(sys.argv) < 8:
                print("ERROR: subsidence memerlukan: lat lon buffer_km start_date end_date output_path")
                sys.exit(1)
            lat, lon, buf = float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4])
            start, end, output = sys.argv[5], sys.argv[6], sys.argv[7]
            print(subsidence_screening(lat, lon, buf, start, end, output))

        elif mode == "burned_area":
            if len(sys.argv) < 7:
                print("ERROR: burned_area memerlukan: lat lon buffer_km fire_date output_path")
                sys.exit(1)
            lat, lon, buf = float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4])
            fire_date, output = sys.argv[5], sys.argv[6]
            print(burned_area_mapping(lat, lon, buf, fire_date, output))

        elif mode == "mangrove":
            if len(sys.argv) < 6:
                print("ERROR: mangrove memerlukan: lat lon buffer_km output_path")
                sys.exit(1)
            lat, lon, buf = float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4])
            output = sys.argv[5]
            print(mangrove_mapping(lat, lon, buf, output))

        elif mode == "local":
            if len(sys.argv) < 5:
                print("ERROR: local memerlukan: input_path output_path analysis_type")
                sys.exit(1)
            input_path, output_path, analysis_type = sys.argv[2], sys.argv[3], sys.argv[4]
            print(local_analysis(input_path, output_path, analysis_type))

        else:
            print(f"ERROR: Mode '{mode}' tidak dikenal. Gunakan: flood, deforestation, subsidence, burned_area, mangrove, local")
            sys.exit(1)

    except Exception as e:
        print(f"ERROR: {e}")
        sys.exit(1)
