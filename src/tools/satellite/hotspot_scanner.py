#!/usr/bin/env python3
"""
Hotspot Auto-Scanner Indonesia
Scans NASA VIIRS active fire data across Indonesia via Google Earth Engine.
Returns hotspot locations with FRP values and estimated provinces.
"""

import ee
import json
import argparse
from datetime import datetime, timedelta, timezone

# Indonesia bounding box
INDONESIA_BBOX = [95.0, -11.0, 141.0, 6.0]

# Province estimation based on longitude ranges
PROVINCE_RANGES = [
    (95.0, 98.0, "Aceh"),
    (98.0, 100.0, "Sumatera Utara"),
    (100.0, 105.0, "Riau/Jambi/Sumatera Selatan"),
    (105.0, 108.0, "Jawa"),
    (108.0, 115.0, "Kalimantan"),
    (115.0, 120.0, "Sulawesi"),
    (120.0, 141.0, "Papua/Maluku/NTT"),
]


def estimate_province(lon: float) -> str:
    """Estimate province from longitude (rough approximation)."""
    for lo, hi, name in PROVINCE_RANGES:
        if lo <= lon < hi:
            return name
    return "Unknown"


def scan_indonesia_hotspots(min_frp: float = 10, days_back: int = 3) -> list:
    """
    Scan Indonesia for active fire hotspots using VIIRS data.

    Args:
        min_frp: Minimum Fire Radiative Power in MW to include.
        days_back: Number of days back from today to search.

    Returns:
        List of dicts with keys: lat, lon, frp_mw, province_estimate
    """
    ee.Initialize()

    roi = ee.Geometry.Rectangle(INDONESIA_BBOX)

    end_date = datetime.now(timezone.utc)
    start_date = end_date - timedelta(days=days_back)
    start_str = start_date.strftime("%Y-%m-%d")
    end_str = end_date.strftime("%Y-%m-%d")

    # Query SNPP VIIRS
    snpp = (
        ee.ImageCollection("NASA/LANCE/SNPP_VIIRS/C2")
        .filterDate(start_str, end_str)
        .filterBounds(roi)
        .select("frp")
    )

    # Query NOAA-20 VIIRS
    noaa20 = (
        ee.ImageCollection("NASA/LANCE/NOAA20_VIIRS/C2")
        .filterDate(start_str, end_str)
        .filterBounds(roi)
        .select("frp")
    )

    # Merge both collections
    merged = snpp.merge(noaa20)

    # Reduce to max FRP per pixel
    max_frp = merged.max().clip(roi)

    # Apply minimum FRP threshold
    hotspot_mask = max_frp.gte(min_frp)
    hotspots = hotspot_mask

    # Reduce to vectors
    vectors = hotspots.reduceToVectors(
        geometry=roi,
        scale=3750,
        maxPixels=int(1e9),
        geometryType="centroid",
        eightConnected=False,
        reducer=ee.Reducer.countEvery(),
    )

    # Get features (limit to 20)
    features = vectors.limit(20).getInfo()

    results = []
    for feat in features.get("features", []):
        coords = feat["geometry"]["coordinates"]
        lon, lat = coords[0], coords[1]
        frp_val = feat["properties"].get("max", feat["properties"].get("label", 0))

        results.append(
            {
                "lat": round(lat, 4),
                "lon": round(lon, 4),
                "frp_mw": round(float(frp_val), 2),
                "province_estimate": estimate_province(lon),
            }
        )

    # Sort by FRP descending
    results.sort(key=lambda x: x["frp_mw"], reverse=True)
    return results


def scan_and_alert(min_frp: float = 50) -> str:
    """
    Scan hotspots and format as alert report text.

    Args:
        min_frp: Minimum FRP threshold for alert-level hotspots.

    Returns:
        Formatted text report string.
    """
    hotspots = scan_indonesia_hotspots(min_frp=min_frp, days_back=3)

    if not hotspots:
        return (
            f"[HOTSPOT REPORT] Tidak ada titik panas terdeteksi "
            f"dengan FRP >= {min_frp} MW dalam 3 hari terakhir."
        )

    lines = [
        f"[HOTSPOT REPORT] {len(hotspots)} titik panas terdeteksi (FRP >= {min_frp} MW)",
        f"Periode: 3 hari terakhir",
        "-" * 60,
    ]

    for i, h in enumerate(hotspots, 1):
        lines.append(
            f"  {i}. {h['province_estimate']}: "
            f"({h['lat']}, {h['lon']}) - FRP: {h['frp_mw']} MW"
        )

    high_frp = [h for h in hotspots if h["frp_mw"] >= 100]
    if high_frp:
        lines.append("")
        lines.append(f"⚠ {len(high_frp)} titik dengan FRP >= 100 MW (intensitas tinggi)")

    return "\n".join(lines)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Indonesia Hotspot Auto-Scanner")
    parser.add_argument(
        "--min_frp", type=float, default=10, help="Minimum FRP in MW (default: 10)"
    )
    parser.add_argument(
        "--days_back", type=int, default=3, help="Days back to scan (default: 3)"
    )
    args = parser.parse_args()

    result = scan_indonesia_hotspots(min_frp=args.min_frp, days_back=args.days_back)
    print(json.dumps(result, indent=2))
