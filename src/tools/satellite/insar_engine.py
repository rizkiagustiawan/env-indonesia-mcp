#!/usr/bin/env python3
"""InSAR Engine — Ground Deformation Monitoring
Mode 1: GEE-based coherence screening (no credentials needed)
Mode 2: ASF HyP3 API for pre-processed InSAR products (needs Earthdata login)
Ref: Ferretti et al. 2001 (PS-InSAR), Berardino et al. 2002 (SBAS)
"""
import sys, os, json, math
import numpy as np

try:
    import ee
    ee.Initialize()
except:
    pass

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap
import requests


def make_roi(lat, lon, buffer_km):
    point = ee.Geometry.Point([lon, lat])
    return point.buffer(buffer_km * 1000).bounds()


def coherence_screening(lat, lon, buffer_km, start_date, end_date, output_path):
    """InSAR coherence screening using Sentinel-1 GRD temporal statistics.
    NOT actual InSAR — uses temporal coefficient of variation as proxy for
    surface stability. Low coherence = potential deformation/change.
    
    For actual displacement measurements, use hyp3_insar() mode.
    """
    roi = make_roi(lat, lon, buffer_km)
    
    # S1 GRD IW VV collection
    s1 = ee.ImageCollection('COPERNICUS/S1_GRD') \
        .filterDate(start_date, end_date) \
        .filterBounds(roi) \
        .filter(ee.Filter.eq('instrumentMode', 'IW')) \
        .filter(ee.Filter.listContains('transmitterReceiverPolarisation', 'VV')) \
        .select('VV')
    
    count = s1.size().getInfo()
    if count < 10:
        print(f"WARNING: Hanya {count} scene. Minimal 10 scene untuk hasil reliable.")
    
    # Temporal statistics
    mean = s1.mean().clip(roi)
    std = s1.reduce(ee.Reducer.stdDev()).clip(roi)
    cv = std.divide(mean.abs()).rename('coherence_proxy')
    
    # Amplitude Dispersion Index (ADI) — PS-InSAR proxy
    # ADI < 0.25 = likely persistent scatterer (Ferretti et al. 2001)
    adi = std.divide(mean.abs()).rename('ADI')
    ps_candidates = adi.lt(0.25).rename('PS_candidates')
    
    # Temporal mean backscatter
    mean_vv = mean.rename('mean_VV_dB')
    
    # SRTM for context
    srtm = ee.Image('USGS/SRTMGL1_003').select('elevation').clip(roi)
    
    result = cv.addBands([adi, ps_candidates.toFloat(), mean_vv, srtm])
    
    # Stats
    stats = cv.reduceRegion(
        reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True)
            .combine(ee.Reducer.max(), '', True),
        geometry=roi, scale=10, maxPixels=1e9
    ).getInfo()
    
    ps_stats = ps_candidates.reduceRegion(
        reducer=ee.Reducer.sum().combine(ee.Reducer.count(), '', True),
        geometry=roi, scale=10, maxPixels=1e9
    ).getInfo()
    
    # Visualization
    vis = {
        'min': 0, 'max': 0.5,
        'palette': ['#313695', '#4575b4', '#74add1', '#abd9e9', '#fee090',
                    '#fdae61', '#f46d43', '#d73027', '#a50026'],
        'region': roi, 'dimensions': 800
    }
    thumb = cv.getThumbURL(vis)
    img_data = requests.get(thumb, timeout=60).content
    with open(output_path, 'wb') as f:
        f.write(img_data)
    
    # GeoTIFF
    geotiff_path = output_path.replace('.png', '.tif')
    try:
        url = result.getDownloadURL({
            'region': roi, 'scale': 10, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
        })
        tif_data = requests.get(url, timeout=120).content
        with open(geotiff_path, 'wb') as f:
            f.write(tif_data)
        print(f"GeoTIFF: {geotiff_path} ({len(tif_data)/1024:.1f} KB)")
    except Exception as e:
        print(f"GeoTIFF export failed: {e}")
    
    ps_sum = ps_stats.get('PS_candidates_sum', 0) or 0
    ps_count = ps_stats.get('PS_candidates_count', 1) or 1
    
    # Provenance metadata
    try:
        sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'gis'))
        from provenance import create_provenance
        create_provenance(output_path,
            tool='insar_screening', gee_collection='COPERNICUS/S1_GRD',
            date_range=[start_date, end_date],
            coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
            algorithms=['Temporal CV', 'Amplitude Dispersion Index (ADI)', 'PS candidate detection'],
            references=['Ferretti et al. 2001'],
            crs='EPSG:4326', scale_m=10)
    except:
        pass  # provenance is non-critical
    
    print(f"SUCCESS: InSAR Coherence Screening. Output: {output_path}")
    print(f"Periode: {start_date} - {end_date}")
    print(f"Jumlah scene S1: {count}")
    print(f"Coherence proxy (CV):")
    print(f"  Mean: {stats.get('coherence_proxy_mean', 'N/A')}")
    print(f"  Min: {stats.get('coherence_proxy_min', 'N/A')}")
    print(f"  Max: {stats.get('coherence_proxy_max', 'N/A')}")
    print(f"PS Candidates (ADI < 0.25): {ps_sum:.0f} piksel ({100*ps_sum/ps_count:.1f}%)")
    print(f"Biru=stabil (low CV), Merah=tidak stabil (high CV)")
    print(f"")
    print(f"CATATAN: Ini screening proxy, BUKAN InSAR displacement.")
    print(f"Untuk displacement aktual (mm/yr), gunakan mode hyp3_insar.")
    print(f"Ref: Ferretti et al. 2001 (PS-InSAR), ADI threshold 0.25")


def hyp3_insar(lat, lon, start_date, end_date, output_path):
    """Submit InSAR job to ASF HyP3 API.
    Returns pre-processed interferogram, coherence, and displacement map.
    Requires Earthdata credentials in ~/.netrc or env vars.
    """
    try:
        from hyp3_sdk import HyP3
    except ImportError:
        print("ERROR: hyp3_sdk belum terinstall. Jalankan: pip install hyp3_sdk")
        print("Juga perlu Earthdata login: https://urs.earthdata.nasa.gov/")
        print("Simpan credentials di ~/.netrc:")
        print("  machine urs.earthdata.nasa.gov")
        print("  login YOUR_USERNAME") 
        print("  password YOUR_PASSWORD")
        return
    
    try:
        import asf_search as asf
    except ImportError:
        print("ERROR: asf_search belum terinstall. Jalankan: pip install asf_search")
        return
    
    # Search for S1 SLC scenes
    print(f"Mencari scene Sentinel-1 SLC di ({lat}, {lon})...")
    results = asf.geo_search(
        intersectsWith=f'POINT({lon} {lat})',
        platform=asf.PLATFORM.SENTINEL1,
        processingLevel=asf.PRODUCT_TYPE.SLC,
        beamMode=asf.BEAMMODE.IW,
        start=start_date,
        end=end_date,
        maxResults=50
    )
    
    if len(results) < 2:
        print(f"ERROR: Hanya {len(results)} scene ditemukan. Minimal 2 untuk InSAR.")
        return
    
    print(f"Ditemukan {len(results)} scene. Memilih pasangan optimal...")
    
    # Sort by date, pick reference and secondary
    results_sorted = sorted(results, key=lambda x: x.properties['startTime'])
    reference = results_sorted[-2]  # second to last
    secondary = results_sorted[-1]  # most recent
    
    ref_name = reference.properties['sceneName']
    sec_name = secondary.properties['sceneName']
    
    print(f"Reference: {ref_name}")
    print(f"Secondary: {sec_name}")
    
    # Submit to HyP3
    try:
        hyp3 = HyP3()
        print("Mengirim job InSAR ke ASF HyP3...")
        # Perbaikan bug SDK v7 (menggunakan object Batch)
        batch = hyp3.submit_insar_job(ref_name, sec_name, 
                                     name=f'insar_{lat}_{lon}',
                                     include_displacement_maps=True,
                                     include_look_vectors=True,
                                     looks='20x4')
        
        job = batch.jobs[0]
        
        print(f"Job submitted: {job.job_id}")
        print(f"Status: {job.status_code}")
        print(f"Estimasi waktu: 15-30 menit")
        print(f"")
        print(f"Untuk cek status: python3 insar_engine.py status {job.job_id}")
        print(f"Output akan berisi:")
        print(f"  - Wrapped interferogram")
        print(f"  - Unwrapped interferogram")
        print(f"  - Coherence map")
        print(f"  - LOS displacement (m)")
        print(f"  - Vertical displacement (m)")
        
        # Optionally wait
        print(f"\nMenunggu hasil (max 30 menit)...")
        job = hyp3.watch(job)
        
        if job.complete():
            print(f"Job selesai! Mengunduh hasil...")
            job.download_files(output_path.replace('.png', ''))
            print(f"SUCCESS: InSAR products downloaded to {output_path.replace('.png', '')}/")
            print(f"Files: interferogram, coherence, displacement maps")
        else:
            print(f"Job belum selesai. Cek manual dengan: python3 insar_engine.py status {job.job_id}")
            
    except Exception as e:
        print(f"ERROR HyP3: {e}")
        print(f"Pastikan credentials Earthdata ada di ~/.netrc")
        print(f"Daftar gratis di: https://urs.earthdata.nasa.gov/")


def check_hyp3_status(job_id):
    """Check status of HyP3 InSAR job."""
    try:
        from hyp3_sdk import HyP3
        hyp3 = HyP3()
        jobs = hyp3.find_jobs(name=job_id)
        for job in jobs:
            print(json.dumps(job.to_dict(), indent=2, default=str))
    except ImportError:
        print("ERROR: hyp3_sdk belum terinstall.")
    except Exception as e:
        print(f"ERROR: {e}")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage:")
        print("  insar_engine.py screening lat lon buffer_km start end output")
        print("  insar_engine.py hyp3 lat lon start end output")
        print("  insar_engine.py status job_id")
        sys.exit(1)
    
    mode = sys.argv[1]
    try:
        if mode == 'screening':
            coherence_screening(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                              sys.argv[5], sys.argv[6], sys.argv[7])
        elif mode == 'hyp3':
            hyp3_insar(float(sys.argv[2]), float(sys.argv[3]),
                      sys.argv[4], sys.argv[5], sys.argv[6])
        elif mode == 'status':
            check_hyp3_status(sys.argv[2])
        else:
            print(f"ERROR: Mode '{mode}' tidak dikenal. Gunakan: screening, hyp3, status")
    except Exception as e:
        print(f"ERROR [E502]: {e}")
        import traceback
        traceback.print_exc()
