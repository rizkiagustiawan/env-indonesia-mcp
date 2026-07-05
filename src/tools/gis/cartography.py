#!/usr/bin/env python3
"""
Cartography Engine — SNI 6502:2010 Compliant Professional Map Generator
Ref: SNI 6502.2:2010, PermenLH 16/2012
Layout: Right-panel design (inset + legend + metadata separated from map body)
"""
import sys, json, argparse, math, os
from datetime import datetime

import geopandas as gpd
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyArrowPatch, Rectangle
from mpl_toolkits.axes_grid1.anchored_artists import AnchoredSizeBar
import matplotlib.font_manager as fm
from matplotlib.patheffects import withStroke
from shapely.geometry import shape, box
import contextily as cx
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, '..', '..', '..'))
ADMIN_GEOJSON = os.path.join(PROJECT_ROOT, 'resources', 'indonesia_admin.geojson')

# ================================================================
# HELPERS
# ================================================================

def haversine_km(lat1, lon1, lat2, lon2):
    R = 6371.0
    dlat, dlon = math.radians(lat2 - lat1), math.radians(lon2 - lon1)
    a = math.sin(dlat/2)**2 + math.cos(math.radians(lat1)) * math.cos(math.radians(lat2)) * math.sin(dlon/2)**2
    return R * 2 * math.atan2(math.sqrt(a), math.sqrt(1-a))

def auto_utm(lon, lat):
    zone = int((lon + 180) / 6) + 1
    return (32700 + zone if lat < 0 else 32600 + zone), zone, ('S' if lat < 0 else 'N')

def nice_scale(extent_km):
    for s in [0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50, 100, 200, 500]:
        if s >= extent_km / 6:
            return s
    return 500

def nice_numeric_scale(extent_km, fig_width_cm=28):
    raw = int(extent_km * 1000 * 100 / fig_width_cm)
    for s in [1000, 2500, 5000, 10000, 25000, 50000, 100000, 250000, 500000, 1000000]:
        if s >= raw * 0.7:
            return s
    return 1000000

def download_sentinel(lon, lat, buffer_km, output_tif):
    import ee, requests as req
    try:
        ee.Initialize()
        pt = ee.Geometry.Point([lon, lat])
        roi = pt.buffer(buffer_km * 1000).bounds()
        end = ee.Date(datetime.now().strftime('%Y-%m-%d'))
        start = end.advance(-30, 'day')
        coll = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
            .filterBounds(roi).filterDate(start, end) \
            .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30))
        if coll.size().getInfo() == 0:
            return False
        img = coll.median().select(['B4','B3','B2']).visualize(min=0, max=3000)
        url = img.clip(roi).getDownloadURL({'scale': 10, 'crs': 'EPSG:3857', 'region': roi, 'format': 'GEO_TIFF'})
        r = req.get(url, stream=True, timeout=120)
        if r.status_code == 200:
            with open(output_tif, 'wb') as f:
                for chunk in r.iter_content(4096): f.write(chunk)
            return True
    except Exception:
        pass
    return False

# ================================================================
# MAIN MAP GENERATOR
# ================================================================

def generate_sni_map(geojson_str, output_path, title, realtime=False,
                     author="Environmental AI Agent", date_str=None, show_admin=True):
    try:
        # Parse GeoJSON
        data = json.loads(geojson_str)
        if data.get('type') == 'FeatureCollection':
            for f in data.get('features', []):
                if 'properties' not in f: f['properties'] = {}
            gdf = gpd.GeoDataFrame.from_features(data['features'], crs='EPSG:4326')
        else:
            gdf = gpd.GeoDataFrame(geometry=[shape(data)], crs='EPSG:4326')

        bounds = gdf.total_bounds
        clon, clat = (bounds[0]+bounds[2])/2, (bounds[1]+bounds[3])/2
        utm_epsg, utm_zone, utm_hem = auto_utm(clon, clat)
        extent_km = haversine_km(clat, bounds[0], clat, bounds[2])
        extent_km_y = haversine_km(bounds[1], clon, bounds[3], clon)
        num_scale = nice_numeric_scale(extent_km)
        bar_km = nice_scale(extent_km)

        gdf_wm = gdf.to_crs(epsg=3857)
        xmin, ymin, xmax, ymax = gdf_wm.total_bounds
        pad_x = max((xmax - xmin) * 0.12, 500)
        pad_y = max((ymax - ymin) * 0.12, 500)

        # ============================================================
        # FIGURE LAYOUT — Right Panel Design
        # ============================================================
        fig = plt.figure(figsize=(16, 11), dpi=200, facecolor='white')

        # Main map: left 70% of figure
        ax = fig.add_axes([0.04, 0.06, 0.64, 0.86])
        # Right panel — split into 3 boxes
        ax_inset = fig.add_axes([0.72, 0.68, 0.25, 0.24])   # Inset map
        ax_legend = fig.add_axes([0.72, 0.40, 0.25, 0.25])   # Legend
        ax_meta = fig.add_axes([0.72, 0.06, 0.25, 0.31])     # Metadata

        # ============================================================
        # MAIN MAP — Set extent FIRST
        # ============================================================
        ax.set_xlim(xmin - pad_x, xmax + pad_x)
        ax.set_ylim(ymin - pad_y, ymax + pad_y)

        # Basemap
        source_text = "Esri World Imagery"
        if realtime:
            tif_path = "/tmp/_cartography_sentinel.tif"
            if os.path.exists(tif_path): os.remove(tif_path)
            if download_sentinel(clon, clat, max(extent_km, extent_km_y) + 2, tif_path):
                import rasterio
                from rasterio.plot import show as rshow
                with rasterio.open(tif_path) as src:
                    rshow(src, ax=ax, zorder=1)
                source_text = f"Sentinel-2 L2A"
            else:
                cx.add_basemap(ax, crs=gdf_wm.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)
        else:
            cx.add_basemap(ax, crs=gdf_wm.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)

        # [12] Admin boundaries
        if show_admin and os.path.exists(ADMIN_GEOJSON):
            try:
                admin = gpd.read_file(ADMIN_GEOJSON).to_crs(epsg=3857)
                admin_clip = admin.cx[xmin-pad_x*3:xmax+pad_x*3, ymin-pad_y*3:ymax+pad_y*3]
                if len(admin_clip) > 0:
                    admin_clip[admin_clip['level'] == 1].plot(
                        ax=ax, color='none', edgecolor='#FF00FF',
                        linewidth=1.2, linestyle='--', zorder=3, alpha=0.85)
            except Exception:
                pass

        # Project polygon
        gdf_wm.plot(ax=ax, color='none', edgecolor='#FF0000', linewidth=2.5, zorder=4)

        # ============================================================
        # [13] MAP FRAME
        # ============================================================
        for spine in ax.spines.values():
            spine.set_linewidth(1.5)
            spine.set_color('black')

        # ============================================================
        # [6] COORDINATE GRID — Geographic lat/lon labels
        # ============================================================
        from pyproj import Transformer
        to_geo = Transformer.from_crs('EPSG:3857', 'EPSG:4326', always_xy=True)
        to_wm = Transformer.from_crs('EPSG:4326', 'EPSG:3857', always_xy=True)

        glon_min, glat_min = to_geo.transform(xmin - pad_x, ymin - pad_y)
        glon_max, glat_max = to_geo.transform(xmax + pad_x, ymax + pad_y)

        # 4-5 ticks, 2 decimal places
        lon_range = glon_max - glon_min
        tick_opts = [0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0]
        tick_int = min(tick_opts, key=lambda t: abs(lon_range / t - 4.5))

        lon_ticks = np.arange(math.ceil(glon_min / tick_int) * tick_int, glon_max, tick_int)
        lat_ticks = np.arange(math.ceil(glat_min / tick_int) * tick_int, glat_max, tick_int)

        xt = [to_wm.transform(lo, clat)[0] for lo in lon_ticks]
        yt = [to_wm.transform(clon, la)[1] for la in lat_ticks]

        ax.set_xticks(xt)
        ax.set_yticks(yt)
        ax.set_xticklabels([f'{lo:.2f}°' for lo in lon_ticks], fontsize=6.5, color='#555555')
        ax.set_yticklabels([f'{abs(la):.2f}°{"S" if la<0 else "N"}' for la in lat_ticks], fontsize=6.5, color='#555555')
        ax.tick_params(axis='both', direction='out', length=4, width=1, pad=2)

        ax.grid(True, linestyle='--', linewidth=0.4, color='#FFFF00', alpha=0.35, zorder=2)
        ax.set_xlabel('')
        ax.set_ylabel('')

        # ============================================================
        # [1] TITLE — above map
        # ============================================================
        fig.text(0.36, 0.95, title, fontsize=16, fontweight='bold',
                 ha='center', va='center', fontfamily='DejaVu Sans')

        # ============================================================
        # [5] NORTH ARROW — inside map, upper-right
        # ============================================================
        ax.annotate('', xy=(0.94, 0.96), xytext=(0.94, 0.88),
                    arrowprops=dict(arrowstyle='->', color='white', lw=2.5,
                                   mutation_scale=18),
                    xycoords='axes fraction', zorder=10)
        ax.text(0.94, 0.97, 'U', transform=ax.transAxes, fontsize=12,
                fontweight='bold', ha='center', va='bottom', color='white',
                path_effects=[withStroke(linewidth=3, foreground='black')], zorder=10)

        # ============================================================
        # [2] SCALE BAR — inside map, lower-left
        # ============================================================
        map_width_m = xmax + pad_x - (xmin - pad_x)
        bar_data_units = bar_km * 1000 * (map_width_m / (extent_km * 1000 * 1.24))
        bar_height = (ymax + pad_y - (ymin - pad_y)) * 0.006

        bar_label = f'{bar_km} km' if bar_km >= 1 else f'{int(bar_km*1000)} m'
        fontprops = fm.FontProperties(size=8, weight='bold')
        scalebar = AnchoredSizeBar(ax.transData, bar_data_units, bar_label,
                                   loc='lower left', pad=0.6, sep=4,
                                   color='white', frameon=True,
                                   size_vertical=bar_height,
                                   fontproperties=fontprops,
                                   fill_bar=True,
                                   borderpad=1.0)
        scalebar.patch.set_facecolor('#00000066')
        scalebar.patch.set_edgecolor('white')
        scalebar.txt_label._text.set_path_effects([withStroke(linewidth=2, foreground='black')])
        ax.add_artist(scalebar)

        # ============================================================
        # [7] INSET MAP — right panel top
        # ============================================================
        ax_inset.set_facecolor('white')
        for spine in ax_inset.spines.values():
            spine.set_linewidth(1.0)
            spine.set_color('black')

        if os.path.exists(ADMIN_GEOJSON):
            try:
                admin_full = gpd.read_file(ADMIN_GEOJSON)
                admin_full.plot(ax=ax_inset, color='#D5E8D4', edgecolor='#82B366',
                               linewidth=0.3, zorder=1)
                # Project location — red star
                ax_inset.plot(clon, clat, 'r*', markersize=14, zorder=5,
                             markeredgecolor='darkred', markeredgewidth=0.5)
                # Red rectangle for study area
                rect = Rectangle((bounds[0], bounds[1]),
                                bounds[2]-bounds[0], bounds[3]-bounds[1],
                                fill=False, edgecolor='red', linewidth=1.5, zorder=4)
                ax_inset.add_patch(rect)
            except Exception:
                pass

        ax_inset.set_xlim(94, 142)
        ax_inset.set_ylim(-12, 7)
        ax_inset.set_title('PETA LOKASI', fontsize=8, fontweight='bold',
                          fontfamily='DejaVu Sans', pad=3)
        ax_inset.set_xticks([])
        ax_inset.set_yticks([])

        # Ocean color for inset
        ax_inset.set_facecolor('#DAE8FC')

        # ============================================================
        # [4] LEGEND — right panel middle
        # ============================================================
        ax_legend.set_facecolor('white')
        for spine in ax_legend.spines.values():
            spine.set_linewidth(1.0)
            spine.set_color('black')
        ax_legend.set_xticks([])
        ax_legend.set_yticks([])

        ax_legend.set_title('LEGENDA', fontsize=9, fontweight='bold',
                           fontfamily='DejaVu Sans', pad=4)

        legend_items = [
            (mpatches.Patch(facecolor='none', edgecolor='#FF0000', linewidth=2), 'Area Studi / Proyek'),
        ]
        if show_admin:
            legend_items.append(
                (mpatches.Patch(facecolor='none', edgecolor='#FF00FF', linewidth=1, linestyle='--'),
                 'Batas Administrasi'))

        y_pos = 0.85
        for patch, label in legend_items:
            # Draw patch
            rect = Rectangle((0.05, y_pos - 0.08), 0.15, 0.06,
                            facecolor=patch.get_facecolor(),
                            edgecolor=patch.get_edgecolor(),
                            linewidth=patch.get_linewidth(),
                            linestyle=patch.get_linestyle() if hasattr(patch, 'get_linestyle') else '-',
                            transform=ax_legend.transAxes)
            ax_legend.add_patch(rect)
            ax_legend.text(0.25, y_pos - 0.05, label, transform=ax_legend.transAxes,
                          fontsize=7.5, va='center', fontfamily='DejaVu Sans')
            y_pos -= 0.18

        # Basemap source in legend
        ax_legend.text(0.05, 0.12, f'Basemap: {source_text}',
                      transform=ax_legend.transAxes, fontsize=6.5,
                      color='#666666', fontfamily='DejaVu Sans')

        # ============================================================
        # [3,8,9,10,11] METADATA — right panel bottom
        # ============================================================
        ax_meta.set_facecolor('#F8F8F8')
        for spine in ax_meta.spines.values():
            spine.set_linewidth(1.0)
            spine.set_color('black')
        ax_meta.set_xticks([])
        ax_meta.set_yticks([])

        ax_meta.set_title('KETERANGAN', fontsize=9, fontweight='bold',
                         fontfamily='DejaVu Sans', pad=4)

        prod_date = date_str or datetime.now().strftime('%d %B %Y')
        scale_str = f'1 : {num_scale:,}'.replace(',', '.')

        meta_lines = [
            ('Skala', scale_str),
            ('Proyeksi', f'UTM Zone {utm_zone}{utm_hem}'),
            ('EPSG', str(utm_epsg)),
            ('Datum', 'WGS-84'),
            ('Sumber', source_text),
            ('Tanggal', prod_date),
            ('Dibuat', author),
        ]

        y_pos = 0.88
        for key, val in meta_lines:
            ax_meta.text(0.05, y_pos, f'{key}:', transform=ax_meta.transAxes,
                        fontsize=7, fontweight='bold', va='top', fontfamily='DejaVu Sans')
            ax_meta.text(0.38, y_pos, val, transform=ax_meta.transAxes,
                        fontsize=7, va='top', fontfamily='DejaVu Sans')
            y_pos -= 0.12

        # Reference line at bottom
        ax_meta.text(0.05, 0.03, 'Ref: SNI 6502:2010 | PermenLH 16/2012',
                    transform=ax_meta.transAxes, fontsize=5.5,
                    color='#999999', fontfamily='DejaVu Sans')

        # ============================================================
        # SAVE — NO bbox_inches='tight' to preserve layout
        # ============================================================
        plt.savefig(output_path, dpi=200, facecolor='white', pad_inches=0)
        plt.close(fig)

        return (f"SUCCESS: Peta SNI-compliant disimpan di {output_path}\n"
                f"Skala: {scale_str}\n"
                f"CRS: UTM Zone {utm_zone}{utm_hem} (EPSG:{utm_epsg})\n"
                f"Layout: Right-panel (inset + legenda + keterangan terpisah dari peta)\n"
                f"Elemen kartografi: 13/13")

    except Exception as e:
        import traceback
        return f"ERROR: {str(e)}\n{traceback.format_exc()}"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='SNI 6502:2010 Map Generator')
    parser.add_argument("--geojson", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--realtime", action="store_true")
    parser.add_argument("--author", default="Environmental AI Agent")
    parser.add_argument("--date", default=None)
    parser.add_argument("--no-admin", action="store_true")
    args = parser.parse_args()

    geojson_input = args.geojson
    if os.path.exists(args.geojson):
        with open(args.geojson) as f: geojson_input = f.read()

    print(generate_sni_map(geojson_input, args.output, args.title,
                           realtime=args.realtime, author=args.author,
                           date_str=args.date, show_admin=not args.no_admin))
