import sys
import json
import requests
try:
    import pystac_client
    import planetary_computer
except ImportError:
    print("ERROR: pystac-client or planetary-computer not installed. Run: pip install pystac-client planetary-computer")
    sys.exit(1)

def query_alos(lat, lon, start_date, end_date):
    """
    Queries Microsoft Planetary Computer for JAXA ALOS PALSAR-2 imagery.
    L-Band radar is crucial for Indonesia because it penetrates dense forest canopies,
    unlike Sentinel-1 (C-Band) which only bounces off the top of the leaves.
    """
    catalog = pystac_client.Client.open(
        "https://planetarycomputer.microsoft.com/api/stac/v1",
        modifier=planetary_computer.sign_inplace,
    )
    
    # Point geometry
    point = {"type": "Point", "coordinates": [lon, lat]}
    
    # Search the ALOS PALSAR-2 mosaic collection
    # JAXA provides yearly global mosaics of ALOS PALSAR-2 at 25m resolution.
    try:
        search = catalog.search(
            collections=["alos-fnf-mosaic"], # Forest/Non-Forest and Backscatter
            intersects=point,
            datetime=f"{start_date}/{end_date}"
        )
        
        items = list(search.items())
        
        if not items:
            print(json.dumps({"status": "empty", "message": "No ALOS PALSAR data found for this time/location."}))
            return
            
        latest_item = items[0]
        
        assets = {}
        for key, asset in latest_item.assets.items():
            assets[key] = asset.href
            
        print(json.dumps({
            "status": "success",
            "scene_id": latest_item.id,
            "datetime": latest_item.datetime.isoformat() if latest_item.datetime else "N/A",
            "platform": "JAXA ALOS-2",
            "sensor": "PALSAR-2 (L-Band SAR)",
            "download_links": assets
        }, indent=2))
        
    except Exception as e:
        print(f"ERROR querying Planetary Computer: {e}")
        sys.exit(1)

if __name__ == "__main__":
    if len(sys.argv) < 5:
        print("Usage: python planetary_computer_engine.py <lat> <lon> <start_date> <end_date>")
        sys.exit(1)
        
    lat = float(sys.argv[1])
    lon = float(sys.argv[2])
    start = sys.argv[3]
    end = sys.argv[4]
    
    query_alos(lat, lon, start, end)
