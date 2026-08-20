#!/usr/bin/env python3
"""1D advective-dispersive REACTIVE TRANSPORT via PHREEQC TRANSPORT.

Reads one JSON request on stdin, writes exactly one line of JSON to stdout.

This couples flow with geochemistry: an influent water is advected through a
column of cells, each of which re-equilibrates with the mineral assemblage it
contains. It answers the question the standalone tools cannot -- WHERE along a
flow path, and after how many pore volumes, does a reactive barrier stop
working.

`phreeqc_run.py` equilibrates one batch. `pyrite_kinetics.py` adds time to one
batch. This adds space and flow.

UNITS: cell lengths and dispersivity in metres, time step in seconds.

Request:

    {"cells": 10, "cell_length_m": 0.2, "shifts": 60, "time_step_s": 3600,
     "dispersivity_m": 0.02,
     "influent": {"pH": 2.5, "Fe(3)": 30.0, "S(6)": 120.0},
     "pore_water": {"pH": 7.0, "Ca": 1.0, "C(4)": 1.0},
     "units": "mmol",
     "reactive_phases": [{"phase": "Calcite", "moles": 0.02}],
     "tracked_elements": ["Fe", "S(6)", "Ca"],
     "punch_frequency": 5}

Contract: {"status":"ok","outlet_series":[...],"guards":{...},...}

Four honesty guards:

  * `numerical_dispersion_dominates` -- PHREEQC's mixing-cell scheme carries
    numerical dispersion of about cell_length/2. When the physical dispersivity
    is smaller than that, the simulated front spreading is a GRID ARTIFACT, not
    transport physics. Reported as the grid Peclet number cell_length/alpha.
  * `breakthrough_reached` -- if the influent front never arrives at the outlet
    within the simulated window, a clean outlet means "simulation too short",
    not "the barrier works". Guarded by pore volumes flushed = shifts / cells.
  * `buffer_exhausted` -- the reactive mineral was consumed at the outlet, so
    the barrier has failed. This is the physically real result and the whole
    point of running the model.
  * `equilibrium_assumed_at_each_cell` -- always true and always reported: each
    cell reaches full thermodynamic equilibrium every shift, so kinetic
    limitation and preferential flow are absent by construction.

Exit codes: 0 ok, 1 runtime error, 2 invalid request.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import traceback
from pathlib import Path

INVALID_REQUEST = 2
RUNTIME_ERROR = 1

DEFAULT_DATABASE = "wateq4f_PWN_repaired.dat"
DEFAULT_DATABASE_DIR = Path(__file__).resolve().parent.parent / "resources" / "phreeqc"

NON_ELEMENT_KEYS = {"pH", "pe", "temp", "temperature", "units", "density", "water", "redox"}

MAX_CELLS = 200
MAX_SHIFTS = 5000

# The mixing-cell scheme's numerical dispersion is about cell_length/2, so a
# grid Peclet number above 2 means numerical dispersion exceeds the physical
# dispersivity the caller asked for.
GRID_PECLET_LIMIT = 2.0

# Fraction of the influent concentration that counts as arrival at the outlet.
BREAKTHROUGH_FRACTION = 0.05

# Below this fraction of its initial moles the reactive phase is spent.
BUFFER_EXHAUSTED_FRACTION = 0.01


def emit(payload: dict) -> None:
    sys.stdout.write(json.dumps(payload, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def fail_invalid(message: str):
    emit({"status": "invalid_request", "error": message})
    sys.exit(INVALID_REQUEST)


def fail_error(message: str):
    emit({"status": "error", "error": message})
    sys.exit(RUNTIME_ERROR)


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def finite(value, default: float = 0.0) -> float:
    try:
        number = float(value)
    except (TypeError, ValueError):
        return default
    if number != number or number in (float("inf"), float("-inf")):
        return default
    return number


def base_element(key: str) -> str:
    return key.split("(", 1)[0].strip()


def validate(request: dict) -> dict:
    cells = int(request.get("cells", 0))
    if not 2 <= cells <= MAX_CELLS:
        fail_invalid("cells must be between 2 and {}".format(MAX_CELLS))

    shifts = int(request.get("shifts", 0))
    if not 1 <= shifts <= MAX_SHIFTS:
        fail_invalid("shifts must be between 1 and {}".format(MAX_SHIFTS))

    cell_length = finite(request.get("cell_length_m"), float("nan"))
    if not cell_length > 0.0:
        fail_invalid("cell_length_m must be a positive finite number")

    time_step = finite(request.get("time_step_s"), float("nan"))
    if not time_step > 0.0:
        fail_invalid("time_step_s must be a positive finite number")

    dispersivity = finite(request.get("dispersivity_m"), float("nan"))
    if not dispersivity >= 0.0:
        fail_invalid("dispersivity_m must not be negative")

    influent = request.get("influent")
    if not isinstance(influent, dict) or not influent:
        fail_invalid("influent must be a non-empty object")
    pore_water = request.get("pore_water")
    if not isinstance(pore_water, dict) or not pore_water:
        fail_invalid("pore_water must be a non-empty object")
    for label, block in (("influent", influent), ("pore_water", pore_water)):
        for key, value in block.items():
            if not finite(value, float("nan")) == finite(value, float("nan")):
                fail_invalid("{}[{}] must be finite".format(label, key))

    units = str(request.get("units", "mmol"))
    if units not in {"mmol", "mol", "mg", "umol", "ug"}:
        fail_invalid("units must be one of mmol, mol, mg, umol, ug")

    phases = request.get("reactive_phases") or []
    if not isinstance(phases, list):
        fail_invalid("reactive_phases must be a list")
    parsed_phases = []
    for index, entry in enumerate(phases):
        if not isinstance(entry, dict):
            fail_invalid("reactive_phases[{}] must be an object".format(index))
        phase = str(entry.get("phase", "")).strip()
        if not phase:
            fail_invalid("reactive_phases[{}].phase is required".format(index))
        moles = finite(entry.get("moles"), float("nan"))
        if not moles >= 0.0:
            fail_invalid("reactive_phases[{}].moles must not be negative".format(index))
        parsed_phases.append({"phase": phase, "moles": moles})

    tracked = request.get("tracked_elements")
    if tracked is None:
        candidates = [k for k in influent if k not in NON_ELEMENT_KEYS]
        tracked = sorted({base_element(k) for k in candidates})
    if not isinstance(tracked, list) or not tracked:
        fail_invalid("tracked_elements must be a non-empty list")
    tracked = [str(t).strip() for t in tracked if str(t).strip()]
    if not tracked:
        fail_invalid("tracked_elements must contain at least one element name")

    punch = int(request.get("punch_frequency", 1))
    if punch < 1:
        fail_invalid("punch_frequency must be at least 1")

    database = str(request.get("database", DEFAULT_DATABASE))
    if "/" in database or "\\" in database or database.startswith("."):
        fail_invalid("database must be a bare file name inside the resources directory")

    return {
        "cells": cells,
        "shifts": shifts,
        "cell_length_m": cell_length,
        "time_step_s": time_step,
        "dispersivity_m": dispersivity,
        "influent": influent,
        "pore_water": pore_water,
        "units": units,
        "reactive_phases": parsed_phases,
        "tracked_elements": tracked,
        "punch_frequency": punch,
        "database": database,
        "database_directory": Path(request.get("database_directory", DEFAULT_DATABASE_DIR)),
    }


def solution_block(number: str, composition: dict, units: str) -> str:
    lines = ["SOLUTION {}".format(number), "    units   {}/kgw".format(units)]
    for key, value in composition.items():
        if key == "pH":
            lines.append("    pH      {}".format(finite(value, 7.0)))
        elif key == "pe":
            lines.append("    pe      {}".format(finite(value, 4.0)))
        elif key in ("temp", "temperature"):
            lines.append("    temp    {}".format(finite(value, 25.0)))
        else:
            lines.append("    {:8s}{}".format(key, finite(value)))
    return "\n".join(lines)


def build_input(spec: dict) -> str:
    cells = spec["cells"]
    blocks = [
        solution_block("0", spec["influent"], spec["units"]),
        "END",
        solution_block("1-{}".format(cells), spec["pore_water"], spec["units"]),
        "END",
    ]

    if spec["reactive_phases"]:
        lines = ["EQUILIBRIUM_PHASES 1-{}".format(cells)]
        for entry in spec["reactive_phases"]:
            lines.append("    {}   0.0   {}".format(entry["phase"], entry["moles"]))
        blocks.append("\n".join(lines))
        blocks.append("END")

    punch = ["SELECTED_OUTPUT", "    -reset      false", "    -distance   true", "    -step       true", "    -ph         true"]
    punch.append("    -totals     " + " ".join(spec["tracked_elements"]))
    if spec["reactive_phases"]:
        punch.append(
            "    -equilibrium_phases " + " ".join(e["phase"] for e in spec["reactive_phases"])
        )
    blocks.append("\n".join(punch))

    blocks.append(
        "TRANSPORT\n"
        "    -cells       {cells}\n"
        "    -shifts      {shifts}\n"
        "    -lengths     {cells}*{length}\n"
        "    -dispersivities {cells}*{disp}\n"
        "    -time_step   {dt}\n"
        "    -flow_direction forward\n"
        "    -boundary_conditions flux flux\n"
        "    -punch_cells {cells}\n"
        "    -punch_frequency {punch}\n"
        "END".format(
            cells=cells,
            shifts=spec["shifts"],
            length=spec["cell_length_m"],
            disp=spec["dispersivity_m"],
            dt=spec["time_step_s"],
            punch=spec["punch_frequency"],
        )
    )
    return "\n".join(blocks)


def run(request: dict) -> dict:
    from phreeqpython.viphreeqc import VIPhreeqc

    spec = validate(request)
    database_path = spec["database_directory"] / spec["database"]
    if not database_path.is_file():
        fail_invalid("database not found: {}".format(database_path))

    ip = VIPhreeqc()
    ip.load_database(database_path)
    ip.run_string(build_input(spec))

    columns = [ip.get_selected_output_value(0, c) for c in range(ip.column_count)]
    index = {name: position for position, name in enumerate(columns)}
    if "step" not in index or "pH" not in index:
        fail_error("selected output missing step/pH columns")

    series = []
    for row in range(1, ip.row_count):
        values = [ip.get_selected_output_value(row, c) for c in range(ip.column_count)]
        step = int(finite(values[index["step"]], 0))
        entry = {
            "shift": step,
            "pore_volumes": step / spec["cells"] if spec["cells"] else None,
            "time_days": step * spec["time_step_s"] / 86400.0,
            "distance_m": finite(values[index["dist_x"]]) if "dist_x" in index else None,
            "ph": finite(values[index["pH"]]),
            "elements_mol_kgw": {},
            "phases_mol": {},
        }
        for element in spec["tracked_elements"]:
            key = "{}(mol/kgw)".format(element)
            if key in index:
                entry["elements_mol_kgw"][element] = finite(values[index[key]])
        for phase_entry in spec["reactive_phases"]:
            phase = phase_entry["phase"]
            if phase in index:
                entry["phases_mol"][phase] = finite(values[index[phase]])
        series.append(entry)

    if not series:
        fail_error("transport run produced no output rows")

    return {
        "status": "ok",
        "database": spec["database"],
        "database_sha256": sha256_of(database_path),
        "column_length_m": spec["cells"] * spec["cell_length_m"],
        "cells": spec["cells"],
        "shifts": spec["shifts"],
        "cell_length_m": spec["cell_length_m"],
        "dispersivity_m": spec["dispersivity_m"],
        "time_step_s": spec["time_step_s"],
        "pore_velocity_m_day": spec["cell_length_m"] / spec["time_step_s"] * 86400.0,
        "total_simulated_days": spec["shifts"] * spec["time_step_s"] / 86400.0,
        "tracked_elements": spec["tracked_elements"],
        "reactive_phases": spec["reactive_phases"],
        "outlet_series": series,
        "guards": build_guards(spec, series),
    }


def build_guards(spec: dict, series: list) -> dict:
    final = series[-1]

    # Grid Peclet: numerical dispersion of the mixing-cell scheme is ~dx/2.
    alpha = spec["dispersivity_m"]
    grid_peclet = None if alpha <= 0.0 else spec["cell_length_m"] / alpha
    numerical_dominates = alpha <= 0.0 or grid_peclet > GRID_PECLET_LIMIT

    pore_volumes = spec["shifts"] / spec["cells"]

    # Breakthrough: has any tracked element arrived at the outlet at a
    # meaningful fraction of its influent concentration?
    breakthrough = False
    breakthrough_element = None
    breakthrough_pore_volumes = None
    for element in spec["tracked_elements"]:
        influent_total = 0.0
        for key, value in spec["influent"].items():
            if key in NON_ELEMENT_KEYS:
                continue
            if base_element(key) == element:
                influent_total += finite(value)
        if influent_total <= 0.0:
            continue
        for entry in series:
            observed = entry["elements_mol_kgw"].get(element)
            if observed is None:
                continue
            # Influent units are per the request; compare in molal by converting
            # the request amount with the same factor PHREEQC used.
            scale = {"mol": 1.0, "mmol": 1e-3, "umol": 1e-6}.get(spec["units"])
            if scale is None:
                # Mass units: fall back to a relative rise above the first row.
                baseline = series[0]["elements_mol_kgw"].get(element, 0.0)
                if observed > baseline and baseline >= 0.0 and observed > 0.0:
                    breakthrough = True
                    breakthrough_element = element
                    breakthrough_pore_volumes = entry["pore_volumes"]
                    break
                continue
            if observed >= BREAKTHROUGH_FRACTION * influent_total * scale:
                breakthrough = True
                breakthrough_element = element
                breakthrough_pore_volumes = entry["pore_volumes"]
                break
        if breakthrough:
            break

    # Buffer exhaustion at the outlet cell.
    buffer_exhausted = False
    exhausted_phases = []
    for entry_phase in spec["reactive_phases"]:
        phase = entry_phase["phase"]
        initial = entry_phase["moles"]
        remaining = final["phases_mol"].get(phase)
        if initial > 0.0 and remaining is not None:
            if remaining <= BUFFER_EXHAUSTED_FRACTION * initial:
                buffer_exhausted = True
                exhausted_phases.append(phase)

    return {
        "grid_peclet": grid_peclet,
        "grid_peclet_limit": GRID_PECLET_LIMIT,
        "numerical_dispersion_dominates": bool(numerical_dominates),
        "pore_volumes_flushed": pore_volumes,
        "front_traversed_column": pore_volumes >= 1.0,
        "breakthrough_reached": bool(breakthrough),
        "breakthrough_element": breakthrough_element,
        "breakthrough_pore_volumes": breakthrough_pore_volumes,
        "buffer_exhausted": bool(buffer_exhausted),
        "exhausted_phases": exhausted_phases,
        "outlet_initial_ph": series[0]["ph"],
        "outlet_final_ph": final["ph"],
        "equilibrium_assumed_at_each_cell": True,
    }


def main(argv: list) -> int:
    parser = argparse.ArgumentParser(
        description="Run 1D reactive transport from a JSON request on stdin.",
    )
    parser.add_argument("--input", help="path to a JSON request file; omit to read stdin")
    try:
        args = parser.parse_args(argv)
    except SystemExit:
        emit({"status": "invalid_request", "error": "bad arguments"})
        return INVALID_REQUEST

    try:
        if args.input:
            if not os.path.isfile(args.input):
                fail_invalid("input file not found: {}".format(args.input))
            payload = Path(args.input).read_text(encoding="utf-8")
        else:
            payload = sys.stdin.read()
    except OSError as error:
        fail_invalid("could not read request: {}".format(error))

    try:
        request = json.loads(payload)
    except json.JSONDecodeError as error:
        fail_invalid("request is not valid JSON: {}".format(error))
    if not isinstance(request, dict):
        fail_invalid("request must be a JSON object")

    try:
        result = run(request)
    except SystemExit:
        raise
    except Exception:
        fail_error(traceback.format_exc(limit=20))

    emit(result)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
