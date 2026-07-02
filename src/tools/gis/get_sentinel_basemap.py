import sys
import ee
import argparse

# Inisialisasi GEE (asumsi auth sudah ada di sistem)
try:
    ee.Initialize()
except Exception:
    print("ERROR: Google Earth Engine belum diotentikasi. Jalankan `earthengine authenticate` di terminal.")
    sys.exit(1)

def get_latest_sentinel2(lon, lat, buffer_km, output_path):
    try:
        # Bounding box
        point = ee.Geometry.Point([lon, lat])
        roi = point.buffer(buffer_km * 1000).bounds()

        # Ambil Sentinel-2 Surface Reflectance 30 hari terakhir
        end_date = ee.Date(sys.argv[4] if len(sys.argv) > 4 else '2026-07-02') # Pakai tgl sekarang atau input
        start_date = end_date.advance(-30, 'day')

        collection = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
            .filterBounds(roi) \
            .filterDate(start_date, end_date) \
            .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 20))

        if collection.size().getInfo() == 0:
            return "ERROR: Tidak ada citra Sentinel-2 bebas awan di area ini dalam 30 hari terakhir."

        # Mosaik dan pilih band True Color (RGB)
        image = collection.median().select(['B4', 'B3', 'B2'])
        
        # Clip ke ROI
        clipped = image.clip(roi)

        # Dapatkan URL download
        url = clipped.getDownloadURL({
            'scale': 10,  # Resolusi 10 meter (standar emas gratis)
            'crs': 'EPSG:4326',
            'region': roi,
            'format': 'GEO_TIFF'
        })
        
        return f"SUCCESS: {url}"
    except Exception as e:
        return f"ERROR: {str(e)}"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--lon", type=float, required=True)
    parser.add_argument("--lat", type=float, required=True)
    parser.add_argument("--buffer_km", type=float, default=5.0)
    args = parser.parse_args()
    
    print(get_latest_sentinel2(args.lon, args.lat, args.buffer_km, ""))
