#!/usr/bin/env python3
"""Air Quality Dispersion Modeling Engine
Simplified AERMOD-like Gaussian Plume with 2D/3D/4D output
Ref: Turner (1970), Briggs (1969-1975), Pasquill-Gifford"""

import sys
import json
import argparse
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation, PillowWriter

# Briggs rural dispersion coefficients
def sigma_yz(stability, x):
    """Return (sigma_y, sigma_z) for given stability class and downwind distance x (meters)"""
    if x <= 0: return (0.001, 0.001)
    params = {
        'A': (0.22, -0.5, 0.0001, 0.20, 1.0, 0.0),
        'B': (0.16, -0.5, 0.0001, 0.12, 1.0, 0.0),
        'C': (0.11, -0.5, 0.0001, 0.08, -0.5, 0.0002),
        'D': (0.08, -0.5, 0.0001, 0.06, -0.5, 0.0015),
        'E': (0.06, -0.5, 0.0001, 0.03, -1.0, 0.0003),
        'F': (0.04, -0.5, 0.0001, 0.016, -1.0, 0.0003),
    }
    a, py, by, c, pz, bz = params.get(stability, params['D'])
    sy = a * x * (1 + by * x) ** py
    sz = c * x * (1 + bz * x) ** pz
    return (max(sy, 0.001), max(sz, 0.001))

def gaussian_conc(Q, u, H, x, y, z, stability):
    """Single source Gaussian plume concentration at (x,y,z)"""
    if x <= 0 or u < 0.28: return 0.0
    sy, sz = sigma_yz(stability, x)
    # Convert Q from g/s to ug/s for output in ug/m3
    Q_ug = Q * 1e6
    fy = np.exp(-y**2 / (2 * sy**2))
    gz = np.exp(-(z - H)**2 / (2 * sz**2)) + np.exp(-(z + H)**2 / (2 * sz**2))
    C = (Q_ug / (2 * np.pi * u * sy * sz)) * fy * gz
    return C

def multi_source_grid(sources, wind_speed, wind_dir_deg, stability, 
                       grid_size_m=5000, resolution_m=50, z=0):
    """
    sources: list of dicts {Q_gs, H_m, x_m, y_m}
    Returns: (X_grid, Y_grid, C_grid) in ug/m3
    """
    half = grid_size_m / 2
    nx = int(grid_size_m / resolution_m) + 1
    x_arr = np.linspace(-half, half, nx)
    y_arr = np.linspace(-half, half, nx)
    X, Y = np.meshgrid(x_arr, y_arr)
    C = np.zeros_like(X)
    
    wind_rad = np.radians(wind_dir_deg)
    cos_w = np.cos(wind_rad)
    sin_w = np.sin(wind_rad)
    
    for src in sources:
        Q = src['Q_gs']
        H = src['H_m']
        sx = src.get('x_m', 0)
        sy_pos = src.get('y_m', 0)
        
        # Rotate coordinates relative to wind direction
        dx = X - sx
        dy = Y - sy_pos
        # Downwind (x') and crosswind (y') in rotated frame
        x_rot = dx * cos_w + dy * sin_w
        y_rot = -dx * sin_w + dy * cos_w
        
        for i in range(nx):
            for j in range(nx):
                xr = x_rot[i, j]
                yr = y_rot[i, j]
                if xr > 10:  # Only downwind
                    C[i, j] += gaussian_conc(Q, wind_speed, H, xr, yr, z, stability)
    
    return X, Y, C

def render_contour_2d(sources, wind_speed, wind_dir, stability, output_path, 
                       title="Air Quality Dispersion", grid_size=5000, resolution=100):
    """2D contour map of ground-level concentration"""
    X, Y, C = multi_source_grid(sources, wind_speed, wind_dir, stability, grid_size, resolution)
    
    fig, ax = plt.subplots(figsize=(12, 10))
    
    # Log-scale contours
    C_safe = np.where(C > 0.01, C, 0.01)
    levels = [1, 5, 10, 25, 50, 65, 100, 150, 200, 500, 1000]
    
    cs = ax.contourf(X/1000, Y/1000, C_safe, levels=levels, cmap='RdYlGn_r', extend='both')
    ax.contour(X/1000, Y/1000, C_safe, levels=[65, 150], colors=['red', 'darkred'], linewidths=2)
    
    cbar = plt.colorbar(cs, ax=ax, label='Konsentrasi (µg/m³)')
    
    # Plot sources
    for src in sources:
        ax.plot(src.get('x_m', 0)/1000, src.get('y_m', 0)/1000, 'k^', markersize=12, label=f'Sumber (Q={src["Q_gs"]}g/s)')
    
    # Wind arrow
    arrow_len = grid_size / 5000
    wx = arrow_len * np.sin(np.radians(wind_dir))
    wy = arrow_len * np.cos(np.radians(wind_dir))
    ax.annotate('', xy=(wx, wy), xytext=(0, 0),
                arrowprops=dict(arrowstyle='->', color='blue', lw=2))
    ax.text(wx*1.2, wy*1.2, f'Angin {wind_dir}° @ {wind_speed}m/s', color='blue', fontsize=9)
    
    ax.set_title(f'{title}\nStabilitas: {stability} | Baku Mutu PM2.5: 65 µg/m³ (garis merah)', fontsize=13, fontweight='bold')
    ax.set_xlabel('X (km)')
    ax.set_ylabel('Y (km)')
    ax.legend(loc='upper right')
    ax.set_aspect('equal')
    
    fig.text(0.02, 0.02, 'Model: Gaussian Plume (Briggs rural) | ZeroClaw Environmental AI | PP 22/2021 Lampiran VII + PermenLHK 8/2024', fontsize=8, style='italic')
    
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    plt.close()
    
    max_c = np.max(C)
    exceed_65 = np.sum(C > 65) / C.size * 100
    return f"SUCCESS: 2D contour map disimpan di {output_path}. Max konsentrasi: {max_c:.1f} µg/m³. Area > 65 µg/m³: {exceed_65:.1f}%"

def render_3d_plume(sources, wind_speed, wind_dir, stability, output_path, 
                     title="3D Air Dispersion", grid_size=5000, resolution=100):
    """3D visualization of plume"""
    X, Y, C = multi_source_grid(sources, wind_speed, wind_dir, stability, grid_size, resolution)
    
    fig = plt.figure(figsize=(14, 10))
    ax = fig.add_subplot(111, projection='3d')
    
    C_log = np.log10(np.where(C > 0.1, C, 0.1))
    
    ax.plot_surface(X/1000, Y/1000, C_log, cmap='hot_r', alpha=0.8, rstride=2, cstride=2)
    
    for src in sources:
        ax.scatter(src.get('x_m',0)/1000, src.get('y_m',0)/1000, 0, c='black', s=100, marker='^')
    
    ax.set_title(title, fontsize=13, fontweight='bold')
    ax.set_xlabel('X (km)')
    ax.set_ylabel('Y (km)')
    ax.set_zlabel('log₁₀(C) µg/m³')
    ax.view_init(elev=30, azim=225)
    
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    plt.close()
    return f"SUCCESS: 3D plume disimpan di {output_path}"

def render_4d_dispersion(sources, wind_speeds, wind_dirs, stability, output_gif,
                          title="4D Air Dispersion", grid_size=5000, resolution=150):
    """4D animation: changing wind direction/speed over time"""
    n_frames = len(wind_speeds)
    
    fig, ax = plt.subplots(figsize=(12, 10))
    
    def update(frame):
        ax.clear()
        ws = wind_speeds[frame]
        wd = wind_dirs[frame]
        X, Y, C = multi_source_grid(sources, ws, wd, stability, grid_size, resolution)
        C_safe = np.where(C > 0.01, C, 0.01)
        levels = [1, 5, 10, 25, 50, 65, 100, 150, 200, 500]
        ax.contourf(X/1000, Y/1000, C_safe, levels=levels, cmap='RdYlGn_r', extend='both')
        ax.contour(X/1000, Y/1000, C_safe, levels=[65], colors=['red'], linewidths=2)
        
        for src in sources:
            ax.plot(src.get('x_m',0)/1000, src.get('y_m',0)/1000, 'k^', markersize=10)
        
        ax.set_title(f'{title}\nFrame {frame+1}/{n_frames} | Wind: {wd}° @ {ws}m/s | Stability: {stability}',
                    fontsize=11, fontweight='bold')
        ax.set_xlabel('X (km)')
        ax.set_ylabel('Y (km)')
        ax.set_aspect('equal')
        return []
    
    anim = FuncAnimation(fig, update, frames=n_frames, interval=800, blit=False)
    anim.save(output_gif, writer=PillowWriter(fps=2))
    plt.close()
    return f"SUCCESS: 4D dispersion animation disimpan di {output_gif} ({n_frames} frames)"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", required=True, choices=["2d", "3d", "4d"])
    parser.add_argument("--sources", required=True, help="JSON array of {Q_gs, H_m, x_m, y_m}")
    parser.add_argument("--wind_speed", type=float, default=3.0)
    parser.add_argument("--wind_dir", type=float, default=180.0)
    parser.add_argument("--stability", default="D")
    parser.add_argument("--output", required=True)
    parser.add_argument("--title", default="Air Quality Dispersion Model")
    parser.add_argument("--grid_size", type=int, default=5000)
    parser.add_argument("--resolution", type=int, default=100)
    # 4D specific
    parser.add_argument("--wind_speeds", help="Comma-separated wind speeds for 4D")
    parser.add_argument("--wind_dirs", help="Comma-separated wind directions for 4D")
    args = parser.parse_args()
    
    sources = json.loads(args.sources)
    
    if args.mode == "2d":
        print(render_contour_2d(sources, args.wind_speed, args.wind_dir, args.stability,
                                args.output, args.title, args.grid_size, args.resolution))
    elif args.mode == "3d":
        print(render_3d_plume(sources, args.wind_speed, args.wind_dir, args.stability,
                              args.output, args.title, args.grid_size, args.resolution))
    elif args.mode == "4d":
        ws = [float(v) for v in args.wind_speeds.split(',')]
        wd = [float(v) for v in args.wind_dirs.split(',')]
        print(render_4d_dispersion(sources, ws, wd, args.stability,
                                    args.output, args.title, args.grid_size, args.resolution))
