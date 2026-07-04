import sys
import json
import geopandas as gpd
from shapely.geometry import shape

def analyze_area(geojson_str):
    try:
        data = json.loads(geojson_str)
        if 'type' in data and data['type'] == 'FeatureCollection':
            gdf = gpd.GeoDataFrame.from_features(data["features"], crs="EPSG:4326")
        else:
            geom = shape(data if 'geometry' not in data else data['geometry'])
            gdf = gpd.GeoDataFrame(geometry=[geom], crs="EPSG:4326")
        
        # Reproject to Cylindrical Equal Area (EPSG:6933) for accurate area calculation globally
        gdf_ea = gdf.to_crs(epsg=6933)
        
        total_area_m2 = gdf_ea.geometry.area.sum()
        total_area_ha = total_area_m2 / 10000.0
        total_area_km2 = total_area_m2 / 1e6
        
        bounds = gdf.total_bounds # [minx, miny, maxx, maxy]
        
        return f"\n[ANALISIS SPASIAL GEOPANDAS]\nArea Total: {total_area_ha:.2f} Hektar ({total_area_km2:.3f} km²)\nBounding Box: {bounds[0]:.4f}, {bounds[1]:.4f}, {bounds[2]:.4f}, {bounds[3]:.4f}"
    except Exception as e:
        return f"\n[ANALISIS SPASIAL GEOPANDAS]\nGagal menghitung area: {e}"

if __name__ == "__main__":
    if len(sys.argv) > 1:
        print(analyze_area(sys.argv[1]))
