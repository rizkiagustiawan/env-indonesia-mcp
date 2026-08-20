#!/usr/bin/env python3
"""Execute a real PHREEQC geochemical speciation / leaching calculation.

Reads one JSON request on stdin, writes exactly one line of JSON to stdout.

Request:

    {"solution": {"pH": 2.8, "Fe(3)": 50.0, "S(6)": 200.0, "Zn": 2.0},
     "units": "mmol",                       # optional, default mmol/kgw
     "temperature_c": 25.0,                 # optional
     "equilibrium_phases": ["Fe(OH)3(a)"],  # optional, equilibrated on the raw solution
     "lime_titration_target_ph": 8.5,       # optional, adds Ca(OH)2 until reached
     "saturation_indices": ["Gypsum"]}      # optional extra phases to report

Contract (single line on stdout):

    {"status":"ok","database":"...","database_sha256":"<64 hex>",
     "raw":{...},"treated":{...}|null,"lime_added_mmol":...,
     "element_recovery":[...],"unsupported_elements":[...]}

The `element_recovery` block is the honesty guard. PHREEQC silently ignores an
element that has no master species in the loaded database: it accepts the input,
reports 0 mg/L, and raises nothing. A caller would read that zero as "this metal
is not mobile" when it actually means "this metal was never modelled". Every
requested element is therefore compared against what the solver reports back,
and any element that vanished is listed in `unsupported_elements`.

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

# AMD / leaching relevant phases reported when present in the loaded database.
DEFAULT_SI_PHASES = [
    "Fe(OH)3(a)",
    "Goethite",
    "Hematite",
    "Jarosite-K",
    "Melanterite",
    "Siderite",
    "Gypsum",
    "Anhydrite",
    "Gibbsite",
    "Alunite",
    "Sphalerite",
    "Smithsonite",
    "Otavite",
    "Cerrusite",
    "Anglesite",
    "Pyrite",
]

# Keys in the solution block that are not elements.
NON_ELEMENT_KEYS = {"pH", "pe", "temp", "temperature", "units", "density", "water", "redox"}

MAX_TITRATION_STEPS = 400
LIME_STEP_MMOL = 5.0

# Bisection refinement after the coarse bracket, and the pH we call "on target".
BISECTION_STEPS = 40
PH_TOLERANCE = 0.05

# PHREEQC returns -999 for a saturation index it could not evaluate.
SI_SENTINEL_THRESHOLD = -998.0

# SI above this counts as supersaturated rather than numerical noise.
SUPERSATURATION_TOLERANCE = 0.05


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


def finite(value) -> float:
    try:
        number = float(value)
    except (TypeError, ValueError):
        return 0.0
    if number != number or number in (float("inf"), float("-inf")):
        return 0.0
    return number


def base_element(key: str) -> str:
    """'Fe(3)' -> 'Fe', 'S(6)' -> 'S'."""
    return key.split("(", 1)[0].strip()


def requested_elements(solution: dict) -> dict:
    """Map base element -> total requested amount, skipping non-element keys."""
    totals: dict = {}
    for key, value in solution.items():
        if key in NON_ELEMENT_KEYS:
            continue
        amount = finite(value)
        if amount <= 0.0:
            continue
        totals[base_element(key)] = totals.get(base_element(key), 0.0) + amount
    return totals


def describe(solution, elements: list, si_phases: list, modelled_phases: list) -> dict:
    """Snapshot a phreeqpython Solution into plain JSON.

    Three PHREEQC behaviours are normalised here, because all three produce
    numbers that look like measurements but mean something else:

    * Specific conductance needs `-dw` diffusion coefficients. Databases that
      omit them (wateq4f is one) return 0.0 uS/cm. For a solution with real
      ionic strength that value is physically impossible, so it is reported as
      null with a reason instead of a plausible-looking zero.
    * A saturation index of -999 is PHREEQC's sentinel for a phase it could not
      evaluate (phase absent, or an element in it has no master species). Left
      as a number it reads as "wildly undersaturated, no risk". Those phases are
      moved into `saturation_indices_not_computed`.
    * A phase with SI > 0 that was NOT listed in `equilibrium_phases` is
      supersaturated and would precipitate in reality, but the solver was never
      told to remove it. The dissolved concentration of the elements it contains
      is therefore an UPPER BOUND, not a prediction. Those phases are listed in
      `supersaturated_but_unmodelled`.
    """
    elements_mg_l = {}
    elements_mmol = {}
    for element in elements:
        elements_mg_l[element] = finite(solution.total(element, units="mg"))
        elements_mmol[element] = finite(solution.total(element, units="mmol"))

    indices = {}
    not_computed = []
    for phase in si_phases:
        try:
            value = solution.si(phase)
        except Exception:
            not_computed.append(phase)
            continue
        if value is None:
            not_computed.append(phase)
            continue
        numeric = finite(value)
        if numeric <= SI_SENTINEL_THRESHOLD:
            not_computed.append(phase)
            continue
        indices[phase] = numeric

    modelled = set(modelled_phases)
    supersaturated = [
        {"phase": phase, "si": si}
        for phase, si in sorted(indices.items(), key=lambda item: -item[1])
        if si > SUPERSATURATION_TOLERANCE and phase not in modelled
    ]

    ionic_strength = finite(solution.I)
    conductance = finite(solution.sc)
    sc_value = conductance if conductance > 0.0 else None
    sc_note = None
    if sc_value is None and ionic_strength > 0.0:
        sc_note = (
            "specific conductance not computable: the loaded database lacks -dw "
            "diffusion coefficients for these species"
        )

    return {
        "ph": finite(solution.pH),
        "pe": finite(solution.pe),
        "sc_us_cm": sc_value,
        "sc_note": sc_note,
        "ionic_strength_mol_kgw": ionic_strength,
        "elements_mg_l": elements_mg_l,
        "elements_mmol": elements_mmol,
        "saturation_indices": indices,
        "saturation_indices_not_computed": not_computed,
        "supersaturated_but_unmodelled": supersaturated,
        "concentrations_are_upper_bounds": bool(supersaturated),
    }


def precipitate_only(solution, phases: list) -> None:
    """Let `phases` PRECIPITATE only — never dissolve into the water.

    `phreeqpython.equalize` defaults `in_phase=[10]`, which puts 10 moles of each
    mineral in the system before equilibrating, and PHREEQC will then dissolve
    that solid into solution. Requesting Zn(OH)2 as a lime-treatment target
    therefore *injected* thousands of mg/L of zinc that was never in the input
    (observed: Zn 2 mmol in, 6513 mg/L reported out). Passing `in_phase = 0`
    means no solid is initially present, so a supersaturated phase can only be
    created and an undersaturated one does nothing.
    """
    if not phases:
        return
    zeros = [0.0] * len(phases)
    solution.equalize(phases, zeros, zeros)


def titrate_to_ph(solution, target_ph: float, phases: list):
    """Add Ca(OH)2 until `target_ph` is met, then bisect to land on it.

    A fixed 5 mmol step overshoots badly: reaching pH 8.5 by coarse steps landed
    at pH 11.49, which changes which hydroxides precipitate and therefore the
    reported metal removal. The coarse phase brackets the target, then a bisection
    refines the lime dose so the reported pH is the pH that was asked for.
    Returns (solution, lime_mmol, steps).
    """
    current = solution.copy()
    if phases:
        precipitate_only(current, phases)
    if current.pH >= target_ph:
        return current, 0.0, 0

    low = 0.0
    high = 0.0
    steps = 0
    bracket = None
    while steps < MAX_TITRATION_STEPS:
        candidate = solution.copy()
        high += LIME_STEP_MMOL
        candidate.add("Ca(OH)2", high, "mmol")
        if phases:
            precipitate_only(candidate, phases)
        steps += 1
        if candidate.pH >= target_ph:
            bracket = candidate
            break
        low = high

    if bracket is None:
        # Never reached the target within the budget; return the most alkaline attempt.
        final = solution.copy()
        final.add("Ca(OH)2", high, "mmol")
        if phases:
            precipitate_only(final, phases)
        return final, high, steps

    best = bracket
    best_dose = high
    for _ in range(BISECTION_STEPS):
        mid = (low + high) / 2.0
        candidate = solution.copy()
        candidate.add("Ca(OH)2", mid, "mmol")
        if phases:
            precipitate_only(candidate, phases)
        steps += 1
        if candidate.pH >= target_ph:
            best, best_dose, high = candidate, mid, mid
        else:
            low = mid
        if abs(best.pH - target_ph) <= PH_TOLERANCE:
            break
    return best, best_dose, steps


def run(request: dict) -> dict:
    import phreeqpython

    solution_spec = request.get("solution")
    if not isinstance(solution_spec, dict) or not solution_spec:
        fail_invalid("solution must be a non-empty object")

    database = str(request.get("database", DEFAULT_DATABASE))
    if "/" in database or "\\" in database or database.startswith("."):
        fail_invalid("database must be a bare file name inside the resources directory")
    database_dir = Path(request.get("database_directory", DEFAULT_DATABASE_DIR))
    database_path = database_dir / database
    if not database_path.is_file():
        fail_invalid("database not found: {}".format(database_path))

    units = str(request.get("units", "mmol"))
    if units not in {"mmol", "mol", "mg", "umol", "ug"}:
        fail_invalid("units must be one of mmol, mol, mg, umol, ug")

    wanted = requested_elements(solution_spec)
    if not wanted:
        fail_invalid("solution must specify at least one element with a positive amount")

    si_phases = list(DEFAULT_SI_PHASES)
    for extra in request.get("saturation_indices", []) or []:
        if isinstance(extra, str) and extra not in si_phases:
            si_phases.append(extra)

    pp = phreeqpython.PhreeqPython(database=database, database_directory=database_dir)

    spec = dict(solution_spec)
    spec.setdefault("units", "{}/kgw".format(units))
    if "temperature_c" in request:
        spec["temp"] = finite(request["temperature_c"])

    raw_solution = pp.add_solution(spec)
    elements = sorted(wanted)

    # Honesty guard: an element with no master species is accepted and silently
    # reported as zero. Detect that before anything downstream reads the number.
    recovery = []
    unsupported = []
    for element in elements:
        reported = finite(raw_solution.total(element, units="mmol"))
        asked = wanted[element]
        # Compare in the request's own units only when they are molar; for mass
        # units the ratio is still monotonic, so a hard zero remains detectable.
        recovered = reported > 0.0
        recovery.append(
            {
                "element": element,
                "requested": asked,
                "requested_units": units,
                "reported_mmol": reported,
                "recovered": recovered,
            }
        )
        if not recovered:
            unsupported.append(element)

    equilibrium_phases = [p for p in (request.get("equilibrium_phases") or []) if isinstance(p, str)]
    if equilibrium_phases:
        precipitate_only(raw_solution, equilibrium_phases)

    raw = describe(raw_solution, elements, si_phases, equilibrium_phases)

    treated = None
    lime_added_mmol = 0.0
    target = request.get("lime_titration_target_ph")
    if target is not None:
        target_ph = finite(target)
        if not 0.0 < target_ph < 14.0:
            fail_invalid("lime_titration_target_ph must be between 0 and 14")
        treated_solution, lime_added_mmol, steps = titrate_to_ph(
            raw_solution, target_ph, equilibrium_phases
        )
        treated = describe(treated_solution, elements, si_phases, equilibrium_phases)
        treated["target_ph"] = target_ph
        treated["ph_error"] = abs(treated_solution.pH - target_ph)
        treated["reached_target"] = treated["ph_error"] <= PH_TOLERANCE
        treated["titration_steps"] = steps

    return {
        "status": "ok",
        "database": database,
        "database_sha256": sha256_of(database_path),
        "units": units,
        "raw": raw,
        "treated": treated,
        "lime_added_mmol": lime_added_mmol,
        "element_recovery": recovery,
        "unsupported_elements": unsupported,
    }


def main(argv: list) -> int:
    parser = argparse.ArgumentParser(
        description="Execute a PHREEQC speciation/leaching calculation from a JSON request on stdin.",
    )
    parser.add_argument(
        "--input",
        help="path to a JSON request file; omit to read the request from stdin",
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
