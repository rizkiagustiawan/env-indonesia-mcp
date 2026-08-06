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
    """Deep climate analysis: multi-year ERA5 + water balance + debit banjir rencana.
    
    Metode: Thornthwaite water balance + Rational Method untuk debit banjir rencana.
    Ref: Thornthwaite (1948), Chow et al. (1988) Applied Hydrology.
    """
    import ee
    from datetime import datetime, timedelta
    ee.Initialize()
    pt = ee.Geometry.Point([lon, lat])
    
    # Multi-year (5 tahun terakhir) untuk statistik iklim robust
    end_date = datetime.now().strftime("%Y-%m-%d")
    start_date = (datetime.now() - timedelta(days=365*5)).strftime("%Y-%m-%d")
    
    era5 = ee.ImageCollection('ECMWF/ERA5_LAND/MONTHLY_AGGR') \
        .filterBounds(pt).filterDate(start_date, end_date)
    
    try:
        count = era5.size().getInfo()
        if count == 0:
            print("ERROR: Tidak ada data ERA5 untuk periode ini.")
            return
        
        # Multi-year statistics
        mean_temp_k = era5.select('temperature_2m').mean().reduceRegion(ee.Reducer.mean(), pt, 11132).getInfo()
        max_temp_k = era5.select('temperature_2m').max().reduceRegion(ee.Reducer.max(), pt, 11132).getInfo()
        min_temp_k = era5.select('temperature_2m').min().reduceRegion(ee.Reducer.min(), pt, 11132).getInfo()
        
        total_precip = era5.select('total_precipitation_sum').sum().reduceRegion(ee.Reducer.sum(), pt, 11132).getInfo()
        mean_monthly_precip = era5.select('total_precipitation_sum').mean().reduceRegion(ee.Reducer.mean(), pt, 11132).getInfo()
        max_monthly_precip = era5.select('total_precipitation_sum').max().reduceRegion(ee.Reducer.max(), pt, 11132).getInfo()
        
        # Extract values
        temp_mean_c = mean_temp_k.get('temperature_2m', 300) - 273.15
        temp_max_c = max_temp_k.get('temperature_2m', 310) - 273.15
        temp_min_c = min_temp_k.get('temperature_2m', 290) - 273.15
        precip_total_mm = total_precip.get('total_precipitation_sum', 0) * 1000  # m → mm
        precip_mean_monthly = mean_monthly_precip.get('total_precipitation_sum', 0) * 1000
        precip_max_monthly = max_monthly_precip.get('total_precipitation_sum', 0) * 1000
        precip_annual = precip_total_mm / 5  # rata-rata tahunan
        
        # Thornthwaite PET (Potential Evapotranspiration)
        # T > 0 and T < 26.5: PET = 16 * (10*T/I)^a
        # I = sum of (T/5)^1.514 for 12 months (approx with annual mean)
        I = (temp_mean_c / 5) ** 1.514 * 12
        a = (0.675 * I**3 - 77.1 * I**2 + 17920 * I + 491390) / 1000000 if I > 0 else 0.5
        pet_monthly = 16 * (10 * temp_mean_c / max(I, 0.01)) ** a if temp_mean_c > 0 else 0
        pet_annual = pet_monthly * 12
        
        # Water balance: P - PET
        water_balance = precip_annual - pet_annual
        
        # Debit banjir rencana (Rational Method: Q = 0.278 * C * I * A)
        # C = runoff coefficient (0.3 for vegetated, 0.6 for mixed, 0.9 for urban)
        # I = rainfall intensity (max monthly / 30 days → mm/hr approx)
        # A = catchment area (buffer_km^2 * pi)
        C_runoff = 0.5  # mixed land cover
        I_intensity = (precip_max_monthly / 30 / 24) if precip_max_monthly > 0 else 5  # mm/hr
        A_km2 = 3.14159 * (10 ** 2)  # 10km buffer
        Q_banjir = 0.278 * C_runoff * I_intensity * A_km2  # m³/s
        
        # Get latest month for reference
        latest = era5.sort('system:time_start', False).first()
        latest_date = ee.Date(latest.get('system:time_start')).format('YYYY-MM').getInfo()
        
        print(f"SUCCESS: ERA5 Deep Climate Analysis at {lat}, {lon}")
        print(f"Dataset: ECMWF/ERA5_LAND/MONTHLY_AGGR (~11km resolution)")
        print(f"Periode: {start_date} to {end_date} ({count} monthly images, 5-year)")
        print(f"Latest month: {latest_date}")
        print(f"\n=== Statistik Suhu (5 tahun) ===")
        print(f"  Mean : {temp_mean_c:.1f}°C")
        print(f"  Max  : {temp_max_c:.1f}°C")
        print(f"  Min  : {temp_min_c:.1f}°C")
        print(f"  Range: {temp_max_c - temp_min_c:.1f}°C (amplitudo termal)")
        print(f"\n=== Statistik Curah Hujan (5 tahun) ===")
        print(f"  Total 5 tahun : {precip_total_mm:.0f} mm")
        print(f"  Rata-rata tahunan: {precip_annual:.0f} mm/thn")
        print(f"  Rata-rata bulanan: {precip_mean_monthly:.1f} mm/bln")
        print(f"  Maks bulanan    : {precip_max_monthly:.1f} mm/bln")
        print(f"\n=== Water Balance (Thornthwaite) ===")
        print(f"  PET tahunan     : {pet_annual:.0f} mm/thn")
        print(f"  Water balance   : {water_balance:.0f} mm/thn (P - PET)")
        if water_balance > 0:
            print(f"  Status: SURPLUS AIR (humid climate) — waspada banjir")
        else:
            print(f"  Status: DEFISIT AIR (dry climate) — waspda kekeringan")
        print(f"\n=== Debit Banjir Rencana (Rational Method) ===")
        print(f"  Koefisien runoff C: {C_runoff} (mixed land cover)")
        print(f"  Intensitas hujan I: {I_intensity:.1f} mm/jam")
        print(f"  Area catchment A : {A_km2:.1f} km²")
        print(f"  Q banjir rencana : {Q_banjir:.1f} m³/s (Q = 0.278*C*I*A)")
        print(f"\nMetode: Thornthwaite (1948) PET | Rational Method (Chow 1988)")
        
    except Exception as e:
        import traceback
        print(f"ERROR: {e}")
        print(traceback.format_exc()[:200])

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

def query_frp(lat, lon, start_date='2024-01-01', end_date='2024-12-31'):
    """Fire Radiative Power (FRP) time series from VIIRS SNPP/NOAA-20.
    375m resolution, MW units. Monitors fire intensity over time.
    Ref: VIIRS Active Fire product (Schroeder et al. 2014)
    """
    import ee, json, datetime
    ee.Initialize()

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(25000)  # 25km buffer

    # VIIRS SNPP active fire
    viirs_snpp = ee.ImageCollection('NASA/LANCE/SNPP_VIIRS/C2') \
        .filterDate(start_date, end_date) \
        .filterBounds(roi) \
        .select(['frp', 'Bright_ti4', 'Bright_ti5', 'confidence'])

    # VIIRS NOAA-20
    viirs_n20 = ee.ImageCollection('NASA/LANCE/NOAA20_VIIRS/C2') \
        .filterDate(start_date, end_date) \
        .filterBounds(roi) \
        .select(['frp', 'Bright_ti4', 'Bright_ti5', 'confidence'])

    # Merge both sensors
    viirs_all = viirs_snpp.merge(viirs_n20)

    count = viirs_all.size().getInfo()

    if count == 0:
        return json.dumps({
            "status": "NO_FIRE",
            "message": f"Tidak ada hotspot terdeteksi di radius 25km dari ({lat}, {lon}) periode {start_date} - {end_date}",
            "sensor": "VIIRS SNPP + NOAA-20",
            "resolution": "375m"
        }, indent=2)

    # Aggregate FRP statistics
    frp_stats = viirs_all.select('frp').reduce(
        ee.Reducer.mean().combine(ee.Reducer.max(), '', True)
            .combine(ee.Reducer.sum(), '', True)
            .combine(ee.Reducer.count(), '', True)
    ).reduceRegion(
        reducer=ee.Reducer.first(),
        geometry=roi, scale=375
    ).getInfo()

    # Monthly FRP time series
    months = []
    start = datetime.datetime.strptime(start_date, '%Y-%m-%d')
    end = datetime.datetime.strptime(end_date, '%Y-%m-%d')
    current = start
    while current < end:
        month_end = (current.replace(day=28) + datetime.timedelta(days=4)).replace(day=1)
        if month_end > end:
            month_end = end
        monthly = viirs_all.filterDate(current.strftime('%Y-%m-%d'), month_end.strftime('%Y-%m-%d')) \
            .filterBounds(roi)
        monthly_count = monthly.size().getInfo()
        if monthly_count > 0:
            monthly_frp_list = monthly.aggregate_array('frp').getInfo()
            # Filter None values
            valid_frp = [v for v in (monthly_frp_list or []) if v is not None]
            total_frp = sum(valid_frp) if valid_frp else 0
            hotspot_count = len(valid_frp)
        else:
            total_frp = 0
            hotspot_count = 0
        months.append({
            'month': current.strftime('%Y-%m'),
            'total_frp_mw': total_frp,
            'hotspot_count': hotspot_count
        })
        current = month_end

    result = {
        "status": "FIRE_DETECTED",
        "koordinat": f"{lat}, {lon}",
        "periode": f"{start_date} - {end_date}",
        "sensor": "VIIRS SNPP + NOAA-20 (375m)",
        "total_deteksi": count,
        "frp_mean_mw": frp_stats.get('frp_mean', 0),
        "frp_max_mw": frp_stats.get('frp_max', 0),
        "frp_total_mw": frp_stats.get('frp_sum', 0),
        "monthly_timeseries": months,
        "interpretasi": "FRP tinggi (>100 MW) = kebakaran besar/intens. FRP rendah (<10 MW) = titik panas kecil/smouldering.",
        "ref": "Schroeder et al. 2014, VIIRS 375m Active Fire"
    }

    return json.dumps(result, indent=2)


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
    elif mode == 'frp':
        print(query_frp(lat, lon, sys.argv[4], sys.argv[5]))
