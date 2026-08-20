#!/usr/bin/env python3
"""Run an EPA SWMM model with pyswmm and emit one line of JSON to stdout.

Contract (single line on stdout):

    {"status":"ok","pyswmm_version":"2.1.0","inp_sha256":"<64 hex>",
     "routing":{"external_inflow_m3":...,"flooding_m3":...,"outflow_m3":...,
                "initial_storage_m3":...,"final_storage_m3":...,
                "routing_error_pct":...},
     "nodes":[{"node_id":"J1","flooding_volume_m3":...,
               "peak_flooding_rate_m3s":...,"flooding_duration_hr":...,
               "max_depth_m":...,"invert_elevation_m":...}]}

Exit codes: 0 ok, 1 runtime error, 2 invalid request.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import traceback

INVALID_REQUEST = 2
RUNTIME_ERROR = 1


def emit(payload: dict) -> None:
    """Write exactly one line of compact JSON to stdout."""
    sys.stdout.write(json.dumps(payload, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def fail_invalid(message: str) -> "NoReturn":  # type: ignore[name-defined]
    emit({"status": "invalid_request", "error": message})
    sys.exit(INVALID_REQUEST)


def fail_error(message: str) -> "NoReturn":  # type: ignore[name-defined]
    emit({"status": "error", "error": message})
    sys.exit(RUNTIME_ERROR)


def sha256_of(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def as_float(value) -> float:
    try:
        number = float(value)
    except (TypeError, ValueError):
        return 0.0
    # SWMM occasionally reports non-finite values for degenerate models.
    if number != number or number in (float("inf"), float("-inf")):
        return 0.0
    return number


def validate_inp(path: str) -> str:
    if not path.lower().endswith(".inp"):
        fail_invalid("inp path must end in .inp")
    resolved = os.path.abspath(path)
    if not os.path.exists(resolved):
        fail_invalid("inp path does not exist: {}".format(resolved))
    if not os.path.isfile(resolved):
        fail_invalid("inp path is not a regular file: {}".format(resolved))
    return resolved


def collect_nodes(sim_nodes, max_nodes: int) -> list:
    """Return flooded nodes only, capped at max_nodes."""
    results = []
    for node in sim_nodes:
        stats = node.statistics or {}
        flooding_volume = as_float(stats.get("flooding_volume", 0.0))
        if flooding_volume <= 0.0:
            continue
        results.append(
            {
                "node_id": str(node.nodeid),
                "flooding_volume_m3": flooding_volume,
                "peak_flooding_rate_m3s": as_float(stats.get("peak_flooding_rate", 0.0)),
                "flooding_duration_hr": as_float(stats.get("flooding_duration", 0.0)),
                "max_depth_m": as_float(getattr(node, "full_depth", 0.0)),
                "invert_elevation_m": as_float(getattr(node, "invert_elevation", 0.0)),
            }
        )
        if len(results) >= max_nodes:
            break
    return results


def run(inp_path: str, max_nodes: int) -> dict:
    import pyswmm
    from pyswmm import Nodes, Simulation, SystemStats

    version = getattr(pyswmm, "__version__", "unknown")

    with Simulation(inp_path) as sim:
        # SystemStats must be constructed before the simulation is stepped.
        system_stats = SystemStats(sim)
        nodes = Nodes(sim)
        for _ in sim:
            pass
        # Statistics are only complete after the routing loop finishes.
        routing = system_stats.routing_stats or {}
        node_results = collect_nodes(nodes, max_nodes)

    return {
        "status": "ok",
        "pyswmm_version": version,
        "inp_sha256": sha256_of(inp_path),
        "routing": {
            "external_inflow_m3": as_float(routing.get("external_inflow", 0.0)),
            "flooding_m3": as_float(routing.get("flooding", 0.0)),
            "outflow_m3": as_float(routing.get("outflow", 0.0)),
            "initial_storage_m3": as_float(routing.get("initial_storage", 0.0)),
            "final_storage_m3": as_float(routing.get("final_storage", 0.0)),
            "routing_error_pct": as_float(routing.get("routing_error", 0.0)),
        },
        "nodes": node_results,
    }


def main(argv: list) -> int:
    parser = argparse.ArgumentParser(
        description="Run an EPA SWMM model and emit JSON routing/node statistics.",
    )
    parser.add_argument("--inp", required=True, help="path to the SWMM .inp model file")
    parser.add_argument(
        "--max-nodes",
        type=int,
        default=5000,
        help="maximum number of flooded nodes to report (default: 5000)",
    )
    try:
        args = parser.parse_args(argv)
    except SystemExit:
        # argparse already wrote usage to stderr; keep stdout machine-readable.
        emit({"status": "invalid_request", "error": "bad arguments"})
        return INVALID_REQUEST

    if args.max_nodes <= 0:
        fail_invalid("--max-nodes must be a positive integer")

    inp_path = validate_inp(args.inp)

    try:
        payload = run(inp_path, args.max_nodes)
    except Exception:
        fail_error(traceback.format_exc(limit=20))

    emit(payload)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
