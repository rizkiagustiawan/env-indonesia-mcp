#!/usr/bin/env python3
"""HYCOM Ocean Current Fetcher for Oil Spill Trajectory
Mengambil data arus laut real-time dari HYCOM GLBu0.08 via OPeNDAP/GEE.
Ref: Chassignet et al. (2007), HYCOM Consortium
"""
import sys, json, argparse
from datetime import datetime, timedelta

def fetch_hycom_currents(lat, lon, buffer_km=50):
    """Ambil arus laut dari GEE dataset HYCOM."""
    import ee
    ee.Initialize()

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000).bounds()

    end_date = datetime.now().strftime('%Y-%m-%d')
    start_date = (datetime.now() - timedelta(days=7)).strftime('%Y-%m-%d')

    hycom = ee.ImageCollection('HYCOM/GLBu0_08/sea_water_velocity') \
        .filterDate(start_date, end_date) \
        .filterBounds(roi)

    count = hycom.size().getInfo()
    if count == 0:
        return json.dumps({"status": "NO_DATA", "message": f"HYCOM tidak tersedia untuk {lat}, {lon} periode {start_date} - {end_date}"}, indent=2)

    latest = hycom.sort('system:time_start', False).first()

    try:
        vals = latest.reduceRegion(
            reducer=ee.Reducer.mean(),
            geometry=roi,
            scale=9000
        ).getInfo()

        u = vals.get('velocity_u_0', 0) or 0
        v = vals.get('velocity_v_0', 0) or 0

        import math
        speed = math.sqrt(u**2 + v**2)
        direction = math.degrees(math.atan2(v, u)) % 360

        timestamp = ee.Date(latest.get('system:time_start')).format('YYYY-MM-dd HH:mm').getInfo()

        return json.dumps({
            "status": "SUCCESS",
            "lat": lat, "lon": lon,
            "u_velocity_ms": round(u, 4),
            "v_velocity_ms": round(v, 4),
            "current_speed_ms": round(speed, 4),
            "current_direction_deg": round(direction, 1),
            "timestamp": timestamp,
            "source": "HYCOM GLBu0.08 via GEE",
            "resolution": "1/12 degree (~9km)",
            "ref": "Chassignet et al. 2007"
        }, indent=2)
    except Exception as e:
        return json.dumps({"status": "ERROR", "message": str(e)}, indent=2)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--lat', type=float, required=True)
    parser.add_argument('--lon', type=float, required=True)
    parser.add_argument('--buffer_km', type=float, default=50)
    args = parser.parse_args()
    print(fetch_hycom_currents(args.lat, args.lon, args.buffer_km))
