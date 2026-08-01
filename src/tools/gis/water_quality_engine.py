#!/usr/bin/env python3
"""Satellite Water Quality Engine — Research-grade estimation from Sentinel-2
Algorithms: Nechad 2010 (TSS/Turbidity), Dogliotti 2015 (Turbidity),
OC3 (Chlorophyll-a), Mishra 2012 (NDCI Chl-a)
Uses Google Earth Engine for cloud processing.
Ref: PP 22/2021 (Baku Mutu Air), KepMenLH 51/2004 (Baku Mutu Air Laut)
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from provenance import create_provenance
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.colors as mcolors
import requests
import ee

try:
    ee.Initialize()
except Exception:
    print("ERROR: Google Earth Engine belum diotentikasi. Jalankan 'earthengine authenticate'.")
    sys.exit(1)


# === Indonesian Water Quality Standards ===
# PP 22/2021 Baku Mutu Air (freshwater classes I-IV)
PP22_TSS_LIMITS = {
    'Kelas I': 50,    # mg/L
    'Kelas II': 50,
    'Kelas III': 400,
    'Kelas IV': 400,
}

# KepMenLH 51/2004 Baku Mutu Air Laut
KEPMENLH51_CHL = {
    'Biota Laut': 0.008,    # mg/L = 8 µg/L
    'Wisata Bahari': 0.015,  # mg/L = 15 µg/L
}


def _get_s2_composite(roi, start_date, end_date):
    """Get cloud-masked Sentinel-2 SR composite with Cloud Score+ and water mask."""
    csPlus = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED')
    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate(start_date, end_date) \
        .filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30)) \
        .linkCollection(csPlus, ['cs_cdf']) \
        .map(lambda img: img.updateMask(img.select('cs_cdf').gte(0.60)))

    count = s2.size().getInfo()
    if count == 0:
        return None, 0

    composite = s2.median().clip(roi)

    # Water mask: MNDWI > 0 (Green - SWIR) / (Green + SWIR)
    green = composite.select('B3').multiply(0.0001)
    swir = composite.select('B11').multiply(0.0001)
    mndwi = green.subtract(swir).divide(green.add(swir))
    water_mask = mndwi.gt(0)

    composite = composite.updateMask(water_mask)
    return composite, count


def _save_geotiff(image, roi, output_path, scale=10):
    """Download GeoTIFF from GEE."""
    tif_path = output_path.replace('.png', '.tif')
    try:
        url = image.toFloat().getDownloadURL({
            'scale': scale, 'region': roi,
            'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
        })
        r = requests.get(url, timeout=120)
        with open(tif_path, 'wb') as f:
            f.write(r.content)
        return tif_path
    except Exception as e:
        print(f"PERINGATAN: GeoTIFF gagal diunduh: {e}")
        return None


def _save_thumb(image, roi, output_path, vis_params):
    """Download PNG thumbnail from GEE."""
    vis_params['region'] = roi
    if 'dimensions' not in vis_params:
        vis_params['dimensions'] = 800
    thumb_url = image.getThumbURL(vis_params)
    r = requests.get(thumb_url, timeout=60)
    with open(output_path, 'wb') as f:
        f.write(r.content)


def _get_stats(image, roi, band_name='result', scale=10):
    """Get mean/min/max stats from image."""
    stats = image.select(band_name).reduceRegion(
        reducer=ee.Reducer.mean().combine(
            ee.Reducer.minMax(), sharedInputs=True
        ),
        geometry=roi, scale=scale, maxPixels=1e9
    ).getInfo()
    mean_val = stats.get(f'{band_name}_mean', None)
    min_val = stats.get(f'{band_name}_min', None)
    max_val = stats.get(f'{band_name}_max', None)
    return mean_val, min_val, max_val


def estimate_tss(lat, lon, buffer_km, start_date, end_date, output_path):
    """Estimasi Total Suspended Solids (TSS) menggunakan algoritma Nechad et al. 2010.
    TSS = A_T * rho_w / (1 - rho_w / C_T)
    Band 4 (665nm): A_T = 355.85, C_T = 0.1728 (Nechad 2010, Table 2)
    rho_w = pi * Rrs (above-water reflectance)
    S2 SR DN [0,10000], scale factor 0.0001
    """
    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    composite, count = _get_s2_composite(roi, start_date, end_date)
    if composite is None:
        print(f"ERROR: Tidak ada data Sentinel-2 untuk lokasi ini pada periode {start_date} - {end_date}")
        return

    # Nechad 2010 coefficients (Table 2, 665nm)
    A_T = 355.85
    C_T = 0.1728

    # Convert B4 DN to Rrs then to rho_w (above-water reflectance)
    # S2 SR: DN * 0.0001 = surface reflectance ≈ pi * Rrs
    # rho_w = surface reflectance (already pi*Rrs in SR product)
    rho_w = composite.select('B4').multiply(0.0001)

    # TSS = A_T * rho_w / (1 - rho_w / C_T)
    tss = rho_w.multiply(A_T).divide(
        ee.Image(1).subtract(rho_w.divide(C_T))
    ).rename('result')

    # Clamp reasonable range
    tss = tss.where(tss.lt(0), 0).where(tss.gt(5000), 5000)

    # Stats
    mean_val, min_val, max_val = _get_stats(tss, roi)

    # GeoTIFF
    tif_path = _save_geotiff(tss, roi, output_path, scale=10)

    # PNG with colorbar using matplotlib
    vis_params = {
        'min': 0, 'max': 200,
        'palette': ['001f3f', '0074D9', '7FDBFF', '2ECC40', 'FFDC00', 'FF851B', 'FF4136']
    }
    png_path = output_path if output_path.endswith('.png') else output_path.replace('.tif', '.png')
    _save_thumb(tss, roi, png_path, vis_params)

    # Generate matplotlib map with colorbar
    fig, ax = plt.subplots(1, 1, figsize=(10, 8))
    img_data = plt.imread(png_path)
    ax.imshow(img_data)
    ax.set_title(f'Total Suspended Solids (TSS) — Nechad et al. 2010\n'
                 f'Lat: {lat}, Lon: {lon} | {start_date} s/d {end_date}', fontweight='bold')
    ax.axis('off')

    # Colorbar
    cmap = mcolors.LinearSegmentedColormap.from_list('tss',
        ['#001f3f', '#0074D9', '#7FDBFF', '#2ECC40', '#FFDC00', '#FF851B', '#FF4136'])
    norm = plt.Normalize(vmin=0, vmax=200)
    sm = plt.cm.ScalarMappable(cmap=cmap, norm=norm)
    sm.set_array([])
    cbar = fig.colorbar(sm, ax=ax, fraction=0.046, pad=0.04)
    cbar.set_label('TSS (mg/L)', fontsize=12)

    plt.tight_layout()
    plt.savefig(png_path, dpi=200, bbox_inches='tight')
    plt.close()

    # Provenance metadata
    try:
        create_provenance(png_path,
            tool='estimate_tss', gee_collection='COPERNICUS/S2_SR_HARMONIZED',
            date_range=[start_date, end_date],
            coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
            algorithms=['Nechad 2010 TSS', 'Cloud Score+ masking', 'MNDWI water mask'],
            references=['Nechad et al. 2010'],
            crs='EPSG:4326', scale_m=10)
    except:
        pass  # provenance is non-critical

    # Print results
    print(f"SUCCESS: Estimasi TSS selesai.")
    print("DISCLAIMER: S2 SR bukan water-leaving reflectance. Hasil approximate. Validasi dengan sampling in-situ.")
    print(f"Output PNG: {png_path}")
    if tif_path:
        print(f"Output GeoTIFF: {tif_path}")
    print(f"Sumber: Sentinel-2 SR (Cloud Score+ masked) | Scene: {count}")
    print(f"Algoritma: Nechad et al. 2010 (Band 4, 665nm)")
    print(f"Periode: {start_date} s/d {end_date}")
    print(f"\n=== Statistik TSS ===")
    if mean_val is not None:
        print(f"  Rata-rata : {mean_val:.2f} mg/L")
        print(f"  Minimum   : {min_val:.2f} mg/L")
        print(f"  Maksimum  : {max_val:.2f} mg/L")
    else:
        print("  Tidak ada piksel air yang terdeteksi.")

    # Compare with PP 22/2021
    print(f"\n=== Perbandingan Baku Mutu Air (PP 22/2021) ===")
    if mean_val is not None:
        for kelas, limit in PP22_TSS_LIMITS.items():
            status = "MEMENUHI" if mean_val <= limit else "MELEBIHI"
            print(f"  {kelas} (TSS ≤ {limit} mg/L): {status}")


def estimate_turbidity(lat, lon, buffer_km, start_date, end_date, output_path):
    """Estimasi Turbiditas menggunakan algoritma Dogliotti et al. 2015.
    Low turbidity (< 15 FNU):  T = 228.1 * rho_w_RED / (1 - rho_w_RED / 0.1641)
    High turbidity (>= 15 FNU): T = 3078.9 * rho_w_NIR / (1 - rho_w_NIR / 0.2112)
    Blending zone: weighted average between 7-15 FNU transition.
    S2 B4 (665nm) for Red, B8 (842nm) for NIR.
    """
    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    composite, count = _get_s2_composite(roi, start_date, end_date)
    if composite is None:
        print(f"ERROR: Tidak ada data Sentinel-2 untuk lokasi ini pada periode {start_date} - {end_date}")
        return

    # Surface reflectance (rho_w)
    rho_red = composite.select('B4').multiply(0.0001)   # 665nm
    rho_nir = composite.select('B8').multiply(0.0001)   # 842nm

    # Dogliotti 2015: low-turbidity formula (Red band)
    # T_low = 228.1 * rho_red / (1 - rho_red / 0.1641)
    t_low = rho_red.multiply(228.1).divide(
        ee.Image(1).subtract(rho_red.divide(0.1641))
    )

    # Dogliotti 2015: high-turbidity formula (NIR band)
    # T_high = 3078.9 * rho_nir / (1 - rho_nir / 0.2112)
    t_high = rho_nir.multiply(3078.9).divide(
        ee.Image(1).subtract(rho_nir.divide(0.2112))
    )

    # Blending zone: weighted average between 7 and 15 FNU
    # w = (t_low - 7) / (15 - 7) => weight for high-turbidity formula
    w = t_low.subtract(7).divide(8).clamp(0, 1)
    turbidity = t_low.multiply(ee.Image(1).subtract(w)).add(t_high.multiply(w)).rename('result')

    # Clamp
    turbidity = turbidity.where(turbidity.lt(0), 0).where(turbidity.gt(3000), 3000)

    # Stats
    mean_val, min_val, max_val = _get_stats(turbidity, roi)

    # GeoTIFF
    tif_path = _save_geotiff(turbidity, roi, output_path, scale=10)

    # PNG thumbnail
    vis_params = {
        'min': 0, 'max': 100,
        'palette': ['0000FF', '00BFFF', '00FF00', 'ADFF2F', 'FFFF00', 'FFA500', 'FF0000']
    }
    png_path = output_path if output_path.endswith('.png') else output_path.replace('.tif', '.png')
    _save_thumb(turbidity, roi, png_path, vis_params)

    # Matplotlib with colorbar
    fig, ax = plt.subplots(1, 1, figsize=(10, 8))
    img_data = plt.imread(png_path)
    ax.imshow(img_data)
    ax.set_title(f'Turbiditas — Dogliotti et al. 2015\n'
                 f'Lat: {lat}, Lon: {lon} | {start_date} s/d {end_date}', fontweight='bold')
    ax.axis('off')

    cmap = mcolors.LinearSegmentedColormap.from_list('turb',
        ['#0000FF', '#00BFFF', '#00FF00', '#ADFF2F', '#FFFF00', '#FFA500', '#FF0000'])
    norm = plt.Normalize(vmin=0, vmax=100)
    sm = plt.cm.ScalarMappable(cmap=cmap, norm=norm)
    sm.set_array([])
    cbar = fig.colorbar(sm, ax=ax, fraction=0.046, pad=0.04)
    cbar.set_label('Turbiditas (FNU)', fontsize=12)

    plt.tight_layout()
    plt.savefig(png_path, dpi=200, bbox_inches='tight')
    plt.close()

    # Provenance metadata
    try:
        create_provenance(png_path,
            tool='estimate_turbidity', gee_collection='COPERNICUS/S2_SR_HARMONIZED',
            date_range=[start_date, end_date],
            coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
            algorithms=['Dogliotti 2015 Turbidity', 'Cloud Score+ masking', 'MNDWI water mask'],
            references=['Dogliotti et al. 2015'],
            crs='EPSG:4326', scale_m=10)
    except:
        pass  # provenance is non-critical

    # Print results
    print(f"SUCCESS: Estimasi Turbiditas selesai.")
    print("DISCLAIMER: S2 SR bukan water-leaving reflectance. Hasil approximate. Validasi dengan sampling in-situ.")
    print(f"Output PNG: {png_path}")
    if tif_path:
        print(f"Output GeoTIFF: {tif_path}")
    print(f"Sumber: Sentinel-2 SR (Cloud Score+ masked) | Scene: {count}")
    print(f"Algoritma: Dogliotti et al. 2015 (Red-NIR switchable)")
    print(f"Periode: {start_date} s/d {end_date}")
    print(f"\n=== Statistik Turbiditas ===")
    if mean_val is not None:
        print(f"  Rata-rata : {mean_val:.2f} FNU")
        print(f"  Minimum   : {min_val:.2f} FNU")
        print(f"  Maksimum  : {max_val:.2f} FNU")
    else:
        print("  Tidak ada piksel air yang terdeteksi.")


def estimate_chlorophyll(lat, lon, buffer_km, start_date, end_date, output_path):
    """Estimasi Klorofil-a menggunakan dua algoritma:
    1. NDCI (Mishra & Mishra 2012): NDCI = (B5-B4)/(B5+B4)
       Chl-a = 14.039 + 86.115*NDCI + 194.325*NDCI²
    2. OC3 adapted for S2 (O'Reilly et al. 2000, NASA OC3Mv6):
       R = log10(max(B2/B3, B3/B4))  — approx. log10(max(Rrs443/Rrs555, Rrs490/Rrs555))
       Chl-a = 10^(a0 + a1*R + a2*R² + a3*R³ + a4*R⁴)
       Coefficients: a0=0.2830, a1=-2.753, a2=1.457, a3=0.659, a4=-1.403
    Bands: B2(490nm), B3(560nm), B4(665nm), B5(705nm)
    """
    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    composite, count = _get_s2_composite(roi, start_date, end_date)
    if composite is None:
        print(f"ERROR: Tidak ada data Sentinel-2 untuk lokasi ini pada periode {start_date} - {end_date}")
        return

    # Band reflectances
    b2 = composite.select('B2').multiply(0.0001)   # 490nm
    b3 = composite.select('B3').multiply(0.0001)   # 560nm
    b4 = composite.select('B4').multiply(0.0001)   # 665nm
    b5 = composite.select('B5').multiply(0.0001)   # 705nm

    # === Algorithm 1: NDCI (Mishra & Mishra 2012) ===
    ndci = b5.subtract(b4).divide(b5.add(b4))
    # Chl-a = 14.039 + 86.115*NDCI + 194.325*NDCI²
    chl_ndci = ndci.multiply(86.115).add(14.039).add(
        ndci.pow(2).multiply(194.325)
    ).rename('result')

    # Clamp NDCI result
    chl_ndci = chl_ndci.where(chl_ndci.lt(0), 0).where(chl_ndci.gt(500), 500)

    # === Algorithm 2: OC3 — adapted for Sentinel-2 (O'Reilly et al. 2000, NASA OC3Mv6) ===
    # R = log10(max(Rrs443/Rrs555, Rrs490/Rrs555)) — for S2: log10(max(B2/B3, B3/B4)) approx.
    ratio_green = b3.divide(b4.where(b4.lte(0), 0.0001))
    ratio_blue = b2.divide(b4.where(b4.lte(0), 0.0001))
    max_ratio = ratio_green.max(ratio_blue)
    R = max_ratio.log10()

    # NASA OC3Mv6 coefficients (O'Reilly et al. 2000, adapted OC3 for S2)
    a0, a1, a2, a3, a4 = 0.2830, -2.753, 1.457, 0.659, -1.403
    # Chl-a = 10^(a0 + a1*R + a2*R² + a3*R³ + a4*R⁴)
    log_chl = R.multiply(a1).add(a0).add(
        R.pow(2).multiply(a2)
    ).add(
        R.pow(3).multiply(a3)
    ).add(
        R.pow(4).multiply(a4)
    )
    chl_oc3 = ee.Image(10).pow(log_chl).rename('result')

    # Clamp OC3 result
    chl_oc3 = chl_oc3.where(chl_oc3.lt(0), 0).where(chl_oc3.gt(500), 500)

    # Stats
    ndci_mean, ndci_min, ndci_max = _get_stats(chl_ndci, roi)
    oc3_mean, oc3_min, oc3_max = _get_stats(chl_oc3, roi)

    # GeoTIFF for NDCI (primary)
    tif_ndci = _save_geotiff(chl_ndci, roi, output_path.replace('.png', '_ndci.tif'), scale=10)
    tif_oc3 = _save_geotiff(chl_oc3, roi, output_path.replace('.png', '_oc3.tif'), scale=10)

    # PNG thumbnails
    vis_params = {
        'min': 0, 'max': 50,
        'palette': ['000080', '0000FF', '00BFFF', '00FF00', 'FFFF00', 'FF8C00', 'FF0000', '8B0000']
    }

    png_ndci = output_path.replace('.png', '_ndci.png') if '.png' in output_path else output_path + '_ndci.png'
    png_oc3 = output_path.replace('.png', '_oc3.png') if '.png' in output_path else output_path + '_oc3.png'
    _save_thumb(chl_ndci, roi, png_ndci, dict(vis_params))
    _save_thumb(chl_oc3, roi, png_oc3, dict(vis_params))

    # Multi-panel matplotlib figure
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 7))

    img1 = plt.imread(png_ndci)
    ax1.imshow(img1)
    ax1.set_title('Klorofil-a — NDCI\n(Mishra & Mishra 2012)', fontweight='bold')
    ax1.axis('off')

    img2 = plt.imread(png_oc3)
    ax2.imshow(img2)
    ax2.set_title('Klorofil-a — OC3\n(O\'Reilly et al. 2000, NASA OC3Mv6, adapted S2)', fontweight='bold')
    ax2.axis('off')

    cmap = mcolors.LinearSegmentedColormap.from_list('chl',
        ['#000080', '#0000FF', '#00BFFF', '#00FF00', '#FFFF00', '#FF8C00', '#FF0000', '#8B0000'])
    norm = plt.Normalize(vmin=0, vmax=50)
    sm = plt.cm.ScalarMappable(cmap=cmap, norm=norm)
    sm.set_array([])
    cbar = fig.colorbar(sm, ax=[ax1, ax2], fraction=0.046, pad=0.04)
    cbar.set_label('Klorofil-a (mg/m³)', fontsize=12)

    fig.suptitle(f'Estimasi Klorofil-a | Lat: {lat}, Lon: {lon} | {start_date} s/d {end_date}',
                 fontweight='bold', fontsize=13)
    plt.tight_layout()
    png_path = output_path if output_path.endswith('.png') else output_path.replace('.tif', '.png')
    plt.savefig(png_path, dpi=200, bbox_inches='tight')
    plt.close()

    # Provenance metadata
    try:
        create_provenance(png_path,
            tool='estimate_chlorophyll', gee_collection='COPERNICUS/S2_SR_HARMONIZED',
            date_range=[start_date, end_date],
            coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
            algorithms=['NDCI Mishra & Mishra 2012', 'OC3 O\'Reilly et al. 2000, NASA OC3Mv6 (adapted S2)', 'Cloud Score+ masking', 'MNDWI water mask'],
            references=['Mishra & Mishra 2012', 'O\'Reilly et al. 2000, NASA OC3Mv6'],
            crs='EPSG:4326', scale_m=10)
    except:
        pass  # provenance is non-critical

    # Print results
    print(f"SUCCESS: Estimasi Klorofil-a selesai.")
    print("DISCLAIMER: S2 SR bukan water-leaving reflectance. Hasil approximate. Validasi dengan sampling in-situ.")
    print(f"Output PNG (gabungan): {png_path}")
    if tif_ndci:
        print(f"GeoTIFF NDCI: {tif_ndci}")
    if tif_oc3:
        print(f"GeoTIFF OC3: {tif_oc3}")
    print(f"Sumber: Sentinel-2 SR (Cloud Score+ masked) | Scene: {count}")
    print(f"Periode: {start_date} s/d {end_date}")

    print(f"\n=== Statistik Klorofil-a (NDCI — Mishra & Mishra 2012) ===")
    if ndci_mean is not None:
        print(f"  Rata-rata : {ndci_mean:.2f} mg/m³")
        print(f"  Minimum   : {ndci_min:.2f} mg/m³")
        print(f"  Maksimum  : {ndci_max:.2f} mg/m³")
    else:
        print("  Tidak ada piksel air yang terdeteksi.")

    print(f"\n=== Statistik Klorofil-a (OC3 — O'Reilly et al. 2000, NASA OC3Mv6, adapted S2) ===")
    if oc3_mean is not None:
        print(f"  Rata-rata : {oc3_mean:.2f} mg/m³")
        print(f"  Minimum   : {oc3_min:.2f} mg/m³")
        print(f"  Maksimum  : {oc3_max:.2f} mg/m³")
    else:
        print("  Tidak ada piksel air yang terdeteksi.")

    # Compare with KepMenLH 51/2004
    print(f"\n=== Perbandingan Baku Mutu Air Laut (KepMenLH 51/2004) ===")
    # Convert mg/m³ to mg/L (1 mg/m³ = 0.001 mg/L)
    if ndci_mean is not None:
        ndci_mg_l = ndci_mean * 0.001
        for kategori, limit in KEPMENLH51_CHL.items():
            status = "MEMENUHI" if ndci_mg_l <= limit else "MELEBIHI"
            print(f"  {kategori} (Chl-a ≤ {limit*1000:.0f} µg/L): {status} (NDCI: {ndci_mean:.2f} mg/m³ = {ndci_mg_l*1000:.1f} µg/L)")


def water_quality_composite(lat, lon, buffer_km, start_date, end_date, output_path):
    """Komposit kualitas air: TSS + Turbiditas + Klorofil-a dalam satu panggilan.
    Menghasilkan figure 3-panel dan penilaian kualitas air menyeluruh.
    """
    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    composite, count = _get_s2_composite(roi, start_date, end_date)
    if composite is None:
        print(f"ERROR: Tidak ada data Sentinel-2 untuk lokasi ini pada periode {start_date} - {end_date}")
        return

    # === Band reflectances ===
    b2 = composite.select('B2').multiply(0.0001)
    b3 = composite.select('B3').multiply(0.0001)
    b4 = composite.select('B4').multiply(0.0001)
    b5 = composite.select('B5').multiply(0.0001)
    b8 = composite.select('B8').multiply(0.0001)
    rho_red = b4
    rho_nir = b8

    # === 1. TSS (Nechad 2010) ===
    A_T, C_T = 355.85, 0.1728
    tss = rho_red.multiply(A_T).divide(
        ee.Image(1).subtract(rho_red.divide(C_T))
    ).rename('result')
    tss = tss.where(tss.lt(0), 0).where(tss.gt(5000), 5000)

    # === 2. Turbidity (Dogliotti 2015) ===
    t_low = rho_red.multiply(228.1).divide(
        ee.Image(1).subtract(rho_red.divide(0.1641))
    )
    t_high = rho_nir.multiply(3078.9).divide(
        ee.Image(1).subtract(rho_nir.divide(0.2112))
    )
    w = t_low.subtract(7).divide(8).clamp(0, 1)
    turbidity = t_low.multiply(ee.Image(1).subtract(w)).add(t_high.multiply(w)).rename('result')
    turbidity = turbidity.where(turbidity.lt(0), 0).where(turbidity.gt(3000), 3000)

    # === 3. Chlorophyll-a NDCI (Mishra 2012) ===
    ndci = b5.subtract(b4).divide(b5.add(b4))
    chl_ndci = ndci.multiply(86.115).add(14.039).add(
        ndci.pow(2).multiply(194.325)
    ).rename('result')
    chl_ndci = chl_ndci.where(chl_ndci.lt(0), 0).where(chl_ndci.gt(500), 500)

    # Stats
    tss_mean, tss_min, tss_max = _get_stats(tss, roi)
    turb_mean, turb_min, turb_max = _get_stats(turbidity, roi)
    chl_mean, chl_min, chl_max = _get_stats(chl_ndci, roi)

    # Individual GeoTIFFs
    base = output_path.replace('.png', '')
    tif_tss = _save_geotiff(tss, roi, f"{base}_tss.tif", scale=10)
    tif_turb = _save_geotiff(turbidity, roi, f"{base}_turbidity.tif", scale=10)
    tif_chl = _save_geotiff(chl_ndci, roi, f"{base}_chla.tif", scale=10)

    # PNG thumbnails for each parameter
    tss_vis = {'min': 0, 'max': 200,
               'palette': ['001f3f', '0074D9', '7FDBFF', '2ECC40', 'FFDC00', 'FF851B', 'FF4136']}
    turb_vis = {'min': 0, 'max': 100,
                'palette': ['0000FF', '00BFFF', '00FF00', 'ADFF2F', 'FFFF00', 'FFA500', 'FF0000']}
    chl_vis = {'min': 0, 'max': 50,
               'palette': ['000080', '0000FF', '00BFFF', '00FF00', 'FFFF00', 'FF8C00', 'FF0000', '8B0000']}

    png_tss = f"{base}_tss_thumb.png"
    png_turb = f"{base}_turb_thumb.png"
    png_chl = f"{base}_chl_thumb.png"
    _save_thumb(tss, roi, png_tss, dict(tss_vis))
    _save_thumb(turbidity, roi, png_turb, dict(turb_vis))
    _save_thumb(chl_ndci, roi, png_chl, dict(chl_vis))

    # === 3-panel matplotlib figure ===
    fig, axes = plt.subplots(1, 3, figsize=(20, 7))

    panels = [
        (axes[0], png_tss, 'TSS (mg/L)\nNechad et al. 2010',
         ['#001f3f', '#0074D9', '#7FDBFF', '#2ECC40', '#FFDC00', '#FF851B', '#FF4136'],
         0, 200, 'TSS (mg/L)'),
        (axes[1], png_turb, 'Turbiditas (FNU)\nDogliotti et al. 2015',
         ['#0000FF', '#00BFFF', '#00FF00', '#ADFF2F', '#FFFF00', '#FFA500', '#FF0000'],
         0, 100, 'Turbiditas (FNU)'),
        (axes[2], png_chl, 'Klorofil-a (mg/m³)\nMishra & Mishra 2012',
         ['#000080', '#0000FF', '#00BFFF', '#00FF00', '#FFFF00', '#FF8C00', '#FF0000', '#8B0000'],
         0, 50, 'Klorofil-a (mg/m³)'),
    ]

    for ax, thumb_path, title, palette, vmin, vmax, label in panels:
        img_data = plt.imread(thumb_path)
        ax.imshow(img_data)
        ax.set_title(title, fontweight='bold', fontsize=11)
        ax.axis('off')

        cmap = mcolors.LinearSegmentedColormap.from_list('', palette)
        norm = plt.Normalize(vmin=vmin, vmax=vmax)
        sm = plt.cm.ScalarMappable(cmap=cmap, norm=norm)
        sm.set_array([])
        cbar = fig.colorbar(sm, ax=ax, fraction=0.046, pad=0.04)
        cbar.set_label(label, fontsize=10)

    fig.suptitle(f'Komposit Kualitas Air Satelit | Lat: {lat}, Lon: {lon}\n'
                 f'{start_date} s/d {end_date} | Sentinel-2 SR ({count} scene)',
                 fontweight='bold', fontsize=14)
    plt.tight_layout()
    png_path = output_path if output_path.endswith('.png') else f"{base}_composite.png"
    plt.savefig(png_path, dpi=200, bbox_inches='tight')
    plt.close()

    # Cleanup temp thumbnails
    for tmp in [png_tss, png_turb, png_chl]:
        try:
            os.remove(tmp)
        except OSError:
            pass

    # Print results
    print(f"SUCCESS: Komposit kualitas air selesai.")
    print("DISCLAIMER: S2 SR bukan water-leaving reflectance. Hasil approximate. Validasi dengan sampling in-situ.")
    print(f"Output PNG (3-panel): {png_path}")
    print(f"GeoTIFF TSS: {tif_tss}")
    print(f"GeoTIFF Turbiditas: {tif_turb}")
    print(f"GeoTIFF Klorofil-a: {tif_chl}")
    print(f"Sumber: Sentinel-2 SR (Cloud Score+ masked) | Scene: {count}")
    print(f"Periode: {start_date} s/d {end_date}")

    print(f"\n{'='*60}")
    print(f"  RINGKASAN KUALITAS AIR")
    print(f"{'='*60}")

    if tss_mean is not None:
        print(f"\n  TSS (Nechad 2010):")
        print(f"    Rata-rata: {tss_mean:.2f} mg/L | Min: {tss_min:.2f} | Maks: {tss_max:.2f}")
    if turb_mean is not None:
        print(f"\n  Turbiditas (Dogliotti 2015):")
        print(f"    Rata-rata: {turb_mean:.2f} FNU | Min: {turb_min:.2f} | Maks: {turb_max:.2f}")
    if chl_mean is not None:
        print(f"\n  Klorofil-a NDCI (Mishra 2012):")
        print(f"    Rata-rata: {chl_mean:.2f} mg/m³ | Min: {chl_min:.2f} | Maks: {chl_max:.2f}")

    # === Overall assessment vs Indonesian regulations ===
    print(f"\n{'='*60}")
    print(f"  PENILAIAN TERHADAP REGULASI INDONESIA")
    print(f"{'='*60}")

    if tss_mean is not None:
        print(f"\n  [PP 22/2021 — Baku Mutu Air]")
        for kelas, limit in PP22_TSS_LIMITS.items():
            status = "MEMENUHI" if tss_mean <= limit else "MELEBIHI"
            icon = "✓" if tss_mean <= limit else "✗"
            print(f"    {icon} {kelas} — TSS ≤ {limit} mg/L: {status}")

    if chl_mean is not None:
        chl_ug_l = chl_mean  # mg/m³ ≈ µg/L
        print(f"\n  [KepMenLH 51/2004 — Baku Mutu Air Laut]")
        for kategori, limit_mg_l in KEPMENLH51_CHL.items():
            limit_ug_l = limit_mg_l * 1000
            status = "MEMENUHI" if chl_ug_l <= limit_ug_l else "MELEBIHI"
            icon = "✓" if chl_ug_l <= limit_ug_l else "✗"
            print(f"    {icon} {kategori} — Chl-a ≤ {limit_ug_l:.0f} µg/L: {status} ({chl_ug_l:.1f} µg/L)")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Penggunaan: python3 water_quality_engine.py <mode> <lat> <lon> <buffer_km> <start_date> <end_date> <output_path>")
        print("Mode: tss | turbidity | chlorophyll | composite")
        print("Contoh: python3 water_quality_engine.py tss -6.9 110.4 5 2024-01-01 2024-03-31 output.png")
        sys.exit(1)

    mode = sys.argv[1]
    try:
        lat = float(sys.argv[2])
        lon = float(sys.argv[3])
        buffer_km = float(sys.argv[4])
        start_date = sys.argv[5]
        end_date = sys.argv[6]
        output_path = sys.argv[7]

        if mode == 'tss':
            estimate_tss(lat, lon, buffer_km, start_date, end_date, output_path)
        elif mode == 'turbidity':
            estimate_turbidity(lat, lon, buffer_km, start_date, end_date, output_path)
        elif mode == 'chlorophyll':
            estimate_chlorophyll(lat, lon, buffer_km, start_date, end_date, output_path)
        elif mode == 'composite':
            water_quality_composite(lat, lon, buffer_km, start_date, end_date, output_path)
        else:
            print(f"ERROR: Mode tidak dikenal: '{mode}'")
            print("Mode yang tersedia: tss, turbidity, chlorophyll, composite")
            sys.exit(1)
    except IndexError:
        print("ERROR: Argumen tidak lengkap.")
        print("Penggunaan: python3 water_quality_engine.py <mode> <lat> <lon> <buffer_km> <start_date> <end_date> <output_path>")
        sys.exit(1)
    except Exception as e:
        print(f"ERROR [E502]: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
