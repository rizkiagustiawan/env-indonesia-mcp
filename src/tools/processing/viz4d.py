#!/usr/bin/env python3
"""4D Visualization: 3D Terrain/Data + Time Animation
Menghasilkan GIF animasi dari sequence data temporal"""

import sys
import argparse
import numpy as np
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import os
import glob

def render_4d_terrain_rotation(dem_path, output_gif, title="4D Terrain", exaggeration=2.0, frames=36):
    """4D: Rotasi 360° dari terrain 3D — simulasi perspektif temporal"""
    try:
        import rasterio
        from matplotlib.animation import FuncAnimation, PillowWriter

        with rasterio.open(dem_path) as src:
            dem = src.read(1)
        
        max_safe_dim = 600 # GIF rendering in CPU is extremely slow, must be aggressively downsampled
        if dem.shape[0] > max_safe_dim or dem.shape[1] > max_safe_dim:
            factor = max(dem.shape[0] // max_safe_dim, dem.shape[1] // max_safe_dim, 1)
            dem = dem[::factor, ::factor]
        
        dem = np.where(dem < -9000, np.nan, dem)
        dem = np.where(np.isnan(dem), np.nanmin(dem), dem)
        
        rows, cols = dem.shape
        x = np.arange(0, cols)
        y = np.arange(0, rows)
        x, y = np.meshgrid(x, y)
        
        fig = plt.figure(figsize=(12, 9))
        ax = fig.add_subplot(111, projection='3d')
        
        colors = plt.cm.terrain((dem - np.nanmin(dem)) / (np.nanmax(dem) - np.nanmin(dem) + 0.001))
        
        surf = ax.plot_surface(x, y, dem * exaggeration, facecolors=colors,
                              rstride=3, cstride=3, antialiased=False, shade=True)
        
        ax.set_title(title, fontsize=14, fontweight='bold')
        ax.set_zlabel(f'Elevasi (m) x{exaggeration}')
        
        def update(frame):
            ax.view_init(elev=30 + 10 * np.sin(frame * np.pi / 18), azim=frame * (360 / frames))
            return []
        
        anim = FuncAnimation(fig, update, frames=frames, interval=100, blit=False)
        anim.save(output_gif, writer=PillowWriter(fps=10))
        plt.close()
        
        return f"SUCCESS: 4D terrain animation disimpan di {output_gif} ({frames} frames)"
    except Exception as e:
        return f"ERROR: {str(e)}"


def render_4d_timeseries(values, labels, output_gif, title="4D Time Series", ylabel="Value"):
    """4D: Animasi time series data lingkungan yang berkembang seiring waktu"""
    try:
        from matplotlib.animation import FuncAnimation, PillowWriter

        fig, ax = plt.subplots(figsize=(14, 7))
        
        n = len(values)
        x = list(range(n))
        
        line, = ax.plot([], [], 'b-o', linewidth=2, markersize=6)
        fill = None
        
        ax.set_xlim(-0.5, n - 0.5)
        ax.set_ylim(min(values) * 0.9, max(values) * 1.1)
        ax.set_title(title, fontsize=14, fontweight='bold')
        ax.set_ylabel(ylabel)
        ax.set_xlabel('Waktu')
        ax.grid(True, alpha=0.3)
        
        if labels:
            ax.set_xticks(x)
            ax.set_xticklabels(labels, rotation=45, ha='right', fontsize=8)
        
        text_box = ax.text(0.02, 0.95, '', transform=ax.transAxes, fontsize=11,
                          verticalalignment='top', bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.8))
        
        def update(frame):
            idx = frame + 1
            line.set_data(x[:idx], values[:idx])
            
            # Color coding
            current = values[frame]
            if current > max(values) * 0.8:
                color = 'red'
                status = 'KRITIS'
            elif current > max(values) * 0.5:
                color = 'orange' 
                status = 'PERINGATAN'
            else:
                color = 'green'
                status = 'NORMAL'
            
            line.set_color(color)
            lbl = labels[frame] if labels and frame < len(labels) else f'T{frame}'
            text_box.set_text(f'{lbl}\nNilai: {current:.1f}\nStatus: {status}')
            
            return [line, text_box]
        
        anim = FuncAnimation(fig, update, frames=n, interval=500, blit=False, repeat=True)
        anim.save(output_gif, writer=PillowWriter(fps=2))
        plt.close()
        
        return f"SUCCESS: 4D time series animation disimpan di {output_gif} ({n} frames)"
    except Exception as e:
        return f"ERROR: {str(e)}"


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", required=True, choices=["terrain", "timeseries"])
    parser.add_argument("--dem", help="DEM GeoTIFF path (terrain mode)")
    parser.add_argument("--values", help="Comma-separated values (timeseries mode)")
    parser.add_argument("--labels", help="Comma-separated labels (timeseries mode)")
    parser.add_argument("--output", required=True)
    parser.add_argument("--title", default="4D Visualization")
    parser.add_argument("--exaggeration", type=float, default=2.0)
    parser.add_argument("--frames", type=int, default=36)
    parser.add_argument("--ylabel", default="Value")
    args = parser.parse_args()
    
    if args.mode == "terrain":
        print(render_4d_terrain_rotation(args.dem, args.output, args.title, args.exaggeration, args.frames))
    elif args.mode == "timeseries":
        vals = [float(v) for v in args.values.split(',')]
        lbls = args.labels.split(',') if args.labels else None
        print(render_4d_timeseries(vals, lbls, args.output, args.title, args.ylabel))
