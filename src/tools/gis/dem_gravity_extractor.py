#!/usr/bin/env python3
"""Build a downhill candidate network from a DEM and tabular graph inputs."""

import csv
import json
import math
import os
import sys

import numpy as np
import rasterio
from rasterio.warp import transform


def _read_nodes(dataset, nodes_csv):
    nodes = {}
    with open(nodes_csv, newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            node_id = row.get("id", "").strip()
            if not node_id or node_id in nodes:
                raise ValueError("nodes CSV requires unique non-empty id values")
            lon = float(row["lon"])
            lat = float(row["lat"])
            xs, ys = transform("EPSG:4326", dataset.crs, [lon], [lat])
            row_idx, col_idx = dataset.index(xs[0], ys[0])
            if not (0 <= row_idx < dataset.height and 0 <= col_idx < dataset.width):
                raise ValueError(f"node {node_id} is outside the DEM")
            value = dataset.read(1, window=((row_idx, row_idx + 1), (col_idx, col_idx + 1)))[0, 0]
            if dataset.nodata is not None and value == dataset.nodata:
                raise ValueError(f"node {node_id} falls on DEM nodata")
            if not math.isfinite(float(value)):
                raise ValueError(f"node {node_id} has a non-finite DEM elevation")
            nodes[node_id] = {"lon": lon, "lat": lat, "z": float(value), "name": row.get("name", "")}
    return nodes


def build_gravity_network(dem_path, nodes_csv, edges_csv, output_edges_csv):
    if os.path.exists(output_edges_csv):
        raise ValueError(f"refusing to overwrite existing output: {output_edges_csv}")
    dataset_parent = os.path.dirname(os.path.abspath(output_edges_csv))
    if not dataset_parent:
        raise ValueError("output parent is invalid")
    os.makedirs(dataset_parent, exist_ok=True)

    with rasterio.open(dem_path) as dataset:
        if dataset.crs is None:
            raise ValueError("DEM CRS is required")
        nodes = _read_nodes(dataset, nodes_csv)

    directed_edges = []
    with open(edges_csv, newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            source = row.get("source", "").strip()
            target = row.get("target", "").strip()
            distance_km = float(row["weight"])
            if source not in nodes or target not in nodes:
                raise ValueError(f"edge references an unknown node: {source}->{target}")
            if not math.isfinite(distance_km) or distance_km <= 0:
                raise ValueError("edge weight must be a finite positive distance in km")
            delta_z = nodes[source]["z"] - nodes[target]["z"]
            if delta_z <= 0.1:
                continue
            distance_m = distance_km * 1000.0
            slope = delta_z / distance_m
            velocity = math.sqrt(slope + 0.0001) / 0.05
            cost = math.hypot(distance_m, delta_z) / 1000.0 / velocity
            directed_edges.append({"source": source, "target": target, "weight": round(cost, 4), "delta_z_m": round(delta_z, 2)})

    nodes_output = os.path.splitext(nodes_csv)[0] + "_3d.csv"
    if os.path.exists(nodes_output):
        raise ValueError(f"refusing to overwrite existing output: {nodes_output}")
    with open(output_edges_csv, "w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=["source", "target", "weight", "delta_z_m"])
        writer.writeheader()
        writer.writerows(directed_edges)
    with open(nodes_output, "w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=["id", "name", "lon", "lat", "z_elevation"])
        writer.writeheader()
        for node_id, node in nodes.items():
            writer.writerow({"id": node_id, "name": node["name"], "lon": node["lon"], "lat": node["lat"], "z_elevation": node["z"]})

    return {"status": "ok", "output_edges_csv": output_edges_csv, "nodes_3d_csv": nodes_output, "nodes": len(nodes), "downhill_edges": len(directed_edges), "limitations": ["Candidate downhill routing only; not a hydraulic solver.", "Manning roughness surrogate is fixed at 0.05 and requires calibration."]}


def main(argv):
    if len(argv) != 5:
        print(json.dumps({"status": "invalid_request", "error": "usage: dem_gravity_extractor.py DEM NODES_CSV EDGES_CSV OUTPUT_CSV"}))
        return 2
    try:
        print(json.dumps(build_gravity_network(*argv[1:])))
        return 0
    except (OSError, ValueError, KeyError, rasterio.errors.RasterioIOError) as error:
        print(json.dumps({"status": "error", "error": str(error)}))
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
