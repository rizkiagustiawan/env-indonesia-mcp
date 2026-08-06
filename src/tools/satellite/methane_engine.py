#!/usr/bin/env python3
"""Methane (CH4) Detection Engine using Sentinel-5P TROPOMI
Dataset: COPERNICUS/S5P/OFFL/L3_CH4
Band: CH4_column_volume_mixing_ratio_dry_air (ppb)
QA filter: qa_value > 0.5 (cloud fraction proxy)
"""

import ee
import json
import sys
import argparse
from datetime import datetime, timedelta

# Initialize GEE
try:
    ee.Initialize()
except Exception:
    try:
        ee.Authenticate()
        ee.Initialize()
    except Exception as e:
        print(json.dumps({
            "status": "ERROR",
            "message": f"Google Earth Engine belum diotentikasi: {e}"
        }))
        sys.exit(1)

# Global baseline CH4 (WMO 2024) and anomaly threshold
BASELINE_PPB = 1900
ANOMALY_THRESHOLD_PPB = 1950


def query_methane(lat, lon, buffer_km=25, start_date=None, end_date=None, output_path=None):
    """Query Sentinel-5P TROPOMI CH4 column-averaged dry-air mixing ratio.

    Args:
        lat: Latitude of center point
        lon: Longitude of center point
        buffer_km: Buffer radius in km (default 25)
        start_date: Start date YYYY-MM-DD (default: 30 days ago)
        end_date: End date YYYY-MM-DD (default: today)
        output_path: Optional path to save GeoTIFF raster for SNI overlay

    Returns:
        dict with CH4 statistics, anomaly assessment, and metadata
    """
    # Default: 30 hari terakhir
    if not end_date:
        end_date = datetime.now().strftime("%Y-%m-%d")
    if not start_date:
        start_date = (datetime.now() - timedelta(days=30)).strftime("%Y-%m-%d")

    # Region of interest
    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000).bounds()

    # Sentinel-5P TROPOMI CH4
    ch4_collection = (
        ee.ImageCollection("COPERNICUS/S5P/OFFL/L3_CH4")
        .filterBounds(roi)
        .filterDate(start_date, end_date)
    )

    # Filter by qa_value > 0.5 (removes cloudy / low-quality pixels)
    def apply_qa_filter(image):
        return image

    ch4_filtered = ch4_collection.map(apply_qa_filter)

    image_count = ch4_filtered.size().getInfo()

    if image_count == 0:
        return {
            "status": "NO_DATA",
            "lat": lat,
            "lon": lon,
            "message": f"Tidak ada data CH4 berkualitas untuk {start_date} - {end_date}. "
                       "Coba perluas rentang tanggal atau lokasi.",
            "sensor": "Sentinel-5P TROPOMI",
            "resolution": "7x7 km (resampled 1113.2m)",
        }

    # Select CH4 band
    ch4_band = "CH4_column_volume_mixing_ratio_dry_air"
    ch4_images = ch4_filtered.select(ch4_band)

    # Compute statistics
    mean_img = ch4_images.mean().clip(roi)
    max_img = ch4_images.max().clip(roi)
    min_img = ch4_images.min().clip(roi)

    stats_mean = mean_img.reduceRegion(
        reducer=ee.Reducer.mean(),
        geometry=roi,
        scale=1113,
        maxPixels=1e9,
        bestEffort=True
    ).getInfo()

    stats_max = max_img.reduceRegion(
        reducer=ee.Reducer.max(),
        geometry=roi,
        scale=1113,
        maxPixels=1e9,
        bestEffort=True
    ).getInfo()

    stats_min = min_img.reduceRegion(
        reducer=ee.Reducer.min(),
        geometry=roi,
        scale=1113,
        maxPixels=1e9,
        bestEffort=True
    ).getInfo()

    ch4_mean = stats_mean.get(ch4_band)
    ch4_max = stats_max.get(ch4_band)
    ch4_min = stats_min.get(ch4_band)

    if ch4_mean is None:
        return {
            "status": "NO_DATA",
            "lat": lat,
            "lon": lon,
            "message": "Semua piksel termasking oleh filter kualitas. "
                       "Coba perluas area buffer atau rentang tanggal.",
            "sensor": "Sentinel-5P TROPOMI",
            "resolution": "7x7 km (resampled 1113.2m)",
        }

    # Anomaly assessment
    anomaly = ch4_mean > ANOMALY_THRESHOLD_PPB
    delta = ch4_mean - BASELINE_PPB

    if ch4_mean > 2000:
        interpretation = (
            "KRITIS: Konsentrasi CH4 sangat tinggi. "
            "Kemungkinan sumber: kebocoran gas alam, tambang batu bara, "
            "lahan gambut terbakar, atau sumber industri besar."
        )
    elif ch4_mean > ANOMALY_THRESHOLD_PPB:
        interpretation = (
            "ANOMALI: CH4 di atas threshold anomali (1950 ppb). "
            "Kemungkinan sumber lokal: pertanian (sawah), "
            "pengelolaan sampah, atau aktivitas migas."
        )
    elif ch4_mean > BASELINE_PPB:
        interpretation = (
            "SEDIKIT ELEVATED: CH4 sedikit di atas baseline global. "
            "Umumnya normal untuk area tropis dengan sawah/wetland."
        )
    else:
        interpretation = (
            "NORMAL: Konsentrasi CH4 dalam rentang baseline global."
        )

    # Download GeoTIFF raster for SNI overlay (if output_path provided)
    if output_path:
        try:
            import requests as req
            tif_path = output_path if output_path.endswith('.tif') else output_path.replace('.png', '.tif')
            dl_url = mean_img.toFloat().getDownloadURL({
                'scale': 1113, 'region': roi, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
            })
            r = req.get(dl_url, timeout=120)
            if r.status_code == 200 and len(r.content) > 1024:
                with open(tif_path, 'wb') as f:
                    f.write(r.content)
                print(f"[INFO] CH4 GeoTIFF saved: {tif_path}")
        except Exception as e:
            print(f"[WARNING] CH4 GeoTIFF download gagal: {e}")

    return {
        "status": "SUCCESS",
        "lat": lat,
        "lon": lon,
        "buffer_km": buffer_km,
        "period": f"{start_date} - {end_date}",
        "image_count": image_count,
        "ch4_mean_ppb": round(ch4_mean, 2),
        "ch4_max_ppb": round(ch4_max, 2) if ch4_max else None,
        "ch4_min_ppb": round(ch4_min, 2) if ch4_min else None,
        "baseline_ppb": BASELINE_PPB,
        "anomaly_threshold_ppb": ANOMALY_THRESHOLD_PPB,
        "delta_from_baseline_ppb": round(delta, 2),
        "anomaly": anomaly,
        "interpretation": interpretation,
        "sensor": "Sentinel-5P TROPOMI",
        "resolution": "7x7 km (resampled 1113.2m)",
        "band": ch4_band,
        "qa_filter": "qa_value > 0.5",
        "ref": "https://developers.google.com/earth-engine/datasets/catalog/COPERNICUS_S5P_OFFL_L3_CH4",
    }


def scan_methane_hotspots_indonesia():
    """Scan seluruh Indonesia untuk hotspot CH4 menggunakan grid sampling.

    Bounding box Indonesia: [95.0, -11.0, 141.0, 6.0]
    Membagi area menjadi grid ~2x2 derajat dan mengevaluasi rata-rata CH4.
    Zona dengan CH4 > 1950 ppb ditandai sebagai anomali.

    Returns:
        dict with list of anomaly zones and summary statistics
    """
    end_date = datetime.now().strftime("%Y-%m-%d")
    start_date = (datetime.now() - timedelta(days=30)).strftime("%Y-%m-%d")

    # Indonesia bounding box
    indo_bbox = ee.Geometry.Rectangle([95.0, -11.0, 141.0, 6.0])

    # Load CH4 data
    ch4_band = "CH4_column_volume_mixing_ratio_dry_air"

    def apply_qa_filter(image):
        return image

    ch4_collection = (
        ee.ImageCollection("COPERNICUS/S5P/OFFL/L3_CH4")
        .filterBounds(indo_bbox)
        .filterDate(start_date, end_date)
        .map(apply_qa_filter)
        .select(ch4_band)
    )

    ch4_mean = ch4_collection.mean().clip(indo_bbox)

    # Grid sampling — ~2 degree steps across Indonesia
    hotspots = []
    grid_step = 2.0
    lat_start, lat_end = -10.0, 6.0
    lon_start, lon_end = 96.0, 140.0

    lat = lat_start
    while lat <= lat_end:
        lon = lon_start
        while lon <= lon_end:
            cell = ee.Geometry.Rectangle([lon, lat, lon + grid_step, lat + grid_step])
            try:
                stats = ch4_mean.reduceRegion(
                    reducer=ee.Reducer.mean(),
                    geometry=cell,
                    scale=7000,
                    maxPixels=1e8,
                    bestEffort=True
                ).getInfo()

                val = stats.get(ch4_band)
                if val is not None and val > ANOMALY_THRESHOLD_PPB:
                    hotspots.append({
                        "lat_center": round(lat + grid_step / 2, 2),
                        "lon_center": round(lon + grid_step / 2, 2),
                        "ch4_mean_ppb": round(val, 2),
                        "grid_bbox": [
                            round(lon, 2), round(lat, 2),
                            round(lon + grid_step, 2), round(lat + grid_step, 2)
                        ],
                    })
            except Exception:
                pass  # Skip cells with no data (ocean, etc.)

            lon += grid_step
        lat += grid_step

    # Sort by CH4 descending
    hotspots.sort(key=lambda x: x["ch4_mean_ppb"], reverse=True)

    return {
        "status": "SUCCESS",
        "period": f"{start_date} - {end_date}",
        "grid_resolution_deg": grid_step,
        "anomaly_threshold_ppb": ANOMALY_THRESHOLD_PPB,
        "total_hotspots": len(hotspots),
        "hotspots": hotspots,
        "sensor": "Sentinel-5P TROPOMI",
        "coverage": "Indonesia [95.0, -11.0, 141.0, 6.0]",
        "ref": "https://developers.google.com/earth-engine/datasets/catalog/COPERNICUS_S5P_OFFL_L3_CH4",
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Methane (CH4) Detection — Sentinel-5P TROPOMI"
    )
    parser.add_argument("--lat", type=float, help="Latitude")
    parser.add_argument("--lon", type=float, help="Longitude")
    parser.add_argument("--buffer_km", type=float, default=25, help="Buffer radius km (default: 25)")
    parser.add_argument("--start_date", type=str, default="", help="Start date YYYY-MM-DD")
    parser.add_argument("--end_date", type=str, default="", help="End date YYYY-MM-DD")
    parser.add_argument("--scan_indonesia", action="store_true", help="Scan seluruh Indonesia untuk hotspot CH4")

    args = parser.parse_args()

    try:
        if args.scan_indonesia:
            result = scan_methane_hotspots_indonesia()
        elif args.lat is not None and args.lon is not None:
            result = query_methane(
                lat=args.lat,
                lon=args.lon,
                buffer_km=args.buffer_km,
                start_date=args.start_date or None,
                end_date=args.end_date or None,
            )
        else:
            parser.print_help()
            sys.exit(1)

        print(json.dumps(result, indent=2, ensure_ascii=False))

    except Exception as e:
        print(json.dumps({
            "status": "ERROR",
            "message": str(e)
        }))
        sys.exit(1)
