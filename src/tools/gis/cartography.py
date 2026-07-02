import sys
import json
import argparse
import geopandas as gpd
import matplotlib.pyplot as plt
from shapely.geometry import shape, Point
import contextily as cx
import os
import rasterio
from rasterio.plot import show
import requests

def download_sentinel_geotiff(lon, lat, buffer_km, output_tif):
    import ee
    try:
        ee.Initialize()
    except Exception:
        return False, "Google Earth Engine belum diotentikasi."

    try:
        point = ee.Geometry.Point([lon, lat])
        roi = point.buffer(buffer_km * 1000).bounds()
        end_date = ee.Date(sys.argv[4] if len(sys.argv) > 4 else '2026-07-02') 
        start_date = end_date.advance(-30, 'day')

        collection = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
            .filterBounds(roi) \
            .filterDate(start_date, end_date) \
            .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30))

        if collection.size().getInfo() == 0:
            return False, "Tidak ada citra bebas awan 30 hari terakhir."

        # Pilih true color, scale 0-10000 to uint8 untuk plot
        image = collection.median().select(['B4', 'B3', 'B2'])
        vis_params = {'min': 0, 'max': 3000}
        image_vis = image.visualize(**vis_params)
        clipped = image_vis.clip(roi)

        url = clipped.getDownloadURL({
            'scale': 10,
            'crs': 'EPSG:3857', # Langsung minta format Web Mercator agar cocok dengan contextily
            'region': roi,
            'format': 'GEO_TIFF'
        })
        
        # Download file
        r = requests.get(url, stream=True)
        if r.status_code == 200:
            with open(output_tif, 'wb') as f:
                for chunk in r.iter_content(1024):
                    f.write(chunk)
            return True, output_tif
        return False, f"HTTP Error {r.status_code}"
    except Exception as e:
        return False, str(e)


def create_sni_map(geojson_str, output_path, title, realtime=False):
    try:
        data = json.loads(geojson_str)
        if 'type' in data and data['type'] == 'FeatureCollection':
            for f in data.get("features", []):
                if "properties" not in f:
                    f["properties"] = {}
            gdf = gpd.GeoDataFrame.from_features(data["features"], crs="EPSG:4326")
        else:
            geom = shape(data)
            gdf = gpd.GeoDataFrame(geometry=[geom], crs="EPSG:4326")

        gdf_wm = gdf.to_crs(epsg=3857)
        fig, ax = plt.subplots(figsize=(15, 10))
        
        # Plot Polygon
        gdf_wm.plot(ax=ax, color='none', edgecolor='red', linewidth=3, zorder=2)
        
        source_text = "Sumber Basis: Esri World Imagery"
        
        if realtime:
            # Ambil centroid polygon untuk query Sentinel
            centroid = gdf.geometry.centroid.iloc[0]
            bounds = gdf.total_bounds # minx, miny, maxx, maxy
            # Estimasikan radius/buffer (kasar) dalam km
            buffer_km = max(bounds[2]-bounds[0], bounds[3]-bounds[1]) * 111 + 2.0
            
            tif_path = "/tmp/sentinel_temp.tif"
            if os.path.exists(tif_path):
                os.remove(tif_path) # Hapus file lama agar tidak memakan space
                
            success, msg = download_sentinel_geotiff(centroid.x, centroid.y, buffer_km, tif_path)
            
            if success:
                src = rasterio.open(tif_path)
                show(src, ax=ax, zorder=1)
                source_text = "Sumber Basis: Sentinel-2 L2A (Analisis Spasial)"
            else:
                print(f"Warning: Sentinel download gagal ({msg}). Fallback ke Esri.")
                cx.add_basemap(ax, crs=gdf_wm.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)
        else:
            cx.add_basemap(ax, crs=gdf_wm.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)
        
        ax.set_title(title, fontsize=20, fontweight='bold', pad=20, loc='center')
        ax.set_xlabel("Easting (Meter - EPSG:3857)", fontsize=12)
        ax.set_ylabel("Northing (Meter - EPSG:3857)", fontsize=12)
        ax.grid(True, linestyle='--', alpha=0.5, color='white')
        
        x, y, arrow_length = 0.95, 0.95, 0.05
        ax.annotate('U', xy=(x, y), xytext=(x, y-arrow_length),
            arrowprops=dict(facecolor='black', width=3, headwidth=10),
            ha='center', va='center', fontsize=16, fontweight='bold',
            xycoords=ax.transAxes)

        ax.text(0.02, 0.02, f"Sistem Koordinat: WGS 84 / Web Mercator\n{source_text}\nDicetak oleh: ZeroClaw AI Agent", 
                transform=ax.transAxes, fontsize=10, 
                bbox=dict(facecolor='white', alpha=0.8, edgecolor='black'))

        plt.tight_layout()
        plt.savefig(output_path, dpi=300, bbox_inches='tight')
        plt.close()
        
        return f"SUCCESS: Peta layout standar berhasil disimpan di {output_path}"
        
    except Exception as e:
        return f"ERROR: Gagal membuat peta - {str(e)}"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--geojson", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--realtime", action="store_true", help="Gunakan Sentinel-2 30 hari terakhir")
    
    args = parser.parse_args()
    print(create_sni_map(args.geojson, args.output, args.title, args.realtime))
