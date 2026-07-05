#!/usr/bin/env python3
"""
Cartography Engine v4 — Bold, Clean, Professional
SNI 6502:2010 | PermenLH 16/2012
- Scale bar + North arrow OUTSIDE basemap (strip atas)
- Thick separator lines, bold fonts
- Logo instansi placeholder
- Approval blocks (Dibuat/Diperiksa/Disetujui)
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
from matplotlib.patheffects import withStroke
from shapely.geometry import shape
import contextily as cx
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, '..', '..', '..'))
ADMIN_GEOJSON = os.path.join(PROJECT_ROOT, 'resources', 'indonesia_admin.geojson')

# ── STYLE ────────────────────────────────────────────────────────
C = {
    'bdr':  '#2D3436', 'tx1':  '#2D3436', 'tx2':  '#4A5568',
    'tx3':  '#718096', 'bg':   '#FAFAF8', 'pan':  '#F7F7F3',
    'sep':  '#4A5568', 'grid': '#CBD5E0', 'ocean':'#DAE8FC',
    'land': '#D5E8D4', 'proj': '#E53E3E', 'adm':  '#9B59B6',
    'row1': '#EDF2F7', 'row2': '#FAFAF8',
}
FNT = 'DejaVu Sans'

# ── HELPERS ──────────────────────────────────────────────────────
def hav(la1,lo1,la2,lo2):
    R=6371; dl=math.radians(la2-la1); dn=math.radians(lo2-lo1)
    a=math.sin(dl/2)**2+math.cos(math.radians(la1))*math.cos(math.radians(la2))*math.sin(dn/2)**2
    return R*2*math.atan2(math.sqrt(a),math.sqrt(1-a))

def auto_utm(lon,lat):
    z=int((lon+180)/6)+1; return (32700+z if lat<0 else 32600+z),z,('S' if lat<0 else 'N')

def nscale(ek,fc=28):
    r=int(ek*1e5/fc)
    for s in [1000,2500,5000,10000,25000,50000,100000,250000,500000,1000000]:
        if s>=r*0.7: return s
    return 1000000

def nbar(ek):
    for s in [0.1,0.2,0.5,1,2,5,10,20,50,100,200,500]:
        if s>=ek/6: return s
    return 500

def dl_s2(lon,lat,bk,out):
    import ee, requests as rq
    try:
        ee.Initialize()
        pt=ee.Geometry.Point([lon,lat]); roi=pt.buffer(bk*1000).bounds()
        end=ee.Date(datetime.now().strftime('%Y-%m-%d')); st=end.advance(-30,'day')
        co=ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED').filterBounds(roi).filterDate(st,end).filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE',30))
        if co.size().getInfo()==0: return False
        url=co.median().select(['B4','B3','B2']).visualize(min=0,max=3000).clip(roi).getDownloadURL({'scale':10,'crs':'EPSG:3857','region':roi,'format':'GEO_TIFF'})
        r=rq.get(url,stream=True,timeout=120)
        if r.status_code==200:
            with open(out,'wb') as f:
                for ch in r.iter_content(4096): f.write(ch)
            return True
    except: pass
    return False

# ── SEPARATOR ────────────────────────────────────────────────────
def sep(fig, x0, y0, x1, y1, lw=2.5):
    fig.add_artist(plt.Line2D([x0,x1],[y0,y1], transform=fig.transFigure,
                              color=C['sep'], linewidth=lw, solid_capstyle='round'))

# ── MAIN ─────────────────────────────────────────────────────────
def generate_sni_map(geojson_str, output_path, title, realtime=False,
                     author="Environmental AI Agent", date_str=None, show_admin=True):
    try:
        data = json.loads(geojson_str)
        if data.get('type')=='FeatureCollection':
            for f in data.get('features',[]): f.setdefault('properties',{})
            gdf = gpd.GeoDataFrame.from_features(data['features'], crs='EPSG:4326')
        else:
            gdf = gpd.GeoDataFrame(geometry=[shape(data)], crs='EPSG:4326')

        bds = gdf.total_bounds
        clon,clat = (bds[0]+bds[2])/2, (bds[1]+bds[3])/2
        epsg,uz,uh = auto_utm(clon,clat)
        ek = hav(clat,bds[0],clat,bds[2])
        ns = nscale(ek); bk = nbar(ek)
        gw = gdf.to_crs(epsg=3857)
        xn,yn,xx,yx = gw.total_bounds
        px=max((xx-xn)*0.15,500); py=max((yx-yn)*0.15,500)

        # ═══════════════════════════════════════════════════════
        # FIGURE
        # ═══════════════════════════════════════════════════════
        fig = plt.figure(figsize=(16,11.5), dpi=150, facecolor=C['bg'])

        # ── DOUBLE NEATLINE ──
        for pad,lw in [(0.006,3.5),(0.013,1.5)]:
            fig.patches.append(FancyBboxPatch((pad,pad),1-2*pad,1-2*pad,
                boxstyle="square,pad=0", fc='none', ec=C['bdr'], lw=lw,
                transform=fig.transFigure, clip_on=False))

        # ── ROW 0: TITLE ──
        fig.text(0.50, 0.955, title.upper(), fontsize=18, fontweight='heavy',
                 ha='center', va='center', color=C['tx1'], fontfamily=FNT,
                 fontstretch='expanded')
        # Title underline
        sep(fig, 0.025, 0.935, 0.975, 0.935, lw=3.0)

        # ── ROW 1: SCALE BAR + NORTH ARROW — RIGHT SIDE (outside basemap) ──
        ax_strip = fig.add_axes([0.025, 0.885, 0.68, 0.045])
        ax_strip.set_xlim(0,1); ax_strip.set_ylim(0,1)
        ax_strip.set_facecolor(C['bg'])
        ax_strip.axis('off')

        # Scale bar — far right
        sb_x = 0.65; sb_w = 0.22; sb_y = 0.30; sb_h = 0.30
        segw = sb_w / 4
        for i in range(4):
            fc = C['bdr'] if i%2==0 else 'white'
            ax_strip.add_patch(Rectangle((sb_x+i*segw, sb_y), segw, sb_h,
                fc=fc, ec=C['bdr'], lw=1.2, clip_on=False))
        # Scale bar labels below bar
        for i, lbl in enumerate(['0', f'{bk/2:.0f}' if bk>=2 else f'{bk/2}', f'{bk:.0f} km' if bk>=1 else f'{int(bk*1000)} m']):
            ax_strip.text(sb_x + i*sb_w/2, sb_y-0.12, lbl, fontsize=8,
                         fontweight='bold', ha='center', va='top', color=C['tx1'], fontfamily=FNT)

        # North arrow — far right, CONTAINED within strip
        na_x = 0.96; na_top = 0.80; na_bot = 0.15
        ax_strip.add_patch(Polygon([(na_x,na_top),(na_x-0.018,na_bot),(na_x,na_bot*1.5)],
            closed=True, fc='white', ec=C['bdr'], lw=1.5, clip_on=False))
        ax_strip.add_patch(Polygon([(na_x,na_top),(na_x+0.018,na_bot),(na_x,na_bot*1.5)],
            closed=True, fc=C['bdr'], ec=C['bdr'], lw=1.5, clip_on=False))
        ax_strip.text(na_x, na_top+0.05, 'U', fontsize=11, fontweight='heavy',
                      ha='center', va='bottom', color=C['tx1'], fontfamily=FNT)

        # Garis vertikal kiri penutup area scale/north
        ax_strip.plot([sb_x - 0.03, sb_x - 0.03], [0, 1], color=C['bdr'], lw=2.0, clip_on=False)

        # Line below strip
        sep(fig, 0.025, 0.882, 0.975, 0.882, lw=2.5)

        # ── VERTICAL SEPARATOR (map | right panel) ──
        rp_x = 0.71  # right panel starts here
        sep(fig, rp_x, 0.025, rp_x, 0.882, lw=2.5)

        # ── MAIN MAP ──
        ax = fig.add_axes([0.025, 0.025, rp_x-0.025, 0.855])
        ax.set_xlim(xn-px, xx+px); ax.set_ylim(yn-py, yx+py)

        # Basemap
        src_txt = "Esri World Imagery"
        if realtime:
            tf="/tmp/_cv4_s2.tif"
            if os.path.exists(tf): os.remove(tf)
            ek2=hav(bds[1],clon,bds[3],clon)
            if dl_s2(clon,clat,max(ek,ek2)+2,tf):
                import rasterio; from rasterio.plot import show as rs
                with rasterio.open(tf) as src: rs(src,ax=ax,zorder=1)
                src_txt="Sentinel-2 L2A"
            else:
                cx.add_basemap(ax, crs=gw.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)
        else:
            cx.add_basemap(ax, crs=gw.crs.to_string(), source=cx.providers.Esri.WorldImagery, zorder=1)

        # Admin boundaries
        if show_admin and os.path.exists(ADMIN_GEOJSON):
            try:
                ad=gpd.read_file(ADMIN_GEOJSON).to_crs(epsg=3857)
                cl=ad.cx[xn-px*3:xx+px*3, yn-py*3:yx+py*3]
                if len(cl)>0:
                    cl[cl['level']==1].plot(ax=ax, color='none', ec=C['adm'],
                        lw=1.5, ls='--', zorder=3, alpha=0.85)
            except: pass

        # Project polygon
        gw.plot(ax=ax, color='none', ec=C['proj'], lw=3.0, zorder=4)

        # Map frame
        for sp in ax.spines.values(): sp.set_lw(2.0); sp.set_color(C['bdr'])

        # Coordinate grid
        from pyproj import Transformer
        tg=Transformer.from_crs('EPSG:3857','EPSG:4326',always_xy=True)
        tw=Transformer.from_crs('EPSG:4326','EPSG:3857',always_xy=True)
        gln,gla=tg.transform(xn-px,yn-py); glx,glx2=tg.transform(xx+px,yx+py)
        lr=glx-gln
        for ti in [0.005,0.01,0.02,0.05,0.1,0.2,0.5,1.0,2.0]:
            if lr/ti<=5.5: tki=ti; break
        else: tki=2.0
        lons=np.arange(math.ceil(gln/tki)*tki,glx,tki)
        lats=np.arange(math.ceil(gla/tki)*tki,glx2,tki)
        ax.set_xticks([tw.transform(lo,clat)[0] for lo in lons])
        ax.set_yticks([tw.transform(clon,la)[1] for la in lats])
        ax.set_xticklabels([f'{lo:.2f}°' for lo in lons], fontsize=7.5, fontweight='bold', color=C['tx2'], fontfamily=FNT)
        ax.set_yticklabels([f'{abs(la):.2f}°{"S" if la<0 else "N"}' for la in lats], fontsize=7.5, fontweight='bold', color=C['tx2'], fontfamily=FNT)
        ax.tick_params(direction='out', length=5, width=1.2, pad=5, colors=C['tx2'])
        ax.grid(True, ls='--', lw=0.4, color=C['grid'], alpha=0.35, zorder=2)
        ax.set_xlabel(''); ax.set_ylabel('')

        # ═══════════════════════════════════════════════════════
        # RIGHT PANEL — 4 sections
        # ═══════════════════════════════════════════════════════
        rp_w = 0.975 - rp_x  # right panel width
        rp_bot = 0.025
        rp_top = 0.882

        # Heights: inset 22%, legend 15%, gap 3%, metadata 40%, logo 20%
        h_inset = (rp_top - rp_bot) * 0.22
        h_legend = (rp_top - rp_bot) * 0.15
        h_gap = (rp_top - rp_bot) * 0.03  # gap between legend and metadata
        h_meta = (rp_top - rp_bot) * 0.38
        h_logo = (rp_top - rp_bot) * 0.20

        y_inset = rp_top - h_inset
        y_legend = y_inset - h_legend
        y_meta = y_legend - h_gap - h_meta  # extra gap
        y_logo = rp_bot

        pad_r = 0.008  # padding inside right panel

        # ── INSET MAP ──
        ax_ins = fig.add_axes([rp_x+pad_r, y_inset+pad_r, rp_w-2*pad_r, h_inset-2*pad_r])
        ax_ins.set_facecolor(C['ocean'])
        for sp in ax_ins.spines.values(): sp.set_lw(1.5); sp.set_color(C['bdr'])
        if os.path.exists(ADMIN_GEOJSON):
            try:
                af=gpd.read_file(ADMIN_GEOJSON)
                af.plot(ax=ax_ins, color=C['land'], ec='#6B8E23', lw=0.3)
                ax_ins.add_patch(Rectangle((bds[0],bds[1]),bds[2]-bds[0],bds[3]-bds[1],
                    fill=True, fc='#E53E3E55', ec='#E53E3E', lw=2, zorder=5))
                ax_ins.plot(clon,clat,'o',color='#E53E3E',ms=6,mec='white',mew=1,zorder=6)
            except: pass
        ax_ins.set_xlim(94,142); ax_ins.set_ylim(-12,7)
        ax_ins.set_xticks([]); ax_ins.set_yticks([])
        ax_ins.text(0.5,1.06,'PETA LOKASI', transform=ax_ins.transAxes,
                    fontsize=10, fontweight='heavy', ha='center', color=C['tx1'], fontfamily=FNT)

        # Separator below inset
        sep(fig, rp_x, y_inset, 0.975, y_inset, lw=2.0)

        # ── LEGEND ──
        ax_lg = fig.add_axes([rp_x+pad_r, y_legend+pad_r, rp_w-2*pad_r, h_legend-2*pad_r])
        ax_lg.set_facecolor(C['bg']); ax_lg.axis('off')
        ax_lg.text(0.5,0.95,'LEGENDA', transform=ax_lg.transAxes,
                   fontsize=10, fontweight='heavy', ha='center', color=C['tx1'], fontfamily=FNT)

        items = [
            ('rect', C['proj'], 2.5, '-', 'Batas Area Studi'),
        ]
        if show_admin:
            items.append(('line', C['adm'], 2.0, '--', 'Batas Administrasi'))
        items.append(('dot', '#3498DB', 8, '', 'Titik Sampling'))

        yp = 0.78
        for typ, col, sz, ls, lbl in items:
            if typ == 'rect':
                ax_lg.add_patch(Rectangle((0.04,yp-0.12),0.14,0.15,
                    fc='none', ec=col, lw=sz, transform=ax_lg.transAxes, clip_on=False))
            elif typ == 'line':
                ax_lg.plot([0.04,0.18],[yp-0.04,yp-0.04], color=col, lw=sz,
                          ls=ls, transform=ax_lg.transAxes, clip_on=False)
            elif typ == 'dot':
                ax_lg.plot(0.11, yp-0.04, 'o', color=col, ms=sz, mec='white', mew=0.8,
                          transform=ax_lg.transAxes, clip_on=False)
            ax_lg.text(0.24, yp-0.04, lbl, transform=ax_lg.transAxes,
                      fontsize=8.5, fontweight='bold', va='center', color=C['tx1'], fontfamily=FNT)
            yp -= 0.32

        # Separator below legend (with gap before metadata)
        sep(fig, rp_x, y_meta + h_meta, 0.975, y_meta + h_meta, lw=2.0)

        # ── METADATA TABLE ──
        ax_mt = fig.add_axes([rp_x+pad_r, y_meta+pad_r, rp_w-2*pad_r, h_meta-2*pad_r])
        ax_mt.set_facecolor(C['bg'])
        for sp in ax_mt.spines.values(): sp.set_lw(1.2); sp.set_color(C['bdr'])
        ax_mt.set_xlim(0,1); ax_mt.set_ylim(0,1); ax_mt.set_xticks([]); ax_mt.set_yticks([])
        ax_mt.text(0.5,0.97,'INFORMASI PETA', transform=ax_mt.transAxes,
                   fontsize=10, fontweight='heavy', ha='center', color=C['tx1'], fontfamily=FNT)

        pd = date_str or datetime.now().strftime('%d %B %Y')
        ss = f'1 : {ns:,}'.replace(',','.')
        rows = [
            ('Skala', ss), ('Proyeksi', f'UTM Zone {uz}{uh}'),
            ('Datum', 'WGS-84'), ('EPSG', str(epsg)),
            ('Sumber', src_txt), ('Tanggal', pd), ('Dibuat', author),
            ('Diperiksa', '________________'), ('Disetujui', '________________'),
        ]
        n=len(rows); rh=1.0/n
        for i,(k,v) in enumerate(rows):
            yy=1.0-(i+1)*rh
            bg = C['row1'] if i%2==0 else C['row2']
            ax_mt.add_patch(Rectangle((0,yy),1,rh, fc=bg, ec=C['grid'], lw=0.4, clip_on=False))
            ax_mt.plot([0.40,0.40],[yy,yy+rh], color=C['grid'], lw=0.5, clip_on=False)
            ax_mt.text(0.03, yy+rh/2, k, fontsize=8, fontweight='heavy',
                      va='center', color=C['tx1'], fontfamily=FNT)
            ax_mt.text(0.43, yy+rh/2, v, fontsize=8, fontweight='bold',
                      va='center', color=C['tx2'], fontfamily=FNT)

        # Separator below metadata
        sep(fig, rp_x, y_meta, 0.975, y_meta, lw=2.0)

        # ── LOGO PLACEHOLDER ──
        ax_lo = fig.add_axes([rp_x+pad_r, y_logo+pad_r, rp_w-2*pad_r, h_logo-2*pad_r])
        ax_lo.set_facecolor(C['pan'])
        for sp in ax_lo.spines.values(): sp.set_lw(1.2); sp.set_color(C['bdr'])
        ax_lo.set_xticks([]); ax_lo.set_yticks([])
        ax_lo.text(0.5, 0.65, 'LOGO', transform=ax_lo.transAxes, fontsize=10,
                  fontweight='heavy', ha='center', va='center', color=C['tx3'], fontfamily=FNT)
        ax_lo.text(0.5, 0.35, 'INSTANSI', transform=ax_lo.transAxes, fontsize=10,
                  fontweight='heavy', ha='center', va='center', color=C['tx3'], fontfamily=FNT)

        # Reference — inside logo box bottom
        ax_lo.text(0.5, 0.08, 'SNI 6502:2010 | PermenLH 16/2012',
                  transform=ax_lo.transAxes, fontsize=5.5, ha='center',
                  color=C['tx3'], fontfamily=FNT, fontstyle='italic')

        # ── SAVE ──
        plt.savefig(output_path, dpi=150, facecolor=C['bg'], pad_inches=0)
        plt.close(fig)

        return (f"SUCCESS: Peta SNI v4 disimpan di {output_path}\n"
                f"Skala: {ss} | CRS: UTM Zone {uz}{uh} (EPSG:{epsg})\n"
                f"Layout: Bold fonts, thick separators, scale/north outside basemap\n"
                f"Elemen kartografi: 13/13 + logo placeholder")

    except Exception as e:
        import traceback
        return f"ERROR: {e}\n{traceback.format_exc()}"

if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("--geojson",required=True); p.add_argument("--output",required=True)
    p.add_argument("--title",required=True); p.add_argument("--realtime",action="store_true")
    p.add_argument("--author",default="Environmental AI Agent")
    p.add_argument("--date",default=None); p.add_argument("--no-admin",action="store_true")
    a=p.parse_args()
    gj=a.geojson
    if os.path.exists(a.geojson):
        with open(a.geojson) as f: gj=f.read()
    print(generate_sni_map(gj,a.output,a.title,realtime=a.realtime,
                           author=a.author,date_str=a.date,show_admin=not a.no_admin))
