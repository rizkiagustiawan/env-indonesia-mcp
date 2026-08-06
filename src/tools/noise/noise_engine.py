#!/usr/bin/env python3
"""Noise Propagation Modeling Engine
ISO 9613-2 point source propagation with barrier support
2D contour and 3D surface noise maps
"""

import sys
import json
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import BoundaryNorm, ListedColormap
from mpl_toolkits.mplot3d import Axes3D


def iso9613_point_source(lw, r, a_ground=0.0, a_barrier=0.0):
    """ISO 9613-2 point source noise level at distance r.
    L = Lw - 20*log10(r) - 11 - A_ground - A_barrier
    """
    if r <= 0.5:
        r = 0.5
    return lw - 20.0 * np.log10(r) - 11.0 - a_ground - a_barrier


def barrier_insertion_loss(source, receiver, barrier):
    """Calculate approximate barrier insertion loss (Maekawa method).
    Returns IL in dB (0 if no barrier effect).
    """
    sx, sy = source
    rx, ry = receiver
    bx1, by1, bx2, by2 = barrier["x1"], barrier["y1"], barrier["x2"], barrier["y2"]
    h_barrier = barrier.get("height_m", 3.0)
    fixed_il = barrier.get("il_db", None)

    # Check if line from source to receiver crosses barrier line
    def cross_product(o, a, b):
        return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])

    d1 = cross_product((bx1, by1), (bx2, by2), (sx, sy))
    d2 = cross_product((bx1, by1), (bx2, by2), (rx, ry))
    d3 = cross_product((sx, sy), (rx, ry), (bx1, by1))
    d4 = cross_product((sx, sy), (rx, ry), (bx2, by2))

    if d1 * d2 < 0 and d3 * d4 < 0:
        # Barrier is between source and receiver
        if fixed_il is not None:
            return fixed_il
        # Simplified Maekawa: IL = 10*log10(3 + 20*N) where N = Fresnel number
        # Approximate N from path difference and frequency (assume 500 Hz)
        d_sr = np.sqrt((rx - sx)**2 + (ry - sy)**2)
        # Find intersection point
        t = d1 / (d1 - d2) if (d1 - d2) != 0 else 0.5
        ix = sx + t * (rx - sx)
        iy = sy + t * (ry - sy)
        d_sb = np.sqrt((ix - sx)**2 + (iy - sy)**2)
        d_br = np.sqrt((rx - ix)**2 + (ry - iy)**2)
        # Path difference delta = d_sb + d_br - d_sr + h_barrier factor
        delta = np.sqrt(d_sb**2 + h_barrier**2) + np.sqrt(d_br**2 + h_barrier**2) - d_sr
        freq = 500.0  # Hz
        c = 343.0  # m/s speed of sound
        N = 2.0 * delta * freq / c
        if N > 0:
            il = 10.0 * np.log10(3.0 + 20.0 * N)
            return min(il, 25.0)  # cap at 25 dB
    return 0.0


def compute_noise_grid(sources, grid_size_m, barriers=None, resolution=None):
    """Compute noise level grid from multiple point sources."""
    if resolution is None:
        resolution = max(2, grid_size_m // 100)
    half = grid_size_m / 2.0
    nx = int(grid_size_m / resolution) + 1
    x_arr = np.linspace(-half, half, nx)
    y_arr = np.linspace(-half, half, nx)
    X, Y = np.meshgrid(x_arr, y_arr)
    L_total = np.full_like(X, -999.0)

    for i in range(nx):
        for j in range(nx):
            rx, ry = X[i, j], Y[i, j]
            linear_sum = 0.0
            for src in sources:
                sx = src.get("x_m", 0)
                sy = src.get("y_m", 0)
                lw = src.get("power_db", 95)
                r = np.sqrt((rx - sx)**2 + (ry - sy)**2)
                # Ground attenuation: soft ground approximation
                a_ground = 0.0
                if r > 50:
                    a_ground = min(3.0, r / 200.0 * 3.0)
                # Barrier attenuation
                a_barrier = 0.0
                if barriers:
                    for b in barriers:
                        il = barrier_insertion_loss((sx, sy), (rx, ry), b)
                        a_barrier = max(a_barrier, il)
                level = iso9613_point_source(lw, r, a_ground, a_barrier)
                if level > 0:
                    linear_sum += 10.0 ** (level / 10.0)
            if linear_sum > 0:
                L_total[i, j] = 10.0 * np.log10(linear_sum)
            else:
                L_total[i, j] = 0.0

    return X, Y, L_total


def render_2d_contour(sources, output_path, title, grid_size, barriers=None):
    """Render 2D noise contour map."""
    X, Y, L = compute_noise_grid(sources, grid_size, barriers)

    fig, ax = plt.subplots(figsize=(12, 10))

    # Zone boundary levels per KepmenLH 48/1996
    levels = [35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90]
    colors = [
        "#1a9641", "#4daf4a", "#73d216", "#a6d96a",  # green (<50)
        "#2196f3", "#42a5f5",                         # blue (50-60)
        "#ffeb3b", "#ffc107", "#ff9800",              # yellow/orange (60-70)
        "#f44336", "#d32f2f",                         # red (>70)
    ]
    cmap = ListedColormap(colors)
    norm = BoundaryNorm(levels, cmap.N)

    cs = ax.contourf(X, Y, L, levels=levels, cmap=cmap, norm=norm, extend="both")
    # Labeled contour lines for zone boundaries
    zone_levels = [50, 55, 65, 70, 73]
    cl = ax.contour(X, Y, L, levels=zone_levels, colors="black", linewidths=1.5)
    ax.clabel(cl, inline=True, fontsize=9, fmt="%.0f dBA")

    # Highlight regulatory boundaries
    ax.contour(X, Y, L, levels=[55], colors=["blue"], linewidths=2, linestyles="dashed")
    ax.contour(X, Y, L, levels=[70], colors=["red"], linewidths=2, linestyles="dashed")

    cbar = plt.colorbar(cs, ax=ax, label="Tingkat Kebisingan (dBA)")

    # Plot sources
    for src in sources:
        ax.plot(src.get("x_m", 0), src.get("y_m", 0), "k^", markersize=12)
        ax.annotate(f'{src.get("power_db", 95)} dB',
                    (src.get("x_m", 0), src.get("y_m", 0)),
                    textcoords="offset points", xytext=(5, 10), fontsize=8)

    # Plot barriers
    if barriers:
        for b in barriers:
            ax.plot([b["x1"], b["x2"]], [b["y1"], b["y2"]], "k-", linewidth=4, label=f'Barrier ({b.get("height_m", 3)}m)')

    ax.set_title(f"{title}\nRef: ISO 9613-2 | Biru putus-putus=55dBA (perumahan) | Merah putus-putus=70dBA (perdagangan)",
                 fontsize=11, fontweight="bold")
    ax.set_xlabel("X (m)")
    ax.set_ylabel("Y (m)")
    ax.set_aspect("equal")
    ax.legend(loc="upper right")

    fig.text(0.02, 0.02, "Model: ISO 9613-2 Point Source | KepmenLH 48/1996 | ZeroClaw Environmental AI",
             fontsize=8, style="italic")

    plt.savefig(output_path, dpi=300, bbox_inches="tight")
    plt.close()

    max_l = np.max(L)
    area_55 = np.sum(L > 55) / L.size * 100
    area_70 = np.sum(L > 70) / L.size * 100
    return (f"SUCCESS: Peta kebisingan 2D disimpan di {output_path}. "
            f"Max: {max_l:.1f} dBA. Area >55dBA: {area_55:.1f}%. Area >70dBA: {area_70:.1f}%")


def render_3d_surface(sources, output_path, title, grid_size):
    """Render 3D surface plot of noise levels."""
    X, Y, L = compute_noise_grid(sources, grid_size)

    fig = plt.figure(figsize=(14, 10))
    ax = fig.add_subplot(111, projection="3d")

    # Cap values for visualization
    L_vis = np.clip(L, 20, 100)

    surf = ax.plot_surface(X, Y, L_vis, cmap="RdYlGn_r", alpha=0.85, rstride=2, cstride=2)

    # Plot source positions
    for src in sources:
        ax.scatter(src.get("x_m", 0), src.get("y_m", 0), src.get("power_db", 95),
                   c="black", s=100, marker="^", zorder=5)

    # Add horizontal planes for zone limits
    half = grid_size / 2.0
    xx = np.array([[-half, half], [-half, half]])
    yy = np.array([[-half, -half], [half, half]])
    ax.plot_surface(xx, yy, np.full_like(xx, 55.0), alpha=0.15, color="blue", label="55 dBA")
    ax.plot_surface(xx, yy, np.full_like(xx, 70.0), alpha=0.15, color="red", label="70 dBA")

    plt.colorbar(surf, ax=ax, label="Tingkat Kebisingan (dBA)", shrink=0.6)
    ax.set_title(f"{title}\nISO 9613-2 | Bidang biru=55dBA, merah=70dBA", fontsize=12, fontweight="bold")
    ax.set_xlabel("X (m)")
    ax.set_ylabel("Y (m)")
    ax.set_zlabel("dBA")
    ax.view_init(elev=35, azim=225)

    plt.savefig(output_path, dpi=300, bbox_inches="tight")
    plt.close()

    return f"SUCCESS: Peta kebisingan 3D disimpan di {output_path}"


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: noise_engine.py <mode> <sources_json> <output_path> <title> <grid_size> [barrier_json]")
        print("Modes: 2d, 3d")
        sys.exit(1)

    mode = sys.argv[1]

    try:
        if mode == "2d":
            if len(sys.argv) < 6:
                print("ERROR: 2d memerlukan: sources_json output_path title grid_size [barrier_json]")
                sys.exit(1)
            sources = json.loads(sys.argv[2])
            output_path = sys.argv[3]
            title = sys.argv[4]
            grid_size = int(sys.argv[5])
            barriers = json.loads(sys.argv[6]) if len(sys.argv) > 6 else None
            if barriers and len(barriers) == 0:
                barriers = None
            print(render_2d_contour(sources, output_path, title, grid_size, barriers))

        elif mode == "3d":
            if len(sys.argv) < 6:
                print("ERROR: 3d memerlukan: sources_json output_path title grid_size")
                sys.exit(1)
            sources = json.loads(sys.argv[2])
            output_path = sys.argv[3]
            title = sys.argv[4]
            grid_size = int(sys.argv[5])
            print(render_3d_surface(sources, output_path, title, grid_size))

        else:
            print(f"ERROR: Mode '{mode}' tidak dikenal. Gunakan: 2d, 3d")
            sys.exit(1)

    except json.JSONDecodeError as e:
        print(f"ERROR: Gagal parsing JSON: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"ERROR: {e}")
        sys.exit(1)
