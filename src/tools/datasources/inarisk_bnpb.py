import sys
import json
import urllib.request
import urllib.parse

def fetch_inarisk_hazard(lat, lon, hazard_type="banjir"):
    """
    Fetches official disaster risk index from BNPB InaRISK ArcGIS REST server.
    This replaces hardcoded values with authentic Indonesian government data.
    """
    # Mapping hazard to InaRISK Layer IDs
    # Based on public InaRISK WFS/REST endpoints
    # 0 = Banjir, 1 = Tanah Longsor, 2 = Karhutla, 4 = Tsunami (simplified IDs for example)
    layer_map = {
        "banjir": "0",
        "longsor": "1",
        "karhutla": "2",
        "tsunami": "4"
    }
    
    layer_id = layer_map.get(hazard_type.lower(), "0")
    
    # Construct ArcGIS REST Query URL
    # We use spatial intersection (geometry) query
    base_url = "https://gis.bnpb.go.id/arcgis/rest/services/InaRISK/Indeks_Bahaya/MapServer"
    query_url = f"{base_url}/{layer_id}/query"
    
    params = {
        "f": "json",
        "geometry": f"{lon},{lat}",
        "geometryType": "esriGeometryPoint",
        "spatialRel": "esriSpatialRelIntersects",
        "outFields": "*",
        "returnGeometry": "false"
    }
    
    encoded_params = urllib.parse.urlencode(params)
    full_url = f"{query_url}?{encoded_params}"
    
    try:
        req = urllib.request.Request(full_url, headers={'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'})
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode())
            
            if 'features' in data and len(data['features']) > 0:
                # Extract risk class
                attributes = data['features'][0].get('attributes', {})
                # Try common InaRISK field names
                risk_class = attributes.get('KELAS_BAHAYA', attributes.get('kelas_bahaya', attributes.get('GRIDCODE', 'Unknown')))
                print(json.dumps({
                    "status": "success",
                    "hazard": hazard_type,
                    "risk_class": risk_class,
                    "source": "BNPB InaRISK",
                    "lat": lat,
                    "lon": lon
                }))
            else:
                print(json.dumps({
                    "status": "not_found",
                    "hazard": hazard_type,
                    "risk_class": "Aman / Tidak Terdampak",
                    "source": "BNPB InaRISK",
                    "message": "No hazard feature found at this coordinate."
                }))
                
    except Exception as e:
        print(json.dumps({
            "status": "error",
            "message": str(e)
        }))

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print(json.dumps({"status": "error", "message": "Usage: python inarisk_bnpb.py <lat> <lon> <hazard_type>"}))
        sys.exit(1)
        
    lat = float(sys.argv[1])
    lon = float(sys.argv[2])
    hazard = sys.argv[3]
    
    fetch_inarisk_hazard(lat, lon, hazard)
