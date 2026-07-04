#!/usr/bin/env python3
"""3D & 4D Flood Simulation dari DEM
Simulasi genangan banjir pada berbagai level air"""

import sys
import argparse
import numpy as np
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
from matplotlib.animation import FuncAnimation, PillowWriter

def load_dem(dem_path, max_dim=300):
    import rasterio
    with rasterio.open(dem_path) as src:
        dem = src.read(1)
        transform = src.transform
    if dem.shape[0] > max_dim or dem.shape[1] > max_dim:
        factor = max(dem.shape[0] // max_dim, dem.shape[1] // max_dim, 1)
        dem = dem[::factor, ::factor]
    dem = np.where(dem < -9000, np.nan, dem)
    dem = np.where(np.isnan(dem), np.nanmin(dem), dem)
    return dem

def render_flood_3d(dem_path, output_path, water_level_m, title="3D Flood Simulation", exaggeration=2.0):
    """3D statis: terrain + genangan air pada level tertentu"""
    try:
        dem = load_dem(dem_path)
        rows, cols = dem.shape
        x = np.arange(0, cols)
        y = np.arange(0, rows)
        x, y = np.meshgrid(x, y)

        fig = plt.figure(figsize=(16, 12))
        ax = fig.add_subplot(111, projection='3d')

        # Terrain surface
        terrain_colors = plt.cm.terrain((dem - np.nanmin(dem)) / (np.nanmax(dem) - np.nanmin(dem) + 0.001))
        ax.plot_surface(x, y, dem * exaggeration, facecolors=terrain_colors,
                       rstride=2, cstride=2, antialiased=True, shade=True, alpha=0.9)

        # Water surface (flat plane at water_level)
        water = np.full_like(dem, water_level_m)
        water_mask = dem <= water_level_m
        water_visible = np.where(water_mask, water_level_m * exaggeration, np.nan)

        # Render air sebagai surface biru transparan
        water_colors = np.zeros((*dem.shape, 4))
        water_colors[water_mask] = [0.1, 0.3, 0.8, 0.6]  # biru transparan
        water_colors[~water_mask] = [0, 0, 0, 0]  # transparan

        ax.plot_surface(x, y, water_visible, facecolors=water_colors,
                       rstride=2, cstride=2, antialiased=True, shade=False)

        # Stats
        flooded_area_pct = np.sum(water_mask) / water_mask.size * 100
        max_depth = np.nanmax(water_level_m - dem[water_mask]) if np.any(water_mask) else 0

        ax.set_title(f'{title}\nLevel Air: {water_level_m}m | Area Genangan: {flooded_area_pct:.1f}% | Kedalaman Maks: {max_depth:.1f}m',
                    fontsize=13, fontweight='bold')
        ax.set_zlabel(f'Elevasi (m) x{exaggeration}')
        ax.view_init(elev=35, azim=225)

        fig.text(0.02, 0.02,
                f'Min Elevasi: {np.nanmin(dem):.0f}m | Max: {np.nanmax(dem):.0f}m | '
                f'Water Level: {water_level_m}m\n'
                # NOTE: This is a bathtub/static inundation model, NOT a hydraulic simulation.
                f'ZeroClaw Environmental AI — Bathtub Inundation Visualization',
                fontsize=9, style='italic')

        plt.savefig(output_path, dpi=200, bbox_inches='tight')
        plt.close()

        return f"SUCCESS: 3D Flood simulation disimpan di {output_path}. Area genangan: {flooded_area_pct:.1f}%, kedalaman maks: {max_depth:.1f}m"

    except Exception as e:
        return f"ERROR: {str(e)}"


def render_flood_4d(dem_path, output_gif, water_start_m, water_end_m, steps=20, title="4D Flood Simulation", exaggeration=2.0):
    """4D animasi: simulasi kenaikan level air dari start ke end"""
    try:
        dem = load_dem(dem_path, max_dim=200)  # Smaller for animation perf
        rows, cols = dem.shape
        x = np.arange(0, cols)
        y = np.arange(0, rows)
        x, y = np.meshgrid(x, y)

        water_levels = np.linspace(water_start_m, water_end_m, steps)

        fig = plt.figure(figsize=(14, 10))
        ax = fig.add_subplot(111, projection='3d')

        terrain_colors = plt.cm.terrain((dem - np.nanmin(dem)) / (np.nanmax(dem) - np.nanmin(dem) + 0.001))

        def update(frame):
            ax.clear()
            wl = water_levels[frame]

            # Terrain
            ax.plot_surface(x, y, dem * exaggeration, facecolors=terrain_colors,
                          rstride=3, cstride=3, antialiased=False, shade=True, alpha=0.85)

            # Water
            water_mask = dem <= wl
            if np.any(water_mask):
                water_surface = np.where(water_mask, wl * exaggeration, np.nan)
                wcolors = np.zeros((*dem.shape, 4))
                wcolors[water_mask] = [0.1, 0.3, 0.85, 0.7]
                ax.plot_surface(x, y, water_surface, facecolors=wcolors,
                              rstride=3, cstride=3, antialiased=False, shade=False)

            flooded_pct = np.sum(water_mask) / water_mask.size * 100
            max_depth = np.nanmax(wl - dem[water_mask]) if np.any(water_mask) else 0

            ax.set_title(f'{title}\nLevel: {wl:.1f}m | Genangan: {flooded_pct:.1f}% | Depth: {max_depth:.1f}m | Frame {frame+1}/{steps}',
                        fontsize=11, fontweight='bold')
            ax.set_zlabel(f'Elevasi x{exaggeration}')
            ax.view_init(elev=35, azim=225 + frame * 2)

            return []

        anim = FuncAnimation(fig, update, frames=steps, interval=500, blit=False)
        anim.save(output_gif, writer=PillowWriter(fps=3))
        plt.close()

        return f"SUCCESS: 4D Flood animation disimpan di {output_gif} ({steps} frames, level {water_start_m}-{water_end_m}m)"

    except Exception as e:
        return f"ERROR: {str(e)}"


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", required=True, choices=["3d", "4d"])
    parser.add_argument("--dem", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--water_level", type=float, default=100)
    parser.add_argument("--water_start", type=float, default=0)
    parser.add_argument("--water_end", type=float, default=200)
    parser.add_argument("--steps", type=int, default=20)
    parser.add_argument("--title", default="Flood Simulation")
    parser.add_argument("--exaggeration", type=float, default=2.0)
    args = parser.parse_args()

    if args.mode == "3d":
        print(render_flood_3d(args.dem, args.output, args.water_level, args.title, args.exaggeration))
    else:
        print(render_flood_4d(args.dem, args.output, args.water_start, args.water_end, args.steps, args.title, args.exaggeration))
