#!/usr/bin/env python3
"""Spatial Statistics Engine — Moran's I, LISA, Semivariogram
Uses PySAL (Python Spatial Analysis Library) for spatial autocorrelation analysis.
Ref: Anselin 1995 (Local Moran's I), Getis & Ord 1992 (Gi*)
"""
import sys, os, json, math
import numpy as np

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

try:
    import ee
    ee.Initialize()
except:
    pass

import requests


def make_roi(lat, lon, buffer_km):
    point = ee.Geometry.Point([lon, lat])
    return point.buffer(buffer_km * 1000).bounds()


def morans_i_analysis(lat, lon, buffer_km, index_type, output_path):
    """Compute Global and Local Moran's I for a spectral index.

    1. Compute index (NDVI/NDWI/etc) from S2 via GEE
    2. Sample to regular grid points
    3. Compute Global Moran's I (spatial autocorrelation)
    4. Compute Local Moran's I / LISA (cluster/outlier detection)
    5. Generate LISA cluster map

    Ref: Anselin 1995, PySAL (Rey & Anselin 2010)
    """
    roi = make_roi(lat, lon, buffer_km)

    # Get spectral index from GEE
    cs = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED') \
        .filterDate('2023-01-01', '2023-12-31').filterBounds(roi)
    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate('2023-01-01', '2023-12-31').filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30)) \
        .linkCollection(cs, ['cs_cdf']) \
        .map(lambda img: img.updateMask(img.select('cs_cdf').gte(0.60))) \
        .median().clip(roi)

    index_map = {
        'ndvi': ('B8', 'B4'),
        'ndwi': ('B3', 'B8'),
        'ndbi': ('B11', 'B8'),
        'mndwi': ('B3', 'B11'),
    }

    if index_type.lower() not in index_map:
        print(f"ERROR: Indeks '{index_type}' tidak dikenal. Gunakan: {list(index_map.keys())}")
        return

    b1, b2 = index_map[index_type.lower()]
    index_img = s2.normalizedDifference([b1, b2]).rename('index')

    # Sample to grid points (reduce resolution for spatial stats)
    sample = index_img.sample(region=roi, scale=100, numPixels=2000, seed=42, geometries=True)
    sample_data = sample.getInfo()

    if not sample_data.get('features'):
        print("ERROR: Tidak ada data sampling.")
        return

    # Extract coordinates and values
    coords = []
    values = []
    for feat in sample_data['features']:
        geom = feat['geometry']['coordinates']
        val = feat['properties'].get('index')
        if val is not None and not math.isnan(val):
            coords.append(geom)
            values.append(val)

    coords = np.array(coords)
    values = np.array(values)
    n = len(values)

    if n < 30:
        print(f"ERROR: Hanya {n} sampel valid. Minimal 30 untuk Moran's I.")
        return

    print(f"Sampel: {n} titik, {index_type.upper()} mean={values.mean():.4f}, std={values.std():.4f}")

    # Try PySAL
    try:
        from libpysal.weights import KNN
        from esda.moran import Moran, Moran_Local

        # Spatial weights (k-nearest neighbors)
        w = KNN.from_array(coords, k=8)
        w.transform = 'r'  # row-standardize

        # Global Moran's I
        mi = Moran(values, w)

        print(f"\n=== GLOBAL MORAN'S I ===")
        print(f"Moran's I: {mi.I:.4f}")
        print(f"Expected I: {mi.EI:.4f}")
        print(f"p-value: {mi.p_sim:.6f}")
        print(f"z-score: {mi.z_sim:.4f}")

        if mi.p_sim < 0.05:
            if mi.I > 0:
                print(f"Interpretasi: Autokorelasi POSITIF signifikan — klaster spasial ada")
            else:
                print(f"Interpretasi: Autokorelasi NEGATIF signifikan — pola dispersi")
        else:
            print(f"Interpretasi: Tidak signifikan (p>{mi.p_sim:.3f}) — distribusi acak")

        # Local Moran's I (LISA)
        lisa = Moran_Local(values, w)

        # LISA cluster classification
        # 1=HH (hot spot), 2=LH (low outlier), 3=LL (cold spot), 4=HL (high outlier), 0=not significant
        sig = lisa.p_sim < 0.05
        quadrant = lisa.q  # 1=HH, 2=LH, 3=LL, 4=HL
        clusters = np.where(sig, quadrant, 0)

        hh = np.sum(clusters == 1)
        lh = np.sum(clusters == 2)
        ll = np.sum(clusters == 3)
        hl = np.sum(clusters == 4)
        ns = np.sum(clusters == 0)

        print(f"\n=== LOCAL MORAN'S I (LISA) ===")
        print(f"High-High (hot spot): {hh} ({100*hh/n:.1f}%)")
        print(f"Low-Low (cold spot): {ll} ({100*ll/n:.1f}%)")
        print(f"High-Low (outlier): {hl} ({100*hl/n:.1f}%)")
        print(f"Low-High (outlier): {lh} ({100*lh/n:.1f}%)")
        print(f"Tidak signifikan: {ns} ({100*ns/n:.1f}%)")

        # LISA cluster map
        fig, axes = plt.subplots(1, 2, figsize=(14, 6), dpi=150)

        # Left: scatter of index values
        sc1 = axes[0].scatter(coords[:, 0], coords[:, 1], c=values, cmap='RdYlGn',
                              s=8, alpha=0.7, edgecolors='none')
        axes[0].set_title(f'{index_type.upper()} Values', fontweight='bold')
        plt.colorbar(sc1, ax=axes[0], shrink=0.8, label=index_type.upper())

        # Right: LISA clusters
        colors_map = {0: '#cccccc', 1: '#d7191c', 2: '#abd9e9', 3: '#2c7bb6', 4: '#fdae61'}
        point_colors = [colors_map.get(c, '#cccccc') for c in clusters]
        axes[1].scatter(coords[:, 0], coords[:, 1], c=point_colors, s=8, alpha=0.7, edgecolors='none')
        axes[1].set_title(f'LISA Clusters (p<0.05)', fontweight='bold')

        # Legend
        from matplotlib.patches import Patch
        legend_elements = [
            Patch(facecolor='#d7191c', label=f'HH Hot Spot ({hh})'),
            Patch(facecolor='#2c7bb6', label=f'LL Cold Spot ({ll})'),
            Patch(facecolor='#fdae61', label=f'HL Outlier ({hl})'),
            Patch(facecolor='#abd9e9', label=f'LH Outlier ({lh})'),
            Patch(facecolor='#cccccc', label=f'Tidak Sig. ({ns})'),
        ]
        axes[1].legend(handles=legend_elements, loc='lower right', fontsize=7)

        for ax in axes:
            ax.set_aspect('equal')
            ax.tick_params(labelsize=7)

        fig.suptitle(f"Analisis Autokorelasi Spasial — Moran's I = {mi.I:.4f} (p={mi.p_sim:.4f})",
                     fontsize=12, fontweight='bold')
        fig.tight_layout()
        fig.savefig(output_path, dpi=150, bbox_inches='tight')
        plt.close(fig)

        # Provenance metadata
        try:
            from provenance import create_provenance
            create_provenance(output_path,
                tool='morans_i', gee_collection='COPERNICUS/S2_SR_HARMONIZED',
                coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
                algorithms=["Global Moran's I", "Local Moran's I (LISA)", 'KNN weights (k=8)'],
                references=['Anselin 1995', 'Rey & Anselin 2010'],
                parameters={'index_type': index_type, 'sample_scale_m': 100})
        except:
            pass  # provenance is non-critical

        print(f"\nOutput: {output_path}")
        print(f"Ref: Anselin 1995 (LISA), Rey & Anselin 2010 (PySAL)")

    except ImportError:
        print("WARNING: PySAL belum terinstall. Jalankan: pip install libpysal esda")
        print("Menghitung Moran's I manual (tanpa p-value)...")

        # Manual Global Moran's I (simplified, no permutation test)
        z = values - values.mean()

        # KNN weights (k=8) — manual distance-based
        from scipy.spatial import cKDTree
        tree = cKDTree(coords)
        _, indices = tree.query(coords, k=9)  # k+1 (includes self)

        numerator = 0.0
        denominator = z.dot(z)
        w_sum = 0.0

        for i in range(n):
            for j_idx in indices[i, 1:]:  # skip self
                numerator += z[i] * z[j_idx]
                w_sum += 1.0

        I = (n / w_sum) * (numerator / denominator)
        EI = -1.0 / (n - 1)

        print(f"\n=== GLOBAL MORAN'S I (manual) ===")
        print(f"Moran's I: {I:.4f}")
        print(f"Expected I: {EI:.4f}")
        print(f"Interpretasi: {'Positif (klaster)' if I > 0 else 'Negatif (dispersi)'}")
        print(f"CATATAN: p-value tidak tersedia tanpa PySAL")

        # Simple scatter plot
        fig, ax = plt.subplots(1, 1, figsize=(8, 6), dpi=150)
        sc = ax.scatter(coords[:, 0], coords[:, 1], c=values, cmap='RdYlGn',
                       s=8, alpha=0.7, edgecolors='none')
        plt.colorbar(sc, ax=ax, shrink=0.8, label=index_type.upper())
        ax.set_title(f"{index_type.upper()} — Moran's I = {I:.4f}", fontweight='bold')
        ax.set_aspect('equal')
        fig.tight_layout()
        fig.savefig(output_path, dpi=150, bbox_inches='tight')
        plt.close(fig)

        # Provenance metadata
        try:
            from provenance import create_provenance
            create_provenance(output_path,
                tool='morans_i', gee_collection='COPERNICUS/S2_SR_HARMONIZED',
                coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
                algorithms=["Global Moran's I (manual)", 'KNN weights (k=8)'],
                references=['Anselin 1995'],
                parameters={'index_type': index_type, 'sample_scale_m': 100})
        except:
            pass  # provenance is non-critical

        print(f"Output: {output_path}")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: spatial_stats_engine.py morans_i lat lon buffer_km index_type output_path")
        sys.exit(1)

    mode = sys.argv[1]
    try:
        if mode == 'morans_i':
            morans_i_analysis(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                             sys.argv[5], sys.argv[6])
        else:
            print(f"ERROR: Mode '{mode}' tidak dikenal.")
    except Exception as e:
        print(f"ERROR [E502]: {e}")
        import traceback
        traceback.print_exc()
