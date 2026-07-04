#!/usr/bin/env python3
"""Ocean Visualization Engine: Bathymetry 3D, Current 2D, Thermal 3D, Pollution 4D"""

import sys
import json
import argparse
import numpy as np
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
from matplotlib.animation import FuncAnimation, PillowWriter

def bathymetry_3d(output_path, center_lat, center_lon, size_deg=1.0, title="3D Bathymetry"):
    """3D ocean floor visualization — simplified model"""
    n = 200
    x = np.linspace(center_lon - size_deg/2, center_lon + size_deg/2, n)
    y = np.linspace(center_lat - size_deg/2, center_lat + size_deg/2, n)
    X, Y = np.meshgrid(x, y)
    
    # Simulate bathymetry: shelf + slope + deep ocean
    dist_from_coast = np.sqrt((X - center_lon)**2 + (Y - center_lat)**2)
    Z = -50 - 2000 * np.tanh(dist_from_coast * 3) + 200 * np.sin(X*10) * np.cos(Y*8) + np.random.normal(0, 30, (n,n))
    Z = np.clip(Z, -5000, 0)
    
    fig = plt.figure(figsize=(14, 10))
    ax = fig.add_subplot(111, projection='3d')
    colors = plt.cm.ocean((Z - Z.min()) / (Z.max() - Z.min() + 0.001))
    ax.plot_surface(X, Y, Z, facecolors=colors, rstride=3, cstride=3, antialiased=True, shade=True)
    ax.set_title(title, fontsize=14, fontweight='bold')
    ax.set_xlabel('Longitude')
    ax.set_ylabel('Latitude')
    ax.set_zlabel('Depth (m)')
    ax.view_init(elev=30, azim=225)
    fig.text(0.02, 0.02, f'Center: {center_lat:.2f}, {center_lon:.2f} | ZeroClaw Environmental AI', fontsize=8, style='italic')
    plt.savefig(output_path, dpi=200, bbox_inches='tight')
    plt.close()
    return f"SUCCESS: 3D Bathymetry disimpan di {output_path}. Depth range: {Z.min():.0f}m to {Z.max():.0f}m"

def current_2d(output_path, center_lat, center_lon, wind_speed=5, wind_dir=180, title="2D Ocean Current"):
    """2D ocean current vector field"""
    n = 25
    x = np.linspace(center_lon - 0.5, center_lon + 0.5, n)
    y = np.linspace(center_lat - 0.5, center_lat + 0.5, n)
    X, Y = np.meshgrid(x, y)
    
    wind_rad = np.radians(wind_dir)
    # Ekman spiral: surface current ~45deg right of wind (Southern Hemisphere: left)
    ekman_angle = -45 if center_lat < 0 else 45
    current_dir = wind_rad + np.radians(ekman_angle)
    current_speed = 0.03 * wind_speed  # ~3% of wind speed
    
    U = current_speed * np.sin(current_dir) + 0.1 * np.sin(X * 5) + np.random.normal(0, 0.02, (n,n))
    V = current_speed * np.cos(current_dir) + 0.1 * np.cos(Y * 5) + np.random.normal(0, 0.02, (n,n))
    speed = np.sqrt(U**2 + V**2)
    
    fig, ax = plt.subplots(figsize=(12, 10))
    cs = ax.contourf(X, Y, speed, levels=15, cmap='YlOrRd', alpha=0.7)
    ax.quiver(X, Y, U, V, speed, cmap='coolwarm', scale=3, width=0.003)
    plt.colorbar(cs, ax=ax, label='Current Speed (m/s)')
    ax.set_title(f'{title}\nWind: {wind_dir} deg @ {wind_speed} m/s | Ekman deflection: {ekman_angle} deg', fontsize=13, fontweight='bold')
    ax.set_xlabel('Longitude')
    ax.set_ylabel('Latitude')
    ax.set_aspect('equal')
    fig.text(0.02, 0.02, 'Model: Wind-driven Ekman + perturbation | ZeroClaw Environmental AI', fontsize=8, style='italic')
    plt.savefig(output_path, dpi=200, bbox_inches='tight')
    plt.close()
    return f"SUCCESS: 2D Current map disimpan di {output_path}. Max speed: {speed.max():.3f} m/s"

def thermal_3d(output_path, discharge_temp, ambient_temp, discharge_rate, title="3D Thermal Mixing"):
    """3D thermal pollution mixing zone from coastal PLTU"""
    n = 100
    x = np.linspace(0, 1000, n)  # meters
    y = np.linspace(-500, 500, n)
    z = np.linspace(-20, 0, 30)  # depth
    X, Y = np.meshgrid(x, y)
    
    L_mix = 200  # horizontal mixing scale (m)
    D_mix = 10   # vertical mixing depth (m)
    dT = discharge_temp - ambient_temp
    
    fig = plt.figure(figsize=(14, 10))
    ax = fig.add_subplot(111, projection='3d')
    
    # Surface layer
    R = np.sqrt(X**2 + Y**2)
    T_surface = ambient_temp + dT * np.exp(-R / L_mix)
    colors = plt.cm.hot((T_surface - ambient_temp) / (dT + 0.001))
    ax.plot_surface(X, Y, T_surface - ambient_temp, facecolors=colors, rstride=2, cstride=2, alpha=0.8)
    
    ax.set_title(f'{title}\nDischarge: {discharge_temp:.0f} C | Ambient: {ambient_temp:.0f} C | DeltaT: {dT:.0f} C', fontsize=13, fontweight='bold')
    ax.set_xlabel('Distance (m)')
    ax.set_ylabel('Crosswind (m)')
    ax.set_zlabel('DeltaT (C)')
    ax.view_init(elev=25, azim=225)
    
    # Baku mutu line
    baku_mutu_radius = -L_mix * np.log(3.0 / dT) if dT > 3 else 0
    fig.text(0.02, 0.02, f'Baku mutu PP 22/2021: DeltaT max 3 C | Radius baku mutu: {baku_mutu_radius:.0f}m\nZeroClaw Environmental AI', fontsize=8, style='italic')
    
    plt.savefig(output_path, dpi=200, bbox_inches='tight')
    plt.close()
    return f"SUCCESS: 3D Thermal mixing disimpan di {output_path}. Mixing length: {L_mix}m, Baku mutu radius: {baku_mutu_radius:.0f}m"

def pollution_4d(output_path, source_x, source_y, current_speeds, current_dirs, diffusion_k=5.0, n_particles=200, title="4D Marine Pollution"):
    """4D Lagrangian particle tracking animation"""
    n_frames = len(current_speeds)
    dt = 3600  # 1 hour timestep
    
    # Initialize particles at source
    px = np.full(n_particles, source_x) + np.random.normal(0, 10, n_particles)
    py = np.full(n_particles, source_y) + np.random.normal(0, 10, n_particles)
    
    fig, ax = plt.subplots(figsize=(12, 10))
    
    all_px = [px.copy()]
    all_py = [py.copy()]
    
    for i in range(n_frames):
        u = current_speeds[i] * np.sin(np.radians(current_dirs[i]))
        v = current_speeds[i] * np.cos(np.radians(current_dirs[i]))
        px = px + u * dt + np.random.normal(0, np.sqrt(2 * diffusion_k * dt), n_particles)
        py = py + v * dt + np.random.normal(0, np.sqrt(2 * diffusion_k * dt), n_particles)
        all_px.append(px.copy())
        all_py.append(py.copy())
    
    def update(frame):
        ax.clear()
        # Plot trajectory trails
        for t in range(max(0, frame-3), frame+1):
            alpha = 0.2 + 0.8 * (t - max(0, frame-3)) / 4
            ax.scatter(all_px[t], all_py[t], s=5, c='brown', alpha=alpha)
        ax.scatter(all_px[frame], all_py[frame], s=15, c='red', alpha=0.8, label='Polutan')
        ax.plot(source_x, source_y, 'k^', markersize=12, label='Sumber')
        
        u = current_speeds[min(frame, n_frames-1)] 
        d = current_dirs[min(frame, n_frames-1)]
        ax.set_title(f'{title}\nFrame {frame+1}/{n_frames+1} | Arus: {d:.0f} deg @ {u:.2f} m/s', fontsize=11, fontweight='bold')
        ax.set_xlabel('X (m)')
        ax.set_ylabel('Y (m)')
        ax.legend(loc='upper right')
        ax.set_aspect('equal')
        return []
    
    anim = FuncAnimation(fig, update, frames=n_frames+1, interval=500, blit=False)
    anim.save(output_path, writer=PillowWriter(fps=2))
    plt.close()
    return f"SUCCESS: 4D Pollution animation disimpan di {output_path} ({n_frames+1} frames)"

def oil_spill_viz(output_path, volume_m3, oil_type, wind_speed, wind_dir, current_speed, current_dir, hours, title="Oil Spill Trajectory"):
    """Animated GIF showing oil spill particle drift over time"""
    n_particles = 300
    dt = 3600  # 1 hour timestep
    n_frames = min(hours, 72)  # cap frames

    # Evaporation rate by oil type
    k_evap = {"crude": 0.02, "mentah": 0.02, "diesel": 0.08, "gasoline": 0.20, "bensin": 0.20, "bunker": 0.005, "hfo": 0.005}.get(oil_type.lower(), 0.02)

    # Wind drift (3% of wind) + current
    drift_wind = 0.03 * wind_speed
    ux = drift_wind * np.sin(np.radians(wind_dir)) + current_speed * np.sin(np.radians(current_dir))
    uy = drift_wind * np.cos(np.radians(wind_dir)) + current_speed * np.cos(np.radians(current_dir))

    # Initialize particles at origin
    px = np.random.normal(0, 20, n_particles)
    py = np.random.normal(0, 20, n_particles)

    all_px = [px.copy()]
    all_py = [py.copy()]
    all_evap = [0.0]

    for t in range(1, n_frames + 1):
        spread = 5.0 + 2.0 * t  # increasing diffusion over time
        px = px + ux * dt + np.random.normal(0, spread, n_particles)
        py = py + uy * dt + np.random.normal(0, spread, n_particles)
        evap_pct = (1.0 - np.exp(-k_evap * t)) * 100
        all_px.append(px.copy())
        all_py.append(py.copy())
        all_evap.append(evap_pct)

    fig, ax = plt.subplots(figsize=(12, 10))

    def update(frame):
        ax.clear()
        # Trail
        for t in range(max(0, frame - 4), frame + 1):
            alpha = 0.1 + 0.15 * (t - max(0, frame - 4))
            evap_frac = all_evap[t] / 100.0
            color_val = plt.cm.YlOrBr(0.3 + 0.5 * evap_frac)
            ax.scatter(all_px[t] / 1000, all_py[t] / 1000, s=4, color=color_val, alpha=alpha)
        # Current frame - oil slick
        evap_frac = all_evap[frame] / 100.0
        remaining = 1.0 - evap_frac
        color = plt.cm.YlOrBr(0.3 + 0.5 * evap_frac)
        ax.scatter(all_px[frame] / 1000, all_py[frame] / 1000, s=8 * remaining + 2, color=color, alpha=0.7, edgecolors='k', linewidths=0.2)
        ax.plot(0, 0, 'r^', markersize=14, label='Spill Source')
        vol_remain = volume_m3 * np.exp(-k_evap * frame)
        ax.set_title(f'{title}\nT+{frame}h | Evap: {all_evap[frame]:.0f}% | Vol: {vol_remain:.0f} m3 | {oil_type}', fontsize=11, fontweight='bold')
        ax.set_xlabel('East-West (km)')
        ax.set_ylabel('North-South (km)')
        ax.legend(loc='upper left')
        ax.set_aspect('equal')
        ax.grid(True, alpha=0.3)
        fig.text(0.02, 0.02, f'Wind: {wind_dir} deg @ {wind_speed} m/s | Current: {current_dir} deg @ {current_speed} m/s | ZeroClaw AI', fontsize=7, style='italic')
        return []

    anim = FuncAnimation(fig, update, frames=n_frames + 1, interval=500, blit=False)
    anim.save(output_path, writer=PillowWriter(fps=2))
    plt.close()
    return f"SUCCESS: Oil spill animation saved to {output_path} ({n_frames + 1} frames)"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", required=True, choices=["bathymetry3d", "current2d", "thermal3d", "pollution4d", "oil_spill_viz"])
    parser.add_argument("--output", required=True)
    parser.add_argument("--title", default="Ocean Visualization")
    parser.add_argument("--lat", type=float, default=-8.5)
    parser.add_argument("--lon", type=float, default=116.5)
    parser.add_argument("--wind_speed", type=float, default=5)
    parser.add_argument("--wind_dir", type=float, default=180)
    parser.add_argument("--discharge_temp", type=float, default=40)
    parser.add_argument("--ambient_temp", type=float, default=28)
    parser.add_argument("--discharge_rate", type=float, default=5)
    parser.add_argument("--current_speeds", default="0.3,0.35,0.4,0.3,0.25,0.2")
    parser.add_argument("--current_dirs", default="180,190,200,210,220,230")
    parser.add_argument("--volume_m3", type=float, default=100)
    parser.add_argument("--oil_type", default="crude")
    parser.add_argument("--current_speed", type=float, default=0.3)
    parser.add_argument("--current_dir", type=float, default=180)
    parser.add_argument("--hours", type=int, default=24)
    args = parser.parse_args()
    
    if args.mode == "bathymetry3d":
        print(bathymetry_3d(args.output, args.lat, args.lon, title=args.title))
    elif args.mode == "current2d":
        print(current_2d(args.output, args.lat, args.lon, args.wind_speed, args.wind_dir, args.title))
    elif args.mode == "thermal3d":
        print(thermal_3d(args.output, args.discharge_temp, args.ambient_temp, args.discharge_rate, args.title))
    elif args.mode == "pollution4d":
        cs = [float(v) for v in args.current_speeds.split(',')]
        cd = [float(v) for v in args.current_dirs.split(',')]
        print(pollution_4d(args.output, 0, 0, cs, cd, title=args.title))
    elif args.mode == "oil_spill_viz":
        print(oil_spill_viz(args.output, args.volume_m3, args.oil_type, args.wind_speed, args.wind_dir, args.current_speed, args.current_dir, args.hours, title=args.title))
