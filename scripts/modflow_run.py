#!/usr/bin/env python3
"""Execute a real MODFLOW 6 groundwater flow model via FloPy.

Reads one JSON request on stdin, writes exactly one line of JSON to stdout.

UNITS ARE EXPLICIT AND FIXED: length = meters, time = days. Hydraulic
conductivity is m/day, recharge is mm/yr (converted internally), pumping is
m3/day. The previous Rust-embedded script mixed m/s and m/day, which silently
scaled conductivity by ~10^5.

Request:

    {"nlay": 2, "nrow": 20, "ncol": 20, "cell_size_m": 100.0,
     "top_m": 50.0, "layer_bottoms_m": [30.0, 0.0],
     "hk_m_day": 10.0, "vk_m_day": 1.0,
     "sy": 0.15, "ss_per_m": 1e-5,
     "initial_head_m": 45.0, "boundary_head_m": 45.0,
     "recharge_mm_yr": 1800.0,
     "wells": [{"layer": 1, "row": 10, "col": 10, "rate_m3_day": 2000.0}],
     "steady_state": true, "duration_days": 365.0,
     "mass_tolerance_pct": 1.0}

Well `rate_m3_day` is a positive EXTRACTION rate; it is negated for MODFLOW.
Indices are 1-based in the request and converted internally.

Contract (single line on stdout):

    {"status":"ok","mf6_version":"...","converged":true,
     "heads":{...},"budget":{...},"gate":{...},"wells":[...]}

Four honesty guards travel with the result:

  * `converged` — if MODFLOW did not converge the heads are meaningless, so
    nothing downstream may treat them as a prediction.
  * `gate.percent_discrepancy` — MODFLOW's own volumetric budget error. This is
    the groundwater equivalent of the SWMM mass-balance gate.
  * `dry_cell_count` — a dry or inactive cell carries a +/-1e30 sentinel head.
    Averaged in blindly it destroys every head statistic, so sentinels are
    excluded and counted.
  * `boundary_inflow_fraction` — when constant-head boundaries supply most of
    the water, drawdown is controlled by where the modeller drew the boundary
    rather than by the aquifer. The result is then a boundary artifact.

Exit codes: 0 ok, 1 runtime error, 2 invalid request.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
import tempfile
import traceback

INVALID_REQUEST = 2
RUNTIME_ERROR = 1

# MODFLOW writes this magnitude for dry (HDRY) and inactive (HNOFLO) cells.
SENTINEL_MAGNITUDE = 1e29

DEFAULT_MASS_TOLERANCE_PCT = 1.0
# Above this share of inflow from constant-head boundaries the solution is
# boundary-controlled rather than aquifer-controlled.
BOUNDARY_DOMINANCE_FRACTION = 0.5
# A well must actually deliver this share of its requested rate. MODFLOW shuts
# off a well whose cell goes dry, and the budget then balances around a pump
# that extracted nothing.
WELL_DELIVERY_TOLERANCE = 0.99

MAX_CELLS = 2_000_000


def emit(payload: dict) -> None:
    sys.stdout.write(json.dumps(payload, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def fail_invalid(message: str):
    emit({"status": "invalid_request", "error": message})
    sys.exit(INVALID_REQUEST)


def fail_error(message: str):
    emit({"status": "error", "error": message})
    sys.exit(RUNTIME_ERROR)


def finite(value, default: float = 0.0) -> float:
    try:
        number = float(value)
    except (TypeError, ValueError):
        return default
    if number != number or number in (float("inf"), float("-inf")):
        return default
    return number


def require_positive(request: dict, key: str) -> float:
    value = finite(request.get(key), float("nan"))
    if not value > 0.0:
        fail_invalid("{} must be a positive finite number".format(key))
    return value


def validate(request: dict) -> dict:
    nlay = int(request.get("nlay", 0))
    nrow = int(request.get("nrow", 0))
    ncol = int(request.get("ncol", 0))
    if nlay < 1 or nrow < 3 or ncol < 3:
        fail_invalid("grid must have nlay >= 1, nrow >= 3, ncol >= 3")
    if nlay * nrow * ncol > MAX_CELLS:
        fail_invalid("grid exceeds {} cells".format(MAX_CELLS))

    cell_size = require_positive(request, "cell_size_m")
    hk = require_positive(request, "hk_m_day")
    vk = require_positive(request, "vk_m_day")

    top = finite(request.get("top_m"), float("nan"))
    if top != top:
        fail_invalid("top_m must be a finite number")

    bottoms = request.get("layer_bottoms_m")
    if not isinstance(bottoms, list) or len(bottoms) != nlay:
        fail_invalid("layer_bottoms_m must be a list with exactly nlay entries")
    bottoms = [finite(b, float("nan")) for b in bottoms]
    if any(b != b for b in bottoms):
        fail_invalid("layer_bottoms_m must contain only finite numbers")
    previous = top
    for index, bottom in enumerate(bottoms):
        if bottom >= previous:
            fail_invalid(
                "layer {} bottom ({}) must be below the surface above it ({})".format(
                    index + 1, bottom, previous
                )
            )
        previous = bottom

    sy = finite(request.get("sy"), float("nan"))
    if not 0.0 < sy < 1.0:
        fail_invalid("sy must be between 0 and 1")
    ss = finite(request.get("ss_per_m"), float("nan"))
    if not 0.0 <= ss < 1.0:
        fail_invalid("ss_per_m must be between 0 and 1")

    initial_head = finite(request.get("initial_head_m"), float("nan"))
    boundary_head = finite(request.get("boundary_head_m"), float("nan"))
    if initial_head != initial_head or boundary_head != boundary_head:
        fail_invalid("initial_head_m and boundary_head_m must be finite")
    if initial_head <= bottoms[-1] or boundary_head <= bottoms[-1]:
        fail_invalid("heads must be above the deepest layer bottom")

    recharge_mm_yr = finite(request.get("recharge_mm_yr"), 0.0)
    if recharge_mm_yr < 0.0:
        fail_invalid("recharge_mm_yr must not be negative")

    wells = request.get("wells") or []
    if not isinstance(wells, list):
        fail_invalid("wells must be a list")
    parsed_wells = []
    for index, well in enumerate(wells):
        if not isinstance(well, dict):
            fail_invalid("wells[{}] must be an object".format(index))
        layer = int(well.get("layer", 0))
        row = int(well.get("row", 0))
        col = int(well.get("col", 0))
        if not (1 <= layer <= nlay and 1 <= row <= nrow and 1 <= col <= ncol):
            fail_invalid(
                "wells[{}] index out of range (1-based, expected layer<={} row<={} col<={})".format(
                    index, nlay, nrow, ncol
                )
            )
        rate = finite(well.get("rate_m3_day"), float("nan"))
        if rate != rate or rate < 0.0:
            fail_invalid("wells[{}].rate_m3_day must be a non-negative extraction rate".format(index))
        parsed_wells.append({"layer": layer, "row": row, "col": col, "rate_m3_day": rate})

    steady_state = bool(request.get("steady_state", True))
    duration_days = finite(request.get("duration_days"), 1.0)
    if not steady_state and duration_days <= 0.0:
        fail_invalid("duration_days must be positive for a transient run")

    tolerance = finite(request.get("mass_tolerance_pct"), DEFAULT_MASS_TOLERANCE_PCT)
    if not 0.0 < tolerance <= 100.0:
        fail_invalid("mass_tolerance_pct must be between 0 and 100")

    return {
        "nlay": nlay,
        "nrow": nrow,
        "ncol": ncol,
        "cell_size_m": cell_size,
        "top_m": top,
        "layer_bottoms_m": bottoms,
        "hk_m_day": hk,
        "vk_m_day": vk,
        "sy": sy,
        "ss_per_m": ss,
        "initial_head_m": initial_head,
        "boundary_head_m": boundary_head,
        "recharge_mm_yr": recharge_mm_yr,
        "wells": parsed_wells,
        "steady_state": steady_state,
        "duration_days": duration_days if not steady_state else 1.0,
        "mass_tolerance_pct": tolerance,
    }


def mf6_version(exe: str) -> str:
    """Ask the executable for its version.

    Scraping the run log is unreliable: MODFLOW 6.7.0 does not echo a line
    containing both "MODFLOW 6" and "VERSION", so the log-scan approach silently
    reported "unknown" for every run and the provenance was useless.
    """
    import subprocess

    try:
        completed = subprocess.run(
            [exe, "-v"],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
    except (OSError, subprocess.SubprocessError):
        return "unknown"
    output = "{}\n{}".format(completed.stdout or "", completed.stderr or "")
    for line in output.splitlines():
        cleaned = line.strip()
        if cleaned.lower().startswith("mf6:"):
            return cleaned
    return "unknown"


def build_and_run(spec: dict, workspace: str) -> dict:
    import numpy as np
    import flopy as fp

    exe = shutil.which("mf6") or "mf6"
    name = "gwf"

    sim = fp.mf6.MFSimulation(sim_name=name, sim_ws=workspace, exe_name=exe)
    fp.mf6.ModflowTdis(
        sim,
        nper=1,
        perioddata=[(spec["duration_days"], 1, 1.0)],
        time_units="days",
    )
    fp.mf6.ModflowIms(sim, print_option="SUMMARY", complexity="MODERATE")

    gwf = fp.mf6.ModflowGwf(sim, modelname=name, save_flows=True)
    fp.mf6.ModflowGwfdis(
        gwf,
        nlay=spec["nlay"],
        nrow=spec["nrow"],
        ncol=spec["ncol"],
        delr=spec["cell_size_m"],
        delc=spec["cell_size_m"],
        top=spec["top_m"],
        botm=spec["layer_bottoms_m"],
        length_units="meters",
    )
    fp.mf6.ModflowGwfnpf(
        gwf,
        k=spec["hk_m_day"],
        k33=spec["vk_m_day"],
        icelltype=1,
        save_flows=True,
    )
    fp.mf6.ModflowGwfic(gwf, strt=spec["initial_head_m"])
    fp.mf6.ModflowGwfsto(
        gwf,
        sy=spec["sy"],
        ss=spec["ss_per_m"],
        iconvert=1,
        steady_state={0: spec["steady_state"]},
        transient={0: not spec["steady_state"]},
    )

    # Constant head on the left and right edges of the top layer.
    chd = []
    for row in range(spec["nrow"]):
        chd.append([(0, row, 0), spec["boundary_head_m"]])
        chd.append([(0, row, spec["ncol"] - 1), spec["boundary_head_m"]])
    fp.mf6.ModflowGwfchd(gwf, stress_period_data=chd, save_flows=True)

    # mm/yr -> m/day. The old script divided by 365 after /1000 and called the
    # result m/s, which is wrong by ~10^5.
    if spec["recharge_mm_yr"] > 0.0:
        recharge_m_day = spec["recharge_mm_yr"] / 1000.0 / 365.25
        fp.mf6.ModflowGwfrcha(gwf, recharge=recharge_m_day, save_flows=True)

    if spec["wells"]:
        well_records = [
            [(w["layer"] - 1, w["row"] - 1, w["col"] - 1), -w["rate_m3_day"]]
            for w in spec["wells"]
        ]
        fp.mf6.ModflowGwfwel(gwf, stress_period_data=well_records, save_flows=True)

    fp.mf6.ModflowGwfoc(
        gwf,
        head_filerecord="{}.hds".format(name),
        budget_filerecord="{}.cbc".format(name),
        saverecord=[("HEAD", "ALL"), ("BUDGET", "ALL")],
    )

    sim.write_simulation(silent=True)
    converged, buff = sim.run_simulation(silent=True)

    version = mf6_version(exe)

    heads_raw = None
    try:
        import flopy.utils.binaryfile as bf

        heads_raw = bf.HeadFile(os.path.join(workspace, "{}.hds".format(name))).get_data()
    except Exception as error:  # noqa: BLE001 - reported, not swallowed
        if converged:
            fail_error("MODFLOW converged but heads could not be read: {}".format(error))

    head_block = {
        "available": heads_raw is not None,
        "dry_cell_count": 0,
        "active_cell_count": 0,
    }
    well_results = []
    if heads_raw is not None:
        heads = np.array(heads_raw, dtype=float)
        sentinel = np.abs(heads) >= SENTINEL_MAGNITUDE
        dry_count = int(np.count_nonzero(sentinel))
        active = heads[~sentinel]
        head_block["dry_cell_count"] = dry_count
        head_block["active_cell_count"] = int(active.size)
        head_block["grid_shape"] = [int(v) for v in heads.shape]
        if active.size:
            head_block["min_head_m"] = float(np.min(active))
            head_block["max_head_m"] = float(np.max(active))
            head_block["mean_head_m"] = float(np.mean(active))
        else:
            head_block["min_head_m"] = None
            head_block["max_head_m"] = None
            head_block["mean_head_m"] = None

        for well in spec["wells"]:
            k, i, j = well["layer"] - 1, well["row"] - 1, well["col"] - 1
            value = float(heads[k, i, j])
            is_dry = abs(value) >= SENTINEL_MAGNITUDE
            well_results.append(
                {
                    "layer": well["layer"],
                    "row": well["row"],
                    "col": well["col"],
                    "rate_m3_day": well["rate_m3_day"],
                    "head_m": None if is_dry else value,
                    "drawdown_m": None if is_dry else spec["initial_head_m"] - value,
                    "cell_is_dry": is_dry,
                }
            )

    budget_block = {"available": False}
    gate = {
        "percent_discrepancy": None,
        "tolerance_pct": spec["mass_tolerance_pct"],
        "gate_passed": False,
        "boundary_inflow_fraction": None,
        "boundary_controlled": None,
        "requested_extraction_m3": None,
        "delivered_extraction_m3": None,
        "extraction_delivery_fraction": None,
        "wells_curtailed": None,
    }
    try:
        listing = gwf.output.list()
        incremental, cumulative = listing.get_budget()
        terms = {}
        for field in cumulative.dtype.names:
            if field in ("totim", "time_step", "stress_period", "tslen"):
                continue
            terms[field] = float(cumulative[field][-1])
        discrepancy = float(cumulative["PERCENT_DISCREPANCY"][-1])
        total_in = terms.get("TOTAL_IN", 0.0)
        chd_in = terms.get("CHD_IN", 0.0)
        fraction = (chd_in / total_in) if total_in > 0.0 else None

        # A well in a cell that goes dry is switched off by MODFLOW. The budget
        # then balances perfectly around a pump that extracted nothing, so
        # `converged` and the discrepancy gate both pass while the scenario the
        # caller asked for never happened. Compare requested against delivered.
        requested = sum(w["rate_m3_day"] for w in spec["wells"]) * spec["duration_days"]
        delivered = terms.get("WEL_OUT", 0.0)
        delivery = None
        curtailed = None
        if requested > 0.0:
            delivery = delivered / requested
            curtailed = delivery < WELL_DELIVERY_TOLERANCE

        budget_block = {"available": True, "cumulative_m3": terms}
        gate["percent_discrepancy"] = discrepancy
        gate["boundary_inflow_fraction"] = fraction
        gate["boundary_controlled"] = (
            None if fraction is None else fraction > BOUNDARY_DOMINANCE_FRACTION
        )
        gate["requested_extraction_m3"] = requested
        gate["delivered_extraction_m3"] = delivered
        gate["extraction_delivery_fraction"] = delivery
        gate["wells_curtailed"] = curtailed
        gate["gate_passed"] = (
            converged
            and abs(discrepancy) <= spec["mass_tolerance_pct"]
            and not bool(curtailed)
        )
    except Exception as error:  # noqa: BLE001
        budget_block = {"available": False, "error": str(error)}

    return {
        "status": "ok",
        "mf6_version": version,
        "mf6_executable": exe,
        "converged": bool(converged),
        "units": {"length": "meters", "time": "days"},
        "steady_state": spec["steady_state"],
        "duration_days": spec["duration_days"],
        "recharge_m_day": spec["recharge_mm_yr"] / 1000.0 / 365.25,
        "heads": head_block,
        "wells": well_results,
        "budget": budget_block,
        "gate": gate,
    }


def main(argv: list) -> int:
    parser = argparse.ArgumentParser(
        description="Run a MODFLOW 6 groundwater model from a JSON request on stdin.",
    )
    parser.add_argument("--input", help="path to a JSON request file; omit to read stdin")
    parser.add_argument(
        "--workspace",
        help="directory for MODFLOW input/output; a temporary directory is used when omitted",
    )
    try:
        args = parser.parse_args(argv)
    except SystemExit:
        emit({"status": "invalid_request", "error": "bad arguments"})
        return INVALID_REQUEST

    try:
        if args.input:
            if not os.path.isfile(args.input):
                fail_invalid("input file not found: {}".format(args.input))
            with open(args.input, "r", encoding="utf-8") as handle:
                payload = handle.read()
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

    spec = validate(request)

    temporary = None
    try:
        if args.workspace:
            workspace = args.workspace
            os.makedirs(workspace, exist_ok=True)
        else:
            temporary = tempfile.mkdtemp(prefix="modflow_run_")
            workspace = temporary
        result = build_and_run(spec, workspace)
    except SystemExit:
        raise
    except Exception:
        fail_error(traceback.format_exc(limit=20))
    finally:
        if temporary and os.path.isdir(temporary):
            shutil.rmtree(temporary, ignore_errors=True)

    emit(result)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
