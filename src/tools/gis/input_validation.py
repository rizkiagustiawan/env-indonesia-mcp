#!/usr/bin/env python3
"""Input Validation for all Python engines."""
import os
from datetime import datetime

__version__ = '1.0.0'

def validate_date_range(start_date, end_date):
    try:
        s = datetime.strptime(start_date, '%Y-%m-%d')
        e = datetime.strptime(end_date, '%Y-%m-%d')
        if e <= s: return False, f"End date ({end_date}) harus setelah start date ({start_date})"
        if (e - s).days > 3650: return False, "Range maksimal 10 tahun"
        return True, "OK"
    except ValueError as ex:
        return False, f"Format tanggal tidak valid (YYYY-MM-DD): {ex}"

def validate_buffer(buffer_km):
    if buffer_km <= 0: return False, "Buffer harus > 0"
    if buffer_km > 200: return False, f"Buffer {buffer_km}km terlalu besar. Maks 200km."
    return True, "OK"

def validate_output_path(output_path):
    d = os.path.dirname(output_path) or '.'
    if not os.path.isdir(d): return False, f"Direktori output tidak ada: {d}"
    return True, "OK"

def validate_coords_indonesia(lat, lon):
    if lat < -11.5 or lat > 6.0: return False, f"Latitude {lat} di luar Indonesia (-11.5 s/d 6.0)"
    if lon < 95.0 or lon > 141.5: return False, f"Longitude {lon} di luar Indonesia (95.0 s/d 141.5)"
    return True, "OK"

def validate_all(lat, lon, buffer_km=None, start_date=None, end_date=None, output_path=None):
    ok, msg = validate_coords_indonesia(lat, lon)
    if not ok: return False, msg
    if buffer_km is not None:
        ok, msg = validate_buffer(buffer_km)
        if not ok: return False, msg
    if start_date and end_date:
        ok, msg = validate_date_range(start_date, end_date)
        if not ok: return False, msg
    if output_path:
        ok, msg = validate_output_path(output_path)
        if not ok: return False, msg
    return True, "OK"
