import sys
import requests
import argparse
from datetime import datetime, timedelta

def get_latest_direct_image(lon, lat):
    print(f"Mencari citra resolusi tinggi (10m) TERBARU dari ESA Copernicus untuk koordinat {lat}, {lon}...")
    
    # Hitung Bounding Box (Kecil saja, 5km)
    buffer = 0.05
    min_lon, min_lat = lon - buffer, lat - buffer
    max_lon, max_lat = lon + buffer, lat + buffer
    
    # Format waktu: 14 hari terakhir
    end_date = datetime.utcnow()
    start_date = end_date - timedelta(days=14)
    start_str = start_date.strftime('%Y-%m-%dT%H:%M:%SZ')
    end_str = end_date.strftime('%Y-%m-%dT%H:%M:%SZ')
    
    # OData API Query (Copernicus Data Space Ecosystem) - API Resmi Satelit Eropa
    # Sentinel-2 Level-2A (Sudah dikoreksi atmosfernya - tajam)
    base_url = "https://catalogue.dataspace.copernicus.eu/odata/v1/Products"
    
    # Filter: Koleksi S2 L2A, Cloud Cover < 20%, Intersect dengan Bounding Box
    filter_query = (
        "Collection/Name eq 'SENTINEL-2' and "
        "Attributes/OData.CSC.StringAttribute/any(att:att/Name eq 'productType' and att/OData.CSC.StringAttribute/Value eq 'S2MSI2A') and "
        "Attributes/OData.CSC.DoubleAttribute/any(att:att/Name eq 'cloudCover' and att/OData.CSC.DoubleAttribute/Value le 20.0) and "
        f"OData.CSC.Intersects(area=geography'SRID=4326;POLYGON(({min_lon} {min_lat}, {max_lon} {min_lat}, {max_lon} {max_lat}, {min_lon} {max_lat}, {min_lon} {min_lat}))')"
    )
    
    params = {
        "$filter": filter_query,
        "$orderby": "ContentDate/Start desc", # Paling baru di atas
        "$top": 3 # Ambil 3 terbaik
    }
    
    try:
        r = requests.get(base_url, params=params)
        r.raise_for_status()
        data = r.json()
        
        results = data.get("value", [])
        if not results:
            return "Tidak ditemukan citra dengan awan <20% dalam 14 hari terakhir. Coba naikkan toleransi awan."
            
        output = "=== CITRA SENTINEL-2 TERBARU (API LANGSUNG ESA) ===\n"
        for i, item in enumerate(results):
            name = item.get("Name")
            date = item.get("ContentDate", {}).get("Start")
            product_id = item.get("Id")
            
            # Cari persentase awan di metadata
            cloud_cover = "N/A"
            for attr in item.get("Attributes", []):
                if attr.get("Name") == "cloudCover":
                    cloud_cover = attr.get("Value")
            
            output += f"\n{i+1}. TANGGAL FOTO: {date}\n"
            output += f"   AWAN: {cloud_cover}%\n"
            output += f"   NAMA FILE: {name}\n"
            output += f"   🔗 DOWNLOAD LINK: https://browser.dataspace.copernicus.eu/?zoom=13&lat={lat}&lng={lon}&themeId=DEFAULT-THEME&datasetId=S2_L2A_CDAS&fromTime={date[:10]}T00:00:00.000Z&toTime={date[:10]}T23:59:59.999Z&layerId=1_TRUE_COLOR\n"
            
        output += "\nCatatan: Klik 'DOWNLOAD LINK' untuk melihat dan mengunduh citra warna asli (True Color) resolusi 10 meter secara gratis melalui web browser Copernicus."
        return output
        
    except Exception as e:
        return f"Error menghubungi Copernicus API: {str(e)}"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--lon", type=float, required=True)
    parser.add_argument("--lat", type=float, required=True)
    args = parser.parse_args()
    
    print(get_latest_direct_image(args.lon, args.lat))