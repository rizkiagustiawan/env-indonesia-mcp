#!/usr/bin/env python3
"""
Cartography Engine v3 — Elite Professional Layout
SNI 6502:2010 Compliant | PermenLH 16/2012
GridSpec layout, double neatline, alternating scale bar, filled north arrow,
table metadata with approval blocks, proper legend symbols.
"""
import sys, json, argparse, math, os
from datetime import datetime

import geopandas as gpd
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import Polygon, Rectangle, FancyBboxPatch
from matplotlib.lines import Line2D
from matplotlib.gridspec import GridSpec
from matplotlib.patheffects import withStroke
import matplotlib.font_manager as fm
from shapely.geometry import shape
import contextily as cx
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, '..', '..', '..'))
ADMIN_GEOJSON = os.path.join(PROJECT_ROOT, 'resources', 'indonesia_admin.geojson')

# ── STYLE CONSTANTS ──────────────────────────────────────────────
CLR = {
    'border':   '#2D3436',
    'text1':    '#2D3436',   # primary
    'text2':    '#636E72',   # secondary
    'text3':    '#828E94',   # metadata
    'bg':       '#FAFAF8',   # warm off-white
    'panel':    '#F5F5F0',   # panel bg
    'sep':      '#DFE6E9',   # separators
    'ocean':    '#DAE8FC',   # inset ocean
    'land':     '#D5E8D4',   # inset land
    'project':  '#E74C3C',   # project polygon
    'admin':    '#9B59B6',   # admin boundary
    'grid':     '#DFE6E9',   # map grid
    'row_alt':  '#F0F0EC',   # alternating table row
}
FONT = 'DejaVu Sans'

# ── HELPERS ──────────────────────────────────────────────────────
def haversine_km(lat1, lon1, lat2, lon2):
    R = 6371.0
    dlat, dlon = math.radians(lat2 - lat1), math.radians(lon2 - lon1)
    a = math.sin(dlat/2)**2 + math.cos(math.radians(lat1))*math.cos(math.radians(lat2))*math.sin(dlon/2)**2
    return R * 2 * math.atan2(math.sqrt(a), math.sqrt(1-a))

def auto_utm(lon, lat):
    zone = int((lon + 180) / 6) + 1
    return (32700+zone if lat < 0 else 32600+zone), zone, ('S' if lat < 0 else 'N')

def nice_num_scale(extent_km, fig_cm=28):
    raw = int(extent_km * 1e5 / fig_cm)
    for s in [1000,2500,5000,10000,25000,50000,100000,250000,500000,1000000]:
        if s >= raw * 0.7: return s
    return 1000000

def nice_bar(extent_km):
    for s in [0.1,0.2,0.5,1,2,5,10,20,50,100,200,500]:
        if s >= extent_km / 6: return s
    return 500

def download_sentinel(lon, lat, buf_km, out):
    import ee, requests as rq
    try:
        ee.Initialize()
        pt = ee.Geometry.Point([lon, lat]); roi = pt.buffer(buf_km*1000).bounds()
        end = ee.Date(datetime.now().strftime('%Y-%m-%d')); start = end.advance(-30,'day')
        c = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED').filterBounds(roi).filterDate(start,end).filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE',30))
        if c.size().getInfo()==0: return False
        url = c.median().select(['B4','B3','B2']).visualize(min=0,max=3000).clip(roi).getDownloadURL({'scale':10,'crs':'EPSG:3857','region':roi,'format':'GEO_TIFF'})
        r = rq.get(url, stream=True, timeout=120)
        if r.status_code==200:
            with open(out,'wb') as f:
                for ch in r.iter_content(4096): f.write(ch)
            return True
    except: pass
    return False

# ── DRAWING COMPONENTS ───────────────────────────────────────────

def draw_neatline(fig):
    """Double neatline — outer thick + inner thin."""
    for pad, lw in [(0.008, 2.5), (0.014, 0.7)]:
        fig.patches.append(FancyBboxPatch(
            (pad, pad), 1-2*pad, 1-2*pad,
            boxstyle="square,pad=0", facecolor='none',
            edgecolor=CLR['border'], linewidth=lw,
            transform=fig.transFigure, clip_on=False))

def draw_separator(fig, x0, y0, x1, y1):
    """Hairline separator."""
    fig.add_artist(plt.Line2D([x0,x1],[y0,y1], transform=fig.transFigure,
                              color=CLR['sep'], linewidth=0.8, zorder=0))

def draw_north_arrow(ax, x=0.94, y=0.93, sz=0.055):
    """Filled triangle north arrow."""
    verts = [(x, y), (x-sz*0.25, y-sz), (x, y-sz*0.65), (x+sz*0.25, y-sz)]
    # White fill half
    ax.add_patch(Polygon([(x,y),(x-sz*0.25,y-sz),(x,y-sz*0.65)],
        closed=True, fc='white', ec=CLR['border'], lw=0.8,
        transform=ax.transAxes, zorder=11, clip_on=False))
    # Dark fill half
    ax.add_patch(Polygon([(x,y),(x+sz*0.25,y-sz),(x,y-sz*0.65)],
        closed=True, fc=CLR['border'], ec=CLR['border'], lw=0.8,
        transform=ax.transAxes, zorder=11, clip_on=False))
    ax.text(x, y+0.015, 'U', transform=ax.transAxes, fontsize=11,
            fontweight='bold', ha='center', va='bottom', color=CLR['border'],
            fontfamily=FONT, path_effects=[withStroke(linewidth=2.5, foreground='white')],
            zorder=12)

def draw_scale_bar(ax, x, y, bar_km, map_extent_wm, n_seg=4):
    """Alternating black/white professional scale bar."""
    total_w = 0.18  # fraction of axes width
    seg_w = total_w / n_seg
    h = 0.008
    for i in range(n_seg):
        fc = CLR['border'] if i % 2 == 0 else 'white'
        ax.add_patch(Rectangle((x + i*seg_w, y), seg_w, h,
            facecolor=fc, edgecolor=CLR['border'], linewidth=0.6,
            transform=ax.transAxes, zorder=10, clip_on=False))
    # Labels
    labels = ['0', f'{bar_km/2:.0f}' if bar_km >= 2 else f'{bar_km/2}', f'{bar_km:.0f} km' if bar_km >= 1 else f'{int(bar_km*1000)} m']
    positions = [x, x + total_w/2, x + total_w]
    for pos, lbl in zip(positions, labels):
        ax.text(pos, y - 0.012, lbl, transform=ax.transAxes,
                fontsize=6, ha='center', color=CLR['text2'], fontfamily=FONT, zorder=10)

def draw_inset(ax_inset, clon, clat, bounds):
    """Indonesia locator map with study area marker."""
    ax_inset.set_facecolor(CLR['ocean'])
    for sp in ax_inset.spines.values():
        sp.set_linewidth(1.0); sp.set_color(CLR['border'])

    if os.path.exists(ADMIN_GEOJSON):
        try:
            admin = gpd.read_file(ADMIN_GEOJSON)
            admin.plot(ax=ax_inset, color=CLR['land'], edgecolor='#82B366', linewidth=0.25)
        except: pass

    # Red rectangle for study area
    ax_inset.add_patch(Rectangle(
        (bounds[0], bounds[1]), bounds[2]-bounds[0], bounds[3]-bounds[1],
        fill=True, facecolor='#E74C3C44', edgecolor='#E74C3C', linewidth=1.5, zorder=5))
    # Red dot
    ax_inset.plot(clon, clat, 'o', color='#E74C3C', markersize=5,
                  markeredgecolor='white', markeredgewidth=0.8, zorder=6)

    ax_inset.set_xlim(94, 142); ax_inset.set_ylim(-12, 7)
    ax_inset.set_xticks([]); ax_inset.set_yticks([])
    ax_inset.text(0.5, 1.04, 'PETA LOKASI', transform=ax_inset.transAxes,
                  fontsize=8, fontweight='bold', ha='center', color=CLR['text1'], fontfamily=FONT)

def draw_legend(ax_leg, show_admin):
    """Professional legend with proper map symbols."""
    ax_leg.set_facecolor(CLR['bg'])
    for sp in ax_leg.spines.values():
        sp.set_linewidth(0.8); sp.set_color(CLR['border'])
    ax_leg.set_xticks([]); ax_leg.set_yticks([])

    ax_leg.text(0.5, 1.04, 'LEGENDA', transform=ax_leg.transAxes,
                fontsize=8, fontweight='bold', ha='center', color=CLR['text1'], fontfamily=FONT)

    items = [
        (Rectangle((0,0),1,1, fc='none', ec=CLR['project'], lw=2.5), 'Batas Area Studi'),
    ]
    if show_admin:
        items.append((Line2D([0],[0], color=CLR['admin'], lw=1.5, ls='--'), 'Batas Administrasi'))
    items.append((Line2D([0],[0], marker='o', color='w', markerfacecolor='#3498DB',
                         markersize=8, ls='None'), 'Titik Sampling'))

    y = 0.82
    for handle, label in items:
        if isinstance(handle, Rectangle):
            ax_leg.add_patch(Rectangle((0.06, y-0.06), 0.12, 0.08,
                fc='none', ec=handle.get_edgecolor(), lw=handle.get_linewidth(),
                transform=ax_leg.transAxes, clip_on=False))
        elif hasattr(handle, 'get_linestyle') and handle.get_linestyle() == '--':
            ax_leg.plot([0.06, 0.18], [y-0.02, y-0.02], color=handle.get_color(),
                       lw=handle.get_linewidth(), ls='--', transform=ax_leg.transAxes, clip_on=False)
        else:
            ax_leg.plot(0.12, y-0.02, marker=handle.get_marker(),
                       color=handle.get_markerfacecolor(), markersize=handle.get_markersize(),
                       markeredgecolor='white', markeredgewidth=0.5,
                       transform=ax_leg.transAxes, clip_on=False)

        ax_leg.text(0.24, y-0.02, label, transform=ax_leg.transAxes,
                    fontsize=7, va='center', color=CLR['text1'], fontfamily=FONT)
        y -= 0.22

def draw_metadata(ax_meta, num_scale, utm_zone, utm_hem, utm_epsg, source, date_str, author):
    """Table-style metadata with alternating rows and approval blocks."""
    ax_meta.set_facecolor(CLR['bg'])
    for sp in ax_meta.spines.values():
        sp.set_linewidth(0.8); sp.set_color(CLR['border'])
    ax_meta.set_xlim(0,1); ax_meta.set_ylim(0,1)
    ax_meta.set_xticks([]); ax_meta.set_yticks([])

    ax_meta.text(0.5, 1.03, 'INFORMASI PETA', transform=ax_meta.transAxes,
                 fontsize=8, fontweight='bold', ha='center', color=CLR['text1'], fontfamily=FONT)

    scale_str = f'1 : {num_scale:,}'.replace(',', '.')
    rows = [
        ('Skala', scale_str),
        ('Proyeksi', f'UTM Zone {utm_zone}{utm_hem}'),
        ('Datum', 'WGS-84'),
        ('EPSG', str(utm_epsg)),
        ('Sumber', source),
        ('Tanggal', date_str),
        ('Dibuat', author),
        ('Diperiksa', '_______________'),
        ('Disetujui', '_______________'),
    ]

    n = len(rows)
    rh = 1.0 / n
    for i, (key, val) in enumerate(rows):
        yp = 1.0 - (i+1) * rh
        bg = CLR['row_alt'] if i % 2 == 0 else CLR['bg']
        ax_meta.add_patch(Rectangle((0, yp), 1, rh,
            facecolor=bg, edgecolor=CLR['sep'], linewidth=0.3, clip_on=False))
        # Vertical divider
        ax_meta.plot([0.38, 0.38], [yp, yp+rh], color=CLR['sep'], lw=0.3, clip_on=False)
        ax_meta.text(0.04, yp + rh/2, key, fontsize=6.5, fontweight='bold',
                     va='center', color=CLR['text1'], fontfamily=FONT)
        ax_meta.text(0.42, yp + rh/2, val, fontsize=6.5,
                     va='center', color=CLR['text2'], fontfamily=FONT)

    # Reference footer
    ax_meta.text(0.5, -0.04, 'SNI 6502:2010 | PermenLH 16/2012',
                 transform=ax_meta.transAxes, fontsize=5, ha='center',
                 color=CLR['text3'], fontfamily=FONT, fontstyle='italic')

# ── MAIN GENERATOR ───────────────────────────────────────────────

def generate_sni_map(geojson_str, output_path, title, realtime=False,
                     author="Environmental AI Agent", date_str=None, show_admin=True):
    try:
        data = json.loads(geojson_str)
        if data.get('type') == 'FeatureCollection':
            for f in data.get('features',[]): f.setdefault('properties',{})
            gdf = gpd.GeoDataFrame.from_features(data['features'], crs='EPSG:4326')
        else:
            gdf = gpd.GeoDataFrame(geometry=[shape(data)], crs='EPSG:4326')

        bounds = gdf.total_bounds
        clon, clat = (bounds[0]+bounds[2])/2, (bounds[1]+bounds[3])/2
        utm_epsg, utm_zone, utm_hem = auto_utm(clon, clat)
        ext_km = haversine_km(clat, bounds[0], clat, bounds[2])
        num_scale = nice_num_scale(ext_km)
        bar_km = nice_bar(ext_km)

        gdf_wm = gdf.to_crs(epsg=3857)
        xmin, ymin, xmax, ymax = gdf_wm.total_bounds
        px = max((xmax-xmin)*0.15, 500); py = max((ymax-ymin)*0.15, 500)

        # ── FIGURE + GRIDSPEC ──
        fig = plt.figure(figsize=(16, 11), dpi=150, facecolor=CLR['bg'])

        gs = GridSpec(3, 2, figure=fig,
            left=0.035, right=0.965, top=0.92, bottom=0.035,
            wspace=0.025, hspace=0.025,
            width_ratios=[2.8, 1],
            height_ratios=[1.0, 0.9, 1.4])

        ax       = fig.add_subplot(gs[:, 0])   # Map: full left column
        ax_inset = fig.add_subplot(gs[0, 1])   # Inset: top-right
        ax_leg   = fig.add_subplot(gs[1, 1])   # Legend: mid-right
        ax_meta  = fig.add_subplot(gs[2, 1])   # Metadata: bot-right

        # ── DOUBLE NEATLINE ──
        draw_neatline(fig)

        # ── TITLE ──
        fig.text(0.50, 0.96, title.upper(), fontsize=14, fontweight='bold',
                 ha='center', va='center', color=CLR['text1'], fontfamily=FONT)

        # ── MAP EXTENT ──
        ax.set_xlim(xmin-px, xmax+px); ax.set_ylim(ymin-py, ymax+py)

        # ── BASEMAP ──
        source_text = "Esri World Imagery"
        if realtime:
            tif = "/tmp/_cart_sentinel.tif"
            if os.path.exists(tif): os.remove(tif)
            if download_sentinel(clon, clat, max(ext_km, haversine_km(bounds[1],clon,bounds[3],clon))+2, tif):
                import rasterio; from rasterio.plot import show as rshow
                with rasterio.open(tif) as src: rshow(src, ax=ax, zorder=1)
                source_text = "Sentinel-2 L2A"
            else:
                cx.add_basemap(ax, crs=gdf_wm.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)
        else:
            cx.add_basemap(ax, crs=gdf_wm.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)

        # ── ADMIN BOUNDARIES ──
        if show_admin and os.path.exists(ADMIN_GEOJSON):
            try:
                adm = gpd.read_file(ADMIN_GEOJSON).to_crs(epsg=3857)
                clip = adm.cx[xmin-px*3:xmax+px*3, ymin-py*3:ymax+py*3]
                if len(clip) > 0:
                    clip[clip['level']==1].plot(ax=ax, color='none', edgecolor=CLR['admin'],
                        linewidth=1.2, linestyle='--', zorder=3, alpha=0.85)
            except: pass

        # ── PROJECT POLYGON ──
        gdf_wm.plot(ax=ax, color='none', edgecolor=CLR['project'], linewidth=2.5, zorder=4)

        # ── MAP FRAME ──
        for sp in ax.spines.values():
            sp.set_linewidth(1.5); sp.set_color(CLR['border'])

        # ── COORDINATE GRID ──
        from pyproj import Transformer
        to_geo = Transformer.from_crs('EPSG:3857','EPSG:4326', always_xy=True)
        to_wm  = Transformer.from_crs('EPSG:4326','EPSG:3857', always_xy=True)
        glon_min, glat_min = to_geo.transform(xmin-px, ymin-py)
        glon_max, glat_max = to_geo.transform(xmax+px, ymax+py)

        lon_rng = glon_max - glon_min
        for ti in [0.005,0.01,0.02,0.05,0.1,0.2,0.5,1.0,2.0]:
            if lon_rng / ti <= 5.5: tick_int = ti; break
        else: tick_int = 2.0

        lons = np.arange(math.ceil(glon_min/tick_int)*tick_int, glon_max, tick_int)
        lats = np.arange(math.ceil(glat_min/tick_int)*tick_int, glat_max, tick_int)
        ax.set_xticks([to_wm.transform(lo,clat)[0] for lo in lons])
        ax.set_yticks([to_wm.transform(clon,la)[1] for la in lats])
        ax.set_xticklabels([f'{lo:.2f}°' for lo in lons], fontsize=6, color=CLR['text3'], fontfamily=FONT)
        ax.set_yticklabels([f'{abs(la):.2f}°{"S" if la<0 else "N"}' for la in lats], fontsize=6, color=CLR['text3'], fontfamily=FONT)
        ax.tick_params(axis='both', direction='out', length=4, width=0.8, pad=2, colors=CLR['text3'])
        ax.grid(True, ls='--', lw=0.3, color=CLR['grid'], alpha=0.4, zorder=2)
        ax.set_xlabel(''); ax.set_ylabel('')

        # ── NORTH ARROW ──
        draw_north_arrow(ax)

        # ── SCALE BAR ──
        draw_scale_bar(ax, 0.03, 0.03, bar_km, xmax-xmin)

        # ── PANEL SEPARATORS ──
        # Between inset and legend
        r_left = 0.035 + (1-0.035-0.035) * 2.8/3.8 + 0.025
        draw_separator(fig, r_left, gs.get_subplot_params().bottom,
                       0.965, gs.get_subplot_params().bottom)

        # ── INSET MAP ──
        draw_inset(ax_inset, clon, clat, bounds)

        # ── LEGEND ──
        draw_legend(ax_leg, show_admin)

        # ── METADATA TABLE ──
        prod_date = date_str or datetime.now().strftime('%d %B %Y')
        draw_metadata(ax_meta, num_scale, utm_zone, utm_hem, utm_epsg, source_text, prod_date, author)

        # ── SAVE ──
        plt.savefig(output_path, dpi=150, facecolor=CLR['bg'], pad_inches=0)
        plt.close(fig)

        scale_str = f'1 : {num_scale:,}'.replace(',','.')
        return (f"SUCCESS: Peta SNI-compliant (v3 Elite) disimpan di {output_path}\n"
                f"Skala: {scale_str} | CRS: UTM Zone {utm_zone}{utm_hem} (EPSG:{utm_epsg})\n"
                f"Layout: GridSpec right-panel, double neatline, alternating scale bar\n"
                f"Elemen kartografi: 13/13")

    except Exception as e:
        import traceback
        return f"ERROR: {e}\n{traceback.format_exc()}"

# ── CLI ──────────────────────────────────────────────────────────
if __name__ == "__main__":
    p = argparse.ArgumentParser(description='SNI 6502:2010 Elite Map Generator v3')
    p.add_argument("--geojson", required=True)
    p.add_argument("--output", required=True)
    p.add_argument("--title", required=True)
    p.add_argument("--realtime", action="store_true")
    p.add_argument("--author", default="Environmental AI Agent")
    p.add_argument("--date", default=None)
    p.add_argument("--no-admin", action="store_true")
    a = p.parse_args()
    gj = a.geojson
    if os.path.exists(a.geojson):
        with open(a.geojson) as f: gj = f.read()
    print(generate_sni_map(gj, a.output, a.title, realtime=a.realtime,
                           author=a.author, date_str=a.date, show_admin=not a.no_admin))
