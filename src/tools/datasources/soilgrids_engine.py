import sys
import json
import math
import urllib.request
import urllib.parse

def fetch_soilgrids(lat, lon):
    """
    Fetches soil properties from SoilGrids REST API (ISRIC)
    and applies Rosetta-like Pedotransfer Function (PTF) to get hydrological params.
    """
    url = f"https://rest.isric.org/soilgrids/v2.0/properties/query?lat={lat}&lon={lon}"
    # Properties we need for hydrology: sand, silt, clay, bdod (bulk density)
    props = ["sand", "silt", "clay", "bdod"]
    for p in props:
        url += f"&property={p}"
    url += "&depth=0-5cm&depth=5-15cm&depth=15-30cm&value=mean"
    
    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'EnvIndonesiaMCP/1.0'})
        with urllib.request.urlopen(req, timeout=15) as response:
            data = json.loads(response.read().decode())
    except Exception as e:
        return {"error": f"Failed to fetch SoilGrids data: {str(e)}"}
        
    properties = data.get("properties", {}).get("layers", [])
    if not properties:
        return {"error": "No data returned from SoilGrids for this location"}
        
    # Parse layers
    soil_data = {}
    for layer in properties:
        name = layer.get("name")
        depths = layer.get("depths", [])
        if depths:
            # Take the 5-15cm layer mean value as representative topsoil
            val = depths[1].get("values", {}).get("mean")
            if val is not None:
                # SoilGrids stores fractions as g/kg (divide by 10 to get %)
                # bdod is cg/cm3 (divide by 100 to get g/cm3)
                if name in ["sand", "silt", "clay"]:
                    soil_data[name] = val / 10.0
                elif name == "bdod":
                    soil_data["bdod"] = val / 100.0

    if not all(k in soil_data for k in ["sand", "clay", "bdod"]):
        return {"error": "Incomplete soil texture data returned"}
        
    sand = soil_data["sand"]
    clay = soil_data["clay"]
    bulk_density = soil_data["bdod"]
    
    # Apply simplified Pedotransfer Function (PTF) (Saxton & Rawls 2006 approx)
    # Porosity (thetaS)
    porosity = 1.0 - (bulk_density / 2.65)
    
    # Field capacity & wilting point approx
    theta_fc = 0.2576 - 0.002 * sand + 0.0036 * clay + 0.0299 * bulk_density
    theta_wp = 0.026 + 0.005 * clay + 0.0158 * bulk_density
    
    # Saturated hydraulic conductivity Ksat (mm/hr)
    # Using an empirical exponential function based on porosity and clay
    ksat_mm_hr = math.exp(12.012 - 0.0755 * sand + (-3.895 + 0.03671 * sand - 0.1103 * clay + 8.7546 * (1-porosity)) / porosity) * 10.0
    
    # Cap maximum values to realistic ranges for Wflow
    if ksat_mm_hr > 500: ksat_mm_hr = 500.0
    if ksat_mm_hr < 1: ksat_mm_hr = 1.0
    
    return {
        "location": {"lat": lat, "lon": lon},
        "soilgrids_raw": soil_data,
        "derived_hydrological_params": {
            "thetaS": round(porosity, 3),
            "thetaR": round(theta_wp * 0.8, 3), # Residual water content approx
            "KsatVer_mm_day": round(ksat_mm_hr * 24.0, 2),
        },
        "method": "SoilGrids 250m + Saxton & Rawls (2006) Pedotransfer Function"
    }

def main():
    if len(sys.argv) < 3:
        print(json.dumps({"error": "Require lat and lon"}))
        sys.exit(1)
    
    lat = float(sys.argv[1])
    lon = float(sys.argv[2])
    res = fetch_soilgrids(lat, lon)
    print(json.dumps(res, indent=2))

if __name__ == "__main__":
    main()
