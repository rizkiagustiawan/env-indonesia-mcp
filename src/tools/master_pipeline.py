import sys
import os
import time
import json
import logging
from concurrent.futures import ThreadPoolExecutor, as_completed

# Tambah path agar bisa import dari subfolder
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), 'datasources'))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), 'satellite'))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), 'gis'))

logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')

# --- Dummy imports or wrappers for existing modules ---
# (Kita gunakan try-except agar tidak crash jika modul spesifik gagal load dependencies)

def layer1_admin(lat, lon, buffer_km):
    logging.info("Executing Layer 1: Admin & Legal (BIG)...")
    try:
        from datasources.big_geoportal import query_admin_kabkota
        return query_admin_kabkota(lat, lon, buffer_km)
    except Exception as e:
        return f"Layer 1 Error: {e}"

def layer2_terrain(lat, lon, buffer_km):
    logging.info("Executing Layer 2: Terrain & Morphology...")
    try:
        from gis.raster_engine import dem_analysis_gee
        output_path = f"/tmp/layer2_dem_{lat}_{lon}.tif"
        return dem_analysis_gee(lat, lon, buffer_km, 'slope', f'/tmp/layer2_dem_{lat}_{lon}.tif')
    except Exception as e:
        return f"Layer 2 Error: {e}"

def layer3_lulc(lat, lon, buffer_km, start_date, end_date):
    logging.info("Executing Layer 3: Land Cover Change (Optic+SAR)...")
    try:
        from gis.landcover_engine import ccdc_change_detection
        output_path = f"/tmp/layer3_ccdc_{lat}_{lon}.tif"
        return ccdc_change_detection(lat, lon, buffer_km, start_date, end_date, output_path)
    except Exception as e:
        return f"Layer 3 Error: {e}"

def layer4_ecological(lat, lon, buffer_km, start_date, end_date):
    logging.info("Executing Layer 4: Ecological Index (NDVI/NDWI)...")
    try:
        from gis.raster_engine import ndvi_timeseries
        output_path = f"/tmp/layer4_ndvi_{lat}_{lon}.tif"
        return ndvi_timeseries(lat, lon, buffer_km, int(start_date[:4]), int(end_date[:4]), f'/tmp/layer4_ndvi_{lat}_{lon}.tif')
    except Exception as e:
        return f"Layer 4 Error: {e}"

def layer5_anomalies(lat, lon, buffer_km):
    logging.info("Executing Layer 5: Anomalies (Methane & Hotspots)...")
    try:
        from satellite.methane_engine import query_methane
        from satellite.hotspot_scanner import scan_indonesia_hotspots
        res = {"methane": query_methane(lat, lon, buffer_km), "hotspots": scan_indonesia_hotspots()}
        return json.dumps(res)
    except Exception as e:
        return f"Layer 5 Error: {e}"

def layer6_risks(lat, lon, buffer_km, start_date, end_date):
    logging.info("Executing Layer 6: Risks & Subsidence (InSAR/BNPB)...")
    try:
        from satellite.sar_engine import subsidence_screening
        output_path = f"/tmp/layer6_subsidence_{lat}_{lon}.tif"
        return subsidence_screening(lat, lon, buffer_km, start_date, end_date, output_path)
    except Exception as e:
        return f"Layer 6 Error: {e}"

def layer7_suitability(lat, lon, buffer_km):
    logging.info("Executing Layer 7: MCDA Suitability Analysis...")
    try:
        from gis.spatial_engine import suitability_analysis
        output_path = f"/tmp/layer7_suitability_{lat}_{lon}.tif"
        criteria = {"slope_weight": 0.4, "ndvi_weight": 0.3, "water_distance_weight": 0.3}
        return suitability_analysis(lat, lon, buffer_km, criteria, f'/tmp/layer7_suitability_{lat}_{lon}.tif')
    except Exception as e:
        return f"Layer 7 Error: {e}"

def layer8_synthesis(results):
    logging.info("Executing Layer 8: Synthesis & Recommendation...")
    # Dalam implementasi riil, ini diproses oleh LLM/Logika aturan
    report = "=== SUPREME 8-LAYER ENVIRONMENTAL IMPACT ASSESSMENT ===\n"
    for k, v in results.items():
        # Truncate output panjang
        v_str = str(v)
        v_trunc = v_str[:150] + "..." if len(v_str) > 150 else v_str
        report += f"[{k}] Status: {v_trunc}\n"
    
    report += "\n>> REKOMENDASI INTERVENSI:\n"
    report += "- Analisis deteksi 8 dimensi selesai. Menunggu validasi human-in-the-loop.\n"
    return report

def run_supreme_pipeline(lat, lon, buffer_km, start_date, end_date):
    logging.info(f"STARTING SUPREME PIPELINE for Lat: {lat}, Lon: {lon}, Buffer: {buffer_km}km")
    start_time = time.time()
    
    results = {}
    
    # Eksekusi paralel untuk layer yang independen
    with ThreadPoolExecutor(max_workers=4) as executor:
        future_to_layer = {
            executor.submit(layer1_admin, lat, lon, buffer_km): "L1_Admin",
            executor.submit(layer2_terrain, lat, lon, buffer_km): "L2_Terrain",
            executor.submit(layer3_lulc, lat, lon, buffer_km, start_date, end_date): "L3_LULC",
            executor.submit(layer4_ecological, lat, lon, buffer_km, start_date, end_date): "L4_Ecological",
            executor.submit(layer5_anomalies, lat, lon, buffer_km): "L5_Anomalies",
            executor.submit(layer6_risks, lat, lon, buffer_km, start_date, end_date): "L6_Risks",
            executor.submit(layer7_suitability, lat, lon, buffer_km): "L7_Suitability"
        }
        
        for future in as_completed(future_to_layer):
            layer_name = future_to_layer[future]
            try:
                data = future.result()
                results[layer_name] = data
            except Exception as exc:
                results[layer_name] = f"Failed: {exc}"
                
    # Eksekusi sekuensial untuk sintesis
    final_report = layer8_synthesis(results)
    
    elapsed = time.time() - start_time
    logging.info(f"PIPELINE COMPLETED in {elapsed:.2f} seconds.")
    print("\n" + final_report)

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Supreme 8-Layer Pipeline")
    parser.add_argument("--lat", type=float, default=-1.2, help="Latitude")
    parser.add_argument("--lon", type=float, default=116.5, help="Longitude")
    parser.add_argument("--buffer", type=float, default=10.0, help="Buffer (km)")
    parser.add_argument("--start", type=str, default="2025-01-01", help="Start Date")
    parser.add_argument("--end", type=str, default="2026-01-01", help="End Date")
    
    args = parser.parse_args()
    run_supreme_pipeline(args.lat, args.lon, args.buffer, args.start, args.end)
