"""Render hydrologic hydrographs and hydraulic depth histories."""

import csv
import json
import subprocess
from pathlib import Path

import imageio.v2 as imageio
import matplotlib.pyplot as plt
import numpy as np


def load_hydrograph_csv(path, value_column=None):
    """Load time labels and one numeric output column from Wflow CSV."""
    with Path(path).open(newline="") as stream:
        rows = list(csv.DictReader(stream))
    if not rows:
        raise ValueError("hydrograph CSV has no data rows")
    columns = list(rows[0])
    value_column = value_column or next((c for c in columns if c != "time"), None)
    if not value_column or value_column not in columns:
        raise ValueError("hydrograph CSV has no requested numeric value column")
    labels = [row["time"] for row in rows]
    values = [float(row[value_column]) for row in rows]
    return labels, values


def load_depth_history(path):
    """Load solver history JSON and retain explicit grid metadata."""
    data = json.loads(Path(path).read_text())
    if "snapshots" not in data or "grid" not in data:
        raise ValueError("depth history requires grid and snapshots")
    return data["snapshots"], int(data["grid"]["nx"]), int(data["grid"]["ny"])


def depth_snapshot_frames(history, nx, ny):
    """Convert x-major flat solver fields into frame matrices [x][y]."""
    expected = nx * ny
    frames = []
    for snapshot in history:
        values = snapshot["depth_grid_m"]
        if len(values) != expected:
            raise ValueError(f"depth snapshot has {len(values)} cells, expected {expected}")
        matrix = [values[x * ny:(x + 1) * ny] for x in range(nx)]
        frames.append({
            "time_s": float(snapshot["time_s"]),
            "depth_grid_m": matrix,
            "volume_m3": float(snapshot.get("volume_m3", 0.0)),
        })
    if not frames:
        raise ValueError("depth history has no snapshots")
    return frames


def _save_frames(frames, output_path, fps=2):
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    suffix = output_path.suffix.lower()
    if suffix not in {".gif", ".mp4"}:
        raise ValueError("animation output must end in .gif or .mp4")
    if suffix == ".gif":
        imageio.mimsave(output_path, frames, duration=1 / fps, loop=0)
    else:
        height, width, channels = frames[0].shape
        if channels != 3:
            raise ValueError("animation frames must be RGB")
        command = [
            "ffmpeg", "-y", "-loglevel", "error",
            "-f", "rawvideo", "-vcodec", "rawvideo",
            "-s", f"{width}x{height}", "-pix_fmt", "rgb24",
            "-r", str(fps), "-i", "-", "-an",
            "-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2",
            "-c:v", "libx264", "-pix_fmt", "yuv420p", str(output_path),
        ]
        process = subprocess.Popen(command, stdin=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            for frame in frames:
                process.stdin.write(np.asarray(frame, dtype=np.uint8).tobytes())
            process.stdin.close()
        except Exception:
            process.kill()
            process.wait()
            raise
        stderr = process.stderr.read().decode(errors="replace")
        return_code = process.wait()
        if return_code != 0:
            raise RuntimeError(f"ffmpeg failed with exit code {return_code}: {stderr[:500]}")
    return output_path


def render_hydrograph(labels, values, output_path, title="Hydrologic response", ylabel="Discharge / response", fps=2):
    """Render a progressive hydrograph animation."""
    if len(labels) != len(values) or not values:
        raise ValueError("labels and values must be non-empty and equal length")
    values = [float(value) for value in values]
    ymin = min(0.0, min(values))
    ymax = max(1.0, max(values))
    frames = []
    for index in range(len(values)):
        fig, ax = plt.subplots(figsize=(10, 5), dpi=110)
        ax.plot(range(index + 1), values[:index + 1], color="#1769aa", linewidth=2.5, marker="o")
        ax.scatter([index], [values[index]], color="#d1495b", zorder=3)
        ax.set_xlim(-0.5, len(values) - 0.5)
        ax.set_ylim(ymin * 1.1, ymax * 1.1 if ymax else 1.0)
        ax.set_xticks(range(len(labels)))
        ax.set_xticklabels(labels, rotation=35, ha="right", fontsize=8)
        ax.set_title(f"{title}\nFrame {index + 1}/{len(values)} | {labels[index]}")
        ax.set_ylabel(ylabel)
        ax.grid(alpha=0.25)
        fig.tight_layout()
        fig.canvas.draw()
        frames.append(np.asarray(fig.canvas.buffer_rgba())[:, :, :3].copy())
        plt.close(fig)
    return _save_frames(frames, output_path, fps=fps)


def render_depth_history(history, nx, ny, output_path, title="Hydraulic flood depth", fps=2):
    """Render solver depth snapshots as a fixed-scale map animation."""
    frames_data = depth_snapshot_frames(history, nx, ny)
    max_depth = max(max(max(row) for row in frame["depth_grid_m"]) for frame in frames_data)
    vmax = max(0.01, float(max_depth))
    frames = []
    for index, frame in enumerate(frames_data):
        matrix = np.asarray(frame["depth_grid_m"], dtype=float).T
        fig, ax = plt.subplots(figsize=(7, 6), dpi=110)
        image = ax.imshow(matrix, origin="lower", cmap="Blues", vmin=0.0, vmax=vmax)
        ax.set_title(f"{title}\nt={frame['time_s']:.1f} s | volume={frame['volume_m3']:.1f} m3 | frame {index + 1}/{len(frames_data)}")
        ax.set_xlabel("x cell")
        ax.set_ylabel("y cell")
        fig.colorbar(image, ax=ax, label="Depth (m)")
        fig.tight_layout()
        fig.canvas.draw()
        frames.append(np.asarray(fig.canvas.buffer_rgba())[:, :, :3].copy())
        plt.close(fig)
    return _save_frames(frames, output_path, fps=fps)
