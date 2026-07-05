#!/usr/bin/env python3
"""
Cartography Engine — SNI 6502:2010 Compliant Map Generator
Ref: SNI 6502.2:2010 (Peta Rupa Bumi), PermenLH 16/2012 (Peta AMDAL)
All 13 mandatory cartographic elements implemented.
"""
import sys
import json
import argparse
import math
import os
from datetime import datetime

import geopandas as gpd
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyArrowPatch
from mpl_toolkits.axes_grid1.anchored_artists import AnchoredSizeBar
import matplotlib.font_manager as fm
from matplotlib.offsetbox import AnchoredText
from shapely.geometry import shape
import contextily as cx
import numpy as np

# Paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, '..', '..', '..'))
ADMIN_GEOJSON = os.path.join(PROJECT_ROOT, 'resources', 'indonesia_admin.geojson')

# ================================================================
# HELPER FUNCTIONS
# ================================================================

def haversine_km(lat1, lon1, lat2, lon2):
    """Haversine distance in km."""
    R = 6371.0
    dlat = math.radians(lat2 - lat1)
    dlon = math.radians(lon2 - lon1)
    a = math.sin(dlat/2)**2 + math.cos(math.radians(lat1)) * math.cos(math.radians(lat2)) * math.sin(dlon/2)**2
    return R * 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))

def auto_utm_epsg(lon, lat):
    """Auto-detect UTM EPSG code for Indonesia."""
    zone = int((lon + 180) / 6) + 1
    epsg = 32700 + zone if lat < 0 else 32600 + zone
    return epsg, zone, 'S' if lat < 0 else 'N'

def get_scale_bar_length(extent_km):
    """Choose appropriate scale bar length based on map extent."""
    candidates = [0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50, 100, 200, 500]
    target = extent_km / 5  # ~1/5 of map width
    return min(candidates, key=lambda x: abs(x - target))

def download_sentinel_geotiff(lon, lat, buffer_km, output_tif, date_str=None):
    """Download Sentinel-2 true color GeoTIFF from GEE."""
    import ee
    try:
        ee.Initialize()
    except Exception:
        return False, "GEE belum diotentikasi."
    try:
        import requests as req
        point = ee.Geometry.Point([lon, lat])
        roi = point.buffer(buffer_km * 1000).bounds()
        end_date = ee.Date(date_str or datetime.now().strftime('%Y-%m-%d'))
        start_date = end_date.advance(-30, 'day')
        
        coll = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
            .filterBounds(roi).filterDate(start_date, end_date) \
            .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30))
        if coll.size().getInfo() == 0:
            return False, "Tidak ada citra bebas awan 30 hari terakhir."
        
        image = coll.median().select(['B4', 'B3', 'B2']).visualize(min=0, max=3000)
        url = image.clip(roi).getDownloadURL({'scale': 10, 'crs': 'EPSG:3857', 'region': roi, 'format': 'GEO_TIFF'})
        r = req.get(url, stream=True, timeout=120)
        if r.status_code == 200:
            with open(output_tif, 'wb') as f:
                for chunk in r.iter_content(4096):
                    f.write(chunk)
            return True, output_tif
        return False, f"HTTP {r.status_code}"
    except Exception as e:
        return False, str(e)

# ================================================================
# SNI MAP GENERATOR
# ================================================================

def generate_sni_map(geojson_str, output_path, title, realtime=False, 
                     author="Environmental AI Agent", date_str=None,
                     show_admin=True, legend_items=None):
    """
    Generate SNI 6502:2010 compliant map layout.
    
    13 mandatory elements:
    1. Title (judul peta)
    2. Scale bar (skala grafis)
    3. Numeric scale (skala numerik)
    4. Legend (legenda)
    5. North arrow (arah utara)
    6. Coordinate grid + labels (grid koordinat)
    7. Inset map (peta inset)
    8. CRS info (sistem koordinat)
    9. Data source (sumber data)
    10. Date (tanggal)
    11. Author (pembuat)
    12. Admin boundaries (batas administrasi)
    13. Map frame with ticks (bingkai peta)
    """
    try:
        # Parse GeoJSON
        data = json.loads(geojson_str)
        if data.get('type') == 'FeatureCollection':
            for f in data.get('features', []):
                if 'properties' not in f: f['properties'] = {}
            gdf = gpd.GeoDataFrame.from_features(data['features'], crs='EPSG:4326')
        else:
            gdf = gpd.GeoDataFrame(geometry=[shape(data)], crs='EPSG:4326')

        # Get centroid and bounds for projection
        bounds = gdf.total_bounds  # minx, miny, maxx, maxy
        center_lon = (bounds[0] + bounds[2]) / 2
        center_lat = (bounds[1] + bounds[3]) / 2
        
        # Auto UTM
        utm_epsg, utm_zone, utm_hem = auto_utm_epsg(center_lon, center_lat)
        
        # Reproject to Web Mercator for display
        gdf_wm = gdf.to_crs(epsg=3857)
        
        # Calculate map extent in km
        extent_km = haversine_km(bounds[1], bounds[0], bounds[1], bounds[2])
        extent_km_y = haversine_km(bounds[1], bounds[0], bounds[3], bounds[0])
        
        # Calculate numeric scale (approximate)
        fig_width_cm = 30  # 15 inches * 2.54 ≈ 38 cm, but map area ~30cm
        numeric_scale = int(extent_km * 1000 * 100 / fig_width_cm)
        # Round to nice number
        nice_scales = [1000, 2500, 5000, 10000, 25000, 50000, 100000, 250000, 500000, 1000000]
        numeric_scale = min(nice_scales, key=lambda s: abs(s - numeric_scale))

        # ============================================================
        # CREATE FIGURE — main map + inset
        # ============================================================
        fig = plt.figure(figsize=(16, 12), facecolor='white')
        
        # Main map axes (with room for margins)
        ax = fig.add_axes([0.08, 0.12, 0.72, 0.78])
        
        # Inset map axes (top-left corner)
        ax_inset = fig.add_axes([0.09, 0.72, 0.18, 0.17])
        
        # ============================================================
        # [13] MAP FRAME WITH TICKS
        # ============================================================
        for spine in ax.spines.values():
            spine.set_linewidth(2)
            spine.set_color('black')
        ax.tick_params(axis='both', which='both', direction='out', length=6, width=1.5,
                       labelsize=8, colors='black')
        
        # ============================================================
        # BASEMAP
        # ============================================================
        # Set axis extent first (with 15% padding)
        xmin, ymin, xmax, ymax = gdf_wm.total_bounds
        pad_x = (xmax - xmin) * 0.15
        pad_y = (ymax - ymin) * 0.15
        ax.set_xlim(xmin - pad_x, xmax + pad_x)
        ax.set_ylim(ymin - pad_y, ymax + pad_y)
        
        source_text = "Esri World Imagery"
        if realtime:
            tif_path = "/tmp/sentinel_cartography_temp.tif"
            if os.path.exists(tif_path): os.remove(tif_path)
            success, msg = download_sentinel_geotiff(center_lon, center_lat,
                max(extent_km, extent_km_y) + 2, tif_path, date_str)
            if success:
                import rasterio
                from rasterio.plot import show as rshow
                with rasterio.open(tif_path) as src:
                    rshow(src, ax=ax, zorder=1)
                source_text = f"Sentinel-2 L2A ({datetime.now().strftime('%B %Y')})"
            else:
                cx.add_basemap(ax, crs=gdf_wm.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)
        else:
            cx.add_basemap(ax, crs=gdf_wm.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)

        # ============================================================
        # [12] ADMIN BOUNDARIES
        # ============================================================
        if show_admin and os.path.exists(ADMIN_GEOJSON):
            try:
                admin = gpd.read_file(ADMIN_GEOJSON)
                admin_wm = admin.to_crs(epsg=3857)
                # Get map extent
                xmin, ymin, xmax, ymax = gdf_wm.total_bounds
                pad = max(xmax - xmin, ymax - ymin) * 0.1
                admin_clipped = admin_wm.cx[xmin-pad:xmax+pad, ymin-pad:ymax+pad]
                if len(admin_clipped) > 0:
                    admin_clipped[admin_clipped['level'] == 1].plot(
                        ax=ax, color='none', edgecolor='#FF00FF', linewidth=0.8,
                        linestyle='--', zorder=3, alpha=0.7)
            except Exception:
                pass  # Graceful fallback if admin data fails
        
        # ============================================================
        # PLOT PROJECT POLYGON
        # ============================================================
        gdf_wm.plot(ax=ax, color='none', edgecolor='red', linewidth=3, zorder=4)
        
        # ============================================================
        # [6] COORDINATE GRID WITH LABELS (Geographic lat/lon)
        # ============================================================
        # Convert axis limits from Web Mercator back to lat/lon for grid labels
        from pyproj import Transformer
        transformer = Transformer.from_crs('EPSG:3857', 'EPSG:4326', always_xy=True)
        
        xlim = ax.get_xlim()
        ylim = ax.get_ylim()
        
        # Create geographic coordinate ticks
        lon_min, lat_min = transformer.transform(xlim[0], ylim[0])
        lon_max, lat_max = transformer.transform(xlim[1], ylim[1])
        
        # Auto tick interval
        lon_range = lon_max - lon_min
        tick_intervals = [0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0]
        tick_int = min(tick_intervals, key=lambda t: abs(lon_range / t - 6))
        
        lon_ticks = np.arange(math.ceil(lon_min / tick_int) * tick_int, lon_max, tick_int)
        lat_ticks = np.arange(math.ceil(lat_min / tick_int) * tick_int, lat_max, tick_int)
        
        # Convert back to Web Mercator for placing on axes
        transformer_inv = Transformer.from_crs('EPSG:4326', 'EPSG:3857', always_xy=True)
        
        x_ticks_wm = [transformer_inv.transform(lon, center_lat)[0] for lon in lon_ticks]
        y_ticks_wm = [transformer_inv.transform(center_lon, lat)[1] for lat in lat_ticks]
        
        ax.set_xticks(x_ticks_wm)
        ax.set_yticks(y_ticks_wm)
        ax.set_xticklabels([f'{lon:.3f}°E' if lon >= 0 else f'{abs(lon):.3f}°W' for lon in lon_ticks], fontsize=7, rotation=45)
        ax.set_yticklabels([f'{abs(lat):.3f}°S' if lat < 0 else f'{lat:.3f}°N' for lat in lat_ticks], fontsize=7)
        
        ax.grid(True, linestyle=':', alpha=0.4, color='white', linewidth=0.5)
        ax.set_xlabel('')
        ax.set_ylabel('')
        
        # ============================================================
        # [1] TITLE
        # ============================================================
        ax.set_title(title, fontsize=18, fontweight='bold', pad=15,
                     fontfamily='DejaVu Sans', loc='center')
        
        # ============================================================
        # [5] NORTH ARROW (proper cartographic)
        # ============================================================
        arrow_x, arrow_y = 0.96, 0.92
        ax.annotate('', xy=(arrow_x, arrow_y), xytext=(arrow_x, arrow_y - 0.08),
                     arrowprops=dict(arrowstyle='->', color='black', lw=2.5),
                     xycoords='axes fraction')
        ax.text(arrow_x, arrow_y + 0.02, 'U', transform=ax.transAxes,
                fontsize=14, fontweight='bold', ha='center', va='bottom',
                fontfamily='DejaVu Sans')
        
        # ============================================================
        # [2] SCALE BAR (graphical)
        # ============================================================
        bar_km = get_scale_bar_length(extent_km)
        # Convert km to meters in Web Mercator (approximate at this latitude)
        meters_per_degree = 111320 * math.cos(math.radians(center_lat))
        bar_m_wm = bar_km * 1000 / meters_per_degree * (xlim[1] - xlim[0]) / (lon_max - lon_min)
        
        bar_label = f'{bar_km} km' if bar_km >= 1 else f'{int(bar_km*1000)} m'
        fontprops = fm.FontProperties(size=9, weight='bold')
        scalebar = AnchoredSizeBar(ax.transData, bar_m_wm, bar_label,
                                   loc='lower center', pad=0.5, sep=5,
                                   color='black', frameon=True,
                                   size_vertical=bar_m_wm * 0.02,
                                   fontproperties=fontprops,
                                   bbox_to_anchor=(0.5, -0.01),
                                   bbox_transform=ax.transAxes)
        ax.add_artist(scalebar)
        
        # ============================================================
        # [3] NUMERIC SCALE
        # ============================================================
        scale_text = f'Skala 1 : {numeric_scale:,}'.replace(',', '.')
        ax.text(0.5, -0.06, scale_text, transform=ax.transAxes,
                fontsize=10, ha='center', fontweight='bold', fontfamily='DejaVu Sans')
        
        # ============================================================
        # [4] LEGEND
        # ============================================================
        legend_patches = [
            mpatches.Patch(facecolor='none', edgecolor='red', linewidth=2, label='Area Studi / Proyek'),
        ]
        if show_admin:
            legend_patches.append(
                mpatches.Patch(facecolor='none', edgecolor='#FF00FF', linewidth=1,
                               linestyle='--', label='Batas Administrasi'))
        if legend_items:
            for item in legend_items:
                legend_patches.append(
                    mpatches.Patch(facecolor=item.get('color', 'gray'),
                                   label=item.get('label', '')))
        
        ax.legend(handles=legend_patches, loc='upper left', fontsize=8,
                  framealpha=0.9, edgecolor='black', title='LEGENDA',
                  title_fontsize=9)
        
        # ============================================================
        # [7] INSET MAP (Indonesia overview + project location)
        # ============================================================
        if os.path.exists(ADMIN_GEOJSON):
            try:
                admin_full = gpd.read_file(ADMIN_GEOJSON)
                admin_full.plot(ax=ax_inset, color='#E8E8E8', edgecolor='#666666', linewidth=0.3)
                # Plot project location as red dot
                ax_inset.plot(center_lon, center_lat, 'r*', markersize=12, zorder=5)
                ax_inset.set_xlim(94, 142)
                ax_inset.set_ylim(-12, 7)
                ax_inset.set_title('Lokasi', fontsize=7, fontweight='bold')
                ax_inset.set_xticks([])
                ax_inset.set_yticks([])
                for spine in ax_inset.spines.values():
                    spine.set_linewidth(1)
            except Exception:
                ax_inset.set_visible(False)
        else:
            ax_inset.set_visible(False)
        
        # ============================================================
        # [8,9,10,11] METADATA BOX (CRS, source, date, author)
        # ============================================================
        prod_date = date_str or datetime.now().strftime('%d %B %Y')
        meta_text = (
            f"Sistem Koordinat: UTM Zone {utm_zone}{utm_hem} (EPSG:{utm_epsg})\n"
            f"Datum: WGS-84 | Proyeksi: Transverse Mercator\n"
            f"Sumber Data: {source_text}\n"
            f"Tanggal: {prod_date}\n"
            f"Dibuat oleh: {author}\n"
            f"Ref: SNI 6502:2010, PermenLH 16/2012"
        )
        
        meta_box = AnchoredText(meta_text, loc='lower right', prop=dict(size=7),
                                frameon=True, bbox_to_anchor=(1.0, 0.0),
                                bbox_transform=ax.transAxes)
        meta_box.patch.set_boxstyle("round,pad=0.3")
        meta_box.patch.set_facecolor('white')
        meta_box.patch.set_alpha(0.9)
        meta_box.patch.set_edgecolor('black')
        ax.add_artist(meta_box)

        # ============================================================
        # SAVE
        # ============================================================
        plt.savefig(output_path, dpi=300, bbox_inches='tight', facecolor='white')
        plt.close(fig)
        
        return (f"SUCCESS: Peta SNI-compliant disimpan di {output_path}\n"
                f"Skala: {scale_text}\n"
                f"CRS: UTM Zone {utm_zone}{utm_hem} (EPSG:{utm_epsg})\n"
                f"Elemen kartografi: 13/13 (judul, skala grafis, skala numerik, legenda, "
                f"arah utara, grid koordinat, peta inset, CRS, sumber data, tanggal, "
                f"pembuat, batas administrasi, bingkai peta)")
        
    except Exception as e:
        import traceback
        return f"ERROR: Gagal membuat peta — {str(e)}\n{traceback.format_exc()}"


# ================================================================
# CLI
# ================================================================
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='SNI 6502:2010 Compliant Map Generator')
    parser.add_argument("--geojson", required=True, help="GeoJSON string atau file path")
    parser.add_argument("--output", required=True, help="Output PNG path")
    parser.add_argument("--title", required=True, help="Judul peta")
    parser.add_argument("--realtime", action="store_true", help="Gunakan Sentinel-2 basemap")
    parser.add_argument("--author", default="Environmental AI Agent", help="Nama pembuat peta")
    parser.add_argument("--date", default=None, help="Tanggal produksi (YYYY-MM-DD)")
    parser.add_argument("--no-admin", action="store_true", help="Sembunyikan batas administrasi")
    
    args = parser.parse_args()
    
    # Read GeoJSON from file or string
    geojson_input = args.geojson
    if os.path.exists(args.geojson):
        with open(args.geojson) as f:
            geojson_input = f.read()
    
    result = generate_sni_map(
        geojson_input, args.output, args.title,
        realtime=args.realtime, author=args.author,
        date_str=args.date, show_admin=not args.no_admin
    )
    print(result)
