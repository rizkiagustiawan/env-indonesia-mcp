#!/usr/bin/env python3
"""
Satellite 4D Timelapse Engine
Men-generate GIF animasi dari citra satelit (Optik Sentinel-2 / Radar Sentinel-1)
Menggunakan GEE, Cloud Masking Probabilistik, dan ImageIO.
"""

import sys
import argparse
import ee
import os
import requests
import imageio.v2 as imageio
from PIL import Image, ImageDraw, ImageFont
import numpy as np
from datetime import datetime
from dateutil.relativedelta import relativedelta
from io import BytesIO

# Initialize GEE
try:
    ee.Initialize()
except Exception:
    print("ERROR: Google Earth Engine belum diotentikasi.")
    sys.exit(1)

def get_s2_cloudless_composite(roi, start_date, end_date):
    """Mendapatkan citra Sentinel-2 bebas awan dengan s2cloudless & percentile."""
    # Load Sentinel-2 and Cloud Probability
    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED').filterBounds(roi).filterDate(start_date, end_date)
    s2_cloudless = ee.ImageCollection('COPERNICUS/S2_CLOUD_PROBABILITY').filterBounds(roi).filterDate(start_date, end_date)

    # Join collections
    inner_join = ee.Join.inner()
    join_filter = ee.Filter.equals(leftField='system:index', rightField='system:index')
    joined = inner_join.apply(s2, s2_cloudless, join_filter)

    def mask_clouds(image):
        img = ee.Image(image.get('primary'))
        cld_prb = ee.Image(image.get('secondary')).select('probability')
        is_cloud = cld_prb.gt(30) # Probabilitas awan > 30% dianggap awan
        return img.updateMask(is_cloud.Not())

    # Map function, lalu ambil 25th percentile untuk membuang sisa awan putih/kabut
    cloud_masked = ee.ImageCollection(joined.map(mask_clouds))
    
    # Jika kosong (seluruh bulan awan), fallback ke median biasa
    if cloud_masked.size().getInfo() == 0:
        composite = s2.median()
    else:
        composite = cloud_masked.reduce(ee.Reducer.percentile([25]))
        # Rename bands back to normal
        composite = composite.select(['B4_p25', 'B3_p25', 'B2_p25'], ['B4', 'B3', 'B2'])

    # Visualisasi (True Color)
    vis = composite.visualize(bands=['B4', 'B3', 'B2'], min=0, max=2500, gamma=1.2)
    return vis

def get_s1_radar_composite(roi, start_date, end_date):
    """Mendapatkan citra Radar Sentinel-1 (Tembus awan)."""
    s1 = ee.ImageCollection('COPERNICUS/S1_GRD') \
        .filterBounds(roi) \
        .filterDate(start_date, end_date) \
        .filter(ee.Filter.listContains('transmitterReceiverPolarisation', 'VV')) \
        .filter(ee.Filter.listContains('transmitterReceiverPolarisation', 'VH')) \
        .filter(ee.Filter.eq('instrumentMode', 'IW'))

    # Median over the month to reduce speckle noise
    composite = s1.median()

    # Create False Color: R=VV, G=VH, B=VV/VH
    vv = composite.select('VV')
    vh = composite.select('VH')
    vv_vh = vv.subtract(vh).rename('VV_VH') # in dB, division is subtraction

    img = ee.Image.cat([vv, vh, vv_vh])
    
    # Visualisasi (False Color)
    vis = img.visualize(bands=['VV', 'VH', 'VV_VH'], min=[-20, -25, 0], max=[0, -5, 15], gamma=1.0)
    return vis

def add_watermark(image_bytes, text_top, text_bottom):
    """Menambahkan timestamp ke frame gambar."""
    img = Image.open(BytesIO(image_bytes)).convert("RGB")
    draw = ImageDraw.Draw(img)
    
    # Coba gunakan font default
    try:
        font_large = ImageFont.truetype("DejaVuSans-Bold.ttf", 24)
        font_small = ImageFont.truetype("DejaVuSans.ttf", 16)
    except:
        font_large = ImageFont.load_default()
        font_small = ImageFont.load_default()

    # Draw black background rectangle for text
    draw.rectangle([(10, 10), (250, 40)], fill=(0, 0, 0, 180))
    draw.text((15, 15), text_top, fill=(255, 255, 255), font=font_large)
    
    draw.rectangle([(10, img.height - 30), (450, img.height - 10)], fill=(0, 0, 0, 180))
    draw.text((15, img.height - 25), text_bottom, fill=(200, 200, 200), font=font_small)

    return np.array(img)

def generate_timelapse(lon, lat, buffer_km, start_year, end_year, sensor_type, output_gif):
    try:
        point = ee.Geometry.Point([lon, lat])
        roi = point.buffer(buffer_km * 1000).bounds()
        
        start_date = datetime(int(start_year), 1, 1)
        end_date = datetime(int(end_year), 12, 31)
        
        frames = []
        current_date = start_date
        
        print(f"Mengumpulkan frame dari {start_year} hingga {end_year} menggunakan {sensor_type}...")
        
        # Iterasi per bulan
        while current_date < end_date:
            next_date = current_date + relativedelta(months=1)
            date_str_start = current_date.strftime('%Y-%m-%d')
            date_str_end = next_date.strftime('%Y-%m-%d')
            label = current_date.strftime('%B %Y')
            
            print(f"  Memproses {label}...")
            
            if sensor_type == "radar_s1":
                img = get_s1_radar_composite(roi, date_str_start, date_str_end)
            else:
                img = get_s2_cloudless_composite(roi, date_str_start, date_str_end)

            try:
                url = img.getThumbURL({
                    'dimensions': 800, # Resolusi 800px untuk GIF HD
                    'region': roi,
                    'format': 'png'
                })
                
                # Download and watermark
                r = requests.get(url)
                if r.status_code == 200:
                    watermark_txt = f"{'Sentinel-1 SAR' if sensor_type == 'radar_s1' else 'Sentinel-2 Cloudless'} | ZeroClaw AI"
                    frame_array = add_watermark(r.content, label, watermark_txt)
                    frames.append(frame_array)
            except Exception as e:
                print(f"    Gagal memproses {label}: {e}")
                
            current_date = next_date

        if not frames:
            return "ERROR: Gagal mengambil data satelit. Pastikan area tercover."

        # Compile GIF
        print(f"Menggabungkan {len(frames)} frames menjadi {output_gif}...")
        imageio.mimsave(output_gif, frames, format='GIF', fps=2, loop=0)
        
        size_mb = os.path.getsize(output_gif) / (1024*1024)
        return f"SUCCESS: 4D Timelapse disimpan di {output_gif} ({len(frames)} frames, {size_mb:.1f} MB)"

    except Exception as e:
        return f"ERROR: {str(e)}"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--lon", type=float, required=True)
    parser.add_argument("--lat", type=float, required=True)
    parser.add_argument("--buffer_km", type=float, default=5.0)
    parser.add_argument("--start_year", type=int, default=2023)
    parser.add_argument("--end_year", type=int, default=2024)
    parser.add_argument("--sensor", choices=["optik_s2", "radar_s1"], default="optik_s2")
    parser.add_argument("--output", required=True)
    
    args = parser.parse_args()
    print(generate_timelapse(args.lon, args.lat, args.buffer_km, args.start_year, args.end_year, args.sensor, args.output))
