#!/usr/bin/env python3
"""Satellite Info Query Engine via Google Earth Engine
Fetches real data for MODIS, VIIRS, SRTM, ERA5, Dynamic World, GRACE
"""
import sys
import math

def query_modis(lat, lon):
    import ee
    ee.Initialize()
    pt = ee.Geometry.Point([lon, lat])
    
    # MOD13Q1: NDVI
    modis_ndvi = ee.ImageCollection('MODIS/061/MOD13Q1') \
        .filterBounds(pt).sort('system:time_start', False).first()
    
    # MOD11A2: LST
    modis_lst = ee.ImageCollection('MODIS/061/MOD11A2') \
        .filterBounds(pt).sort('system:time_start', False).first()
    
    try:
        n_val = modis_ndvi.reduceRegion(ee.Reducer.first(), pt, 250).getInfo()
        t_val = modis_lst.reduceRegion(ee.Reducer.first(), pt, 1000).getInfo()
        
        ndvi = n_val.get('NDVI', 0) * 0.0001 if n_val else "N/A"
        lst_k = t_val.get('LST_Day_1km', 0) * 0.02 if t_val else "N/A"
        lst_c = lst_k - 273.15 if isinstance(lst_k, float) else "N/A"
        
        date_n = ee.Date(modis_ndvi.get('system:time_start')).format('YYYY-MM-DD').getInfo()
        date_t = ee.Date(modis_lst.get('system:time_start')).format('YYYY-MM-DD').getInfo()
        
        print(f"SUCCESS: MODIS Query at {lat}, {lon}")
        print(f"Dataset: MOD13Q1 (250m) | Date: {date_n} | NDVI: {ndvi:.4f}" if isinstance(ndvi, float) else f"NDVI: {ndvi}")
        print(f"Dataset: MOD11A2 (1km) | Date: {date_t} | LST Day: {lst_c:.1f}°C" if isinstance(lst_c, float) else f"LST: {lst_c}")
    except Exception as e:
        print(f"ERROR: {e}")

def query_viirs(lat, lon):
    import ee
    ee.Initialize()
    pt = ee.Geometry.Point([lon, lat])
    
    viirs = ee.ImageCollection('NOAA/VIIRS/DNB/MONTHLY_V1/VCMSLCFG') \
        .filterBounds(pt).sort('system:time_start', False).first()
    
    try:
        val = viirs.reduceRegion(ee.Reducer.first(), pt, 463).getInfo()
        rad = val.get('avg_rad', 0)
        date = ee.Date(viirs.get('system:time_start')).format('YYYY-MM').getInfo()
        
        cls = "Urban/Bright" if rad > 10 else "Suburban" if rad > 2 else "Rural/Dark"
        
        print(f"SUCCESS: VIIRS Nighttime Lights at {lat}, {lon}")
        print(f"Dataset: NOAA/VIIRS/DNB/MONTHLY_V1/VCMSLCFG (463m)")
        print(f"Month: {date} | Radiance: {rad:.2f} nanoWatts/cm2/sr")
        print(f"Classification: {cls}")
    except Exception as e:
        print(f"ERROR: {e}")

def query_srtm(lat, lon):
    import ee
    ee.Initialize()
    pt = ee.Geometry.Point([lon, lat])
    
    srtm = ee.Image('USGS/SRTMGL1_003')
    terrain = ee.Terrain.products(srtm)
    
    try:
        val = terrain.reduceRegion(ee.Reducer.first(), pt, 30).getInfo()
        elev = val.get('elevation', 'N/A')
        slope = val.get('slope', 'N/A')
        aspect = val.get('aspect', 'N/A')
        
        print(f"SUCCESS: SRTM Terrain at {lat}, {lon}")
        print(f"Dataset: USGS/SRTMGL1_003 (30m)")
        print(f"Elevation: {elev} m dpl")
        print(f"Slope: {slope:.1f}°" if isinstance(slope, float) else f"Slope: {slope}")
        print(f"Aspect: {aspect:.1f}°" if isinstance(aspect, float) else f"Aspect: {aspect}")
    except Exception as e:
        print(f"ERROR: {e}")

def query_era5(lat, lon):
    import ee
    ee.Initialize()
    pt = ee.Geometry.Point([lon, lat])
    
    era5 = ee.ImageCollection('ECMWF/ERA5_LAND/MONTHLY_AGGR') \
        .filterBounds(pt).sort('system:time_start', False).first()
    
    try:
        val = era5.reduceRegion(ee.Reducer.first(), pt, 11132).getInfo()
        temp_k = val.get('temperature_2m', 0)
        temp_c = temp_k - 273.15 if temp_k else "N/A"
        precip = val.get('total_precipitation_sum', 0)
        precip_mm = precip * 1000 if precip else "N/A"
        
        date = ee.Date(era5.get('system:time_start')).format('YYYY-MM').getInfo()
        
        print(f"SUCCESS: ERA5 Climate at {lat}, {lon}")
        print(f"Dataset: ECMWF/ERA5_LAND/MONTHLY_AGGR (~11km)")
        print(f"Month: {date}")
        print(f"Temperature (2m): {temp_c:.1f}°C" if isinstance(temp_c, float) else f"Temperature: {temp_c}")
        print(f"Total Precipitation: {precip_mm:.1f} mm" if isinstance(precip_mm, float) else f"Precipitation: {precip_mm}")
    except Exception as e:
        print(f"ERROR: {e}")

def query_dw(lat, lon):
    import ee
    ee.Initialize()
    pt = ee.Geometry.Point([lon, lat])
    
    dw = ee.ImageCollection('GOOGLE/DYNAMICWORLD/V1') \
        .filterBounds(pt).sort('system:time_start', False).first()
    
    classes = ['water', 'trees', 'grass', 'flooded_vegetation', 'crops', 'shrub_and_scrub', 'built', 'bare', 'snow_and_ice']
    
    try:
        val = dw.reduceRegion(ee.Reducer.first(), pt, 10).getInfo()
        label_idx = val.get('label', -1)
        label_name = classes[label_idx] if 0 <= label_idx < len(classes) else "Unknown"
        
        date = ee.Date(dw.get('system:time_start')).format('YYYY-MM-DD').getInfo()
        
        print(f"SUCCESS: Dynamic World Land Cover at {lat}, {lon}")
        print(f"Dataset: GOOGLE/DYNAMICWORLD/V1 (10m)")
        print(f"Date: {date}")
        print(f"Primary Class: {label_name.upper()} (Index: {label_idx})")
        
        # Probabilities
        print("Probabilities:")
        for c in classes:
            prob = val.get(c, 0)
            if prob > 0.05:
                print(f"  {c}: {prob*100:.1f}%")
    except Exception as e:
        print(f"ERROR: {e}")

def query_grace(lat, lon):
    import ee
    ee.Initialize()
    pt = ee.Geometry.Point([lon, lat])
    
    grace = ee.ImageCollection('NASA/GRACE/MASS_GRIDS/LAND') \
        .filterBounds(pt).sort('system:time_start', False).first()
    
    try:
        val = grace.reduceRegion(ee.Reducer.first(), pt, 111320).getInfo()
        lwe = val.get('lwe_thickness', 'N/A')
        date = ee.Date(grace.get('system:time_start')).format('YYYY-MM').getInfo()
        
        print(f"SUCCESS: GRACE Water Storage at {lat}, {lon}")
        print(f"Dataset: NASA/GRACE/MASS_GRIDS/LAND (~111km)")
        print(f"Month: {date}")
        print(f"Equivalent Water Thickness (Anomaly): {lwe:.2f} cm" if isinstance(lwe, float) else f"Anomaly: {lwe}")
        if isinstance(lwe, float):
            print(f"Status: {'Surplus' if lwe > 0 else 'Deficit/Drought'}")
    except Exception as e:
        print(f"ERROR: {e}")

if __name__ == '__main__':
    if len(sys.argv) < 4:
        sys.exit(1)
    mode = sys.argv[1]
    lat = float(sys.argv[2])
    lon = float(sys.argv[3])
    
    if mode == 'modis': query_modis(lat, lon)
    elif mode == 'viirs': query_viirs(lat, lon)
    elif mode == 'srtm': query_srtm(lat, lon)
    elif mode == 'era5': query_era5(lat, lon)
    elif mode == 'dynamic_world': query_dw(lat, lon)
    elif mode == 'grace': query_grace(lat, lon)
