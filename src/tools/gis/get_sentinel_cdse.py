import sys
import argparse
import requests
import os
import datetime

# --- CARA MENDAPATKAN TOKEN ---
# 1. Daftar gratis di https://dataspace.copernicus.eu/
# 2. Buat OAuth Client di dashboard
# 3. Dapatkan CLIENT_ID dan CLIENT_SECRET

CLIENT_ID = os.environ.get("COP_CLIENT_ID", "")
CLIENT_SECRET = os.environ.get("COP_CLIENT_SECRET", "")

def get_token():
    auth_url = "https://identity.dataspace.copernicus.eu/auth/realms/CDSE/protocol/openid-connect/token"
    data = {
        "grant_type": "client_credentials",
        "client_id": CLIENT_ID,
        "client_secret": CLIENT_SECRET
    }
    r = requests.post(auth_url, data=data)
    if r.status_code == 200:
        return r.json()["access_token"]
    return None

def download_sentinel_direct(lon, lat, buffer_km, output_tif):
    if not CLIENT_ID or not CLIENT_SECRET:
        return "ERROR: Kredensial Copernicus belum diatur. Set ENV COP_CLIENT_ID dan COP_CLIENT_SECRET."

    token = get_token()
    if not token:
        return "ERROR: Gagal mendapatkan token Copernicus."

    # Hitung bounding box sederhana (1 derajat lat/lon ~ 111 km)
    offset = buffer_km / 111.0
    min_lon = lon - offset
    min_lat = lat - offset
    max_lon = lon + offset
    max_lat = lat + offset

    # API Processing Copernicus Sentinel Hub
    process_url = "https://sh.dataspace.copernicus.eu/api/v1/process"
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }

    # Evalscript: Kode Javascript yang dijalankan di server ESA untuk memilih band True Color (RGB)
    evalscript = """
    //VERSION=3
    function setup() {
        return {
            input: ["B02", "B03", "B04", "dataMask"],
            output: { bands: 3, sampleType: "AUTO" }
        };
    }
    function evaluatePixel(sample) {
        // Boost brightness (2.5x)
        return [sample.B04 * 2.5, sample.B03 * 2.5, sample.B02 * 2.5];
    }
    """

    payload = {
        "input": {
            "bounds": {
                "properties": {"crs": "http://www.opengis.net/def/crs/EPSG/0/3857"}, # Minta dalam format Web Mercator (EPSG:3857)
                # Bbox dalam WGS84, API akan mengkonversi
                "bbox": [min_lon, min_lat, max_lon, max_lat]
            },
            "data": [{
                "type": "sentinel-2-l2a", # L2A = Surface Reflectance
                "dataFilter": {
                    "timeRange": {
                        "from": (datetime.datetime.now() - datetime.timedelta(days=30)).strftime("%Y-%m-%dT00:00:00Z"),
                        "to": datetime.datetime.now().strftime("%Y-%m-%dT23:59:59Z")
                    },
                    "maxCloudCoverage": 30
                }
            }]
        },
        "output": {
            "width": 1024, # Resolusi output gambar
            "height": 1024,
            "responses": [{"identifier": "default", "format": {"type": "image/tiff"}}]
        },
        "evalscript": evalscript
    }

    print("Requesting data directly from Copernicus European Space Agency...")
    r = requests.post(process_url, headers=headers, json=payload)
    
    if r.status_code == 200:
        with open(output_tif, 'wb') as f:
            f.write(r.content)
        return f"SUCCESS: Download citra selesai. Tersimpan di {output_tif}"
    else:
        return f"ERROR: Copernicus API membalas dengan status {r.status_code}\n{r.text}"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--lon", type=float, required=True)
    parser.add_argument("--lat", type=float, required=True)
    parser.add_argument("--buffer_km", type=float, default=5.0)
    parser.add_argument("--output", type=str, required=True)
    args = parser.parse_args()
    
    print(download_sentinel_direct(args.lon, args.lat, args.buffer_km, args.output))
