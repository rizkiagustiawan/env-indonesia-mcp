import sys
import os
import time
import json
import logging
from concurrent.futures import ThreadPoolExecutor, as_completed

# Tambah paths
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, BASE_DIR)
sys.path.insert(0, os.path.join(BASE_DIR, 'datasources'))
sys.path.insert(0, os.path.join(BASE_DIR, 'satellite'))
sys.path.insert(0, os.path.join(BASE_DIR, 'gis'))

logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')
OUTPUT_DIR = os.path.join(os.path.dirname(BASE_DIR), "../output_amdal")
os.makedirs(OUTPUT_DIR, exist_ok=True)

# Helper function: Integrasi Cartography
import matplotlib
import matplotlib.pyplot as plt

def render_amdal_map(geojson_str, raster_path, title, out_png, cmap_style='viridis'):
    try:
        import matplotlib
        import matplotlib.pyplot as plt
        import rasterio
        import numpy as np
        matplotlib.use('Agg')
        logging.info(f"Mencetak Peta SNI: {title} ...")
        
        with rasterio.open(raster_path) as src:
            img = src.read(1).astype(np.float32)
            img[img == 0] = np.nan
            img[img < -999] = np.nan
            if not np.all(np.isnan(img)):
                vmin, vmax = np.nanpercentile(img, [2, 98])
                if np.isnan(vmin) or np.isnan(vmax) or vmin == vmax:
                    vmin, vmax = np.nanmin(img), np.nanmax(img)
                
                fig, ax = plt.subplots(figsize=(12, 10))
                plt.imshow(img, cmap=cmap_style, vmin=vmin, vmax=vmax)
                plt.colorbar(label='Index / Value')
                plt.title(f"AMDAL: {title}
Kawasan Studi (BBox) - Skala 1:50.000", fontsize=16, fontweight='bold')
                plt.axis('off')
                plt.figtext(0.9, 0.05, 'Digenerate otomatis oleh AI Agent 2026', horizontalalignment='right')
                plt.savefig(out_png, bbox_inches='tight', dpi=300)
                plt.close()
                logging.info(f"Berhasil: {out_png}")
            else:
                logging.warning(f"Gambar kosong/NaN semua: {raster_path}")
    except Exception as e:
        logging.error(f"Gagal mencetak peta {title}: {e}")

# Layer Tasks
def task_admin(lat, lon, buffer_km):
    try:
        from datasources.big_geoportal import query_admin_kabkota
        res = query_admin_kabkota(lat, lon, buffer_km)
        # Ambil BBOX dari output jika bisa, atau buat dummy geojson dari buffer
        geojson_str = json.dumps({
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "Point", "coordinates": [lon, lat]}, "properties": {}}]
        })
        return geojson_str
    except Exception as e:
        logging.error(f"L1 Error: {e}")
        return None

def task_terrain(lat, lon, buffer_km):
    try:
        from gis.raster_engine import dem_analysis_gee
        out_tif = f"/tmp/l2_terrain_{lat}_{lon}.tif"
        dem_analysis_gee(lat, lon, buffer_km, 'slope', out_tif)
        return out_tif
    except Exception as e:
        logging.error(f"L2 Error: {e}")
        return None

def task_lulc(lat, lon, buffer_km, start, end):
    try:
        from gis.landcover_engine import ccdc_change_detection
        out_tif = f"/tmp/l3_ccdc_{lat}_{lon}.tif"
        ccdc_change_detection(lat, lon, buffer_km, start, end, out_tif)
        return out_tif
    except Exception as e:
        logging.error(f"L3 Error: {e}")
        return None

def task_subsidence(lat, lon, buffer_km, start, end):
    try:
        from satellite.sar_engine import subsidence_screening
        out_tif = f"/tmp/l6_subs_{lat}_{lon}.tif"
        subsidence_screening(lat, lon, buffer_km, start, end, out_tif)
        return out_tif
    except Exception as e:
        logging.error(f"L6 Error: {e}")
        return None

def run_amdal_factory(lat, lon, buffer_km, start_date, end_date):
    logging.info(f"=== FULL AMDAL FACTORY STARTING ===")
    
    # Dapatkan batas wilayah (L1) untuk SNI basemap
    geojson_admin = task_admin(lat, lon, buffer_km)
    if not geojson_admin:
        geojson_admin = json.dumps({"type": "FeatureCollection", "features": []})

    results = {}
    results['terrain'] = task_terrain(lat, lon, buffer_km)
    results['lulc'] = task_lulc(lat, lon, buffer_km, start_date, end_date)
    results['subsidence'] = task_subsidence(lat, lon, buffer_km, start_date, end_date)

    # RENDER PETA JADI
    logging.info("\n--- RENDER PETA AMDAL SNI (PRODUK JADI) ---")
    if results['terrain']:
        render_amdal_map(geojson_admin, results['terrain'], "Peta Kemiringan Lereng & Rawan Longsor", os.path.join(OUTPUT_DIR, "amdal_01_topografi_slope.png"), 'terrain')
    if results['lulc']:
        render_amdal_map(geojson_admin, results['lulc'], "Peta Deteksi Perubahan Tutupan Lahan (CCDC)", os.path.join(OUTPUT_DIR, "amdal_02_perubahan_lahan.png"), 'RdYlGn_r')
    if results['subsidence']:
        render_amdal_map(geojson_admin, results['subsidence'], "Peta Risiko Penurunan Muka Tanah (InSAR Subsidence)", os.path.join(OUTPUT_DIR, "amdal_03_risiko_subsiden.png"), 'magma')
    
    logging.info(f"=== AMDAL FACTORY SELESAI. Silakan cek folder: {OUTPUT_DIR} ===")

if __name__ == "__main__":
    run_amdal_factory(-1.2, 116.5, 10.0, "2025-01-01", "2026-01-01")

