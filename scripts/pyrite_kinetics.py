#!/usr/bin/env python3
"""Simulate time-dependent pyrite oxidation (acid mine drainage generation).

Reads one JSON request on stdin, writes exactly one line of JSON to stdout.

This is the KINETIC counterpart to `phreeqc_run.py`. Static ABA screening
(MPA/NAPP) says how much acid a rock *could* make; equilibrium speciation says
what a given water *is*. Neither says how fast acid appears, which is the
question that decides whether a pit needs treatment in month 3 or year 30.

Rate law: Williamson & Rimstidt (1994), as shipped in the RATES block of the
database. In the repaired WATEQ4F database it evaluates as

    log10(rate) = -10.19 + parm1 + parm3*log(mO2) + parm4*log(mH+)
                  + parm2*log10(m/m0)

    parm1 = log10(A/V) with A/V in 1/dm      parm2 = exponent on (m/m0)
    parm3 = exponent on O2 (0.5)             parm4 = exponent on H+ (-0.11)

WARNING ON DATABASE PORTABILITY: `phreeqc.dat` implements the same reference
with a different intercept (-8.19) and treats parm1 as log10(specific area in
m2/mol) combined with log10(M0). The SAME `-parms` values therefore mean
DIFFERENT things in the two databases. This script pins the database and
reports which one produced the numbers.

Request:

    {"pyrite_mol_per_kgw": 0.05,
     "initial_ph": 6.5,
     "initial_o2_mmol": 0.27,
     "replenish_o2": true,
     "o2_partial_pressure_log10": -0.68,
     "steps_days": [1, 30, 90, 180, 365],
     "parms": [1.0, 0.67, 0.5, -0.11],
     "temperature_c": 25.0,
     "neutralising_phases": ["Calcite"]}

Contract: {"status":"ok","database":...,"series":[...],"guards":{...}}

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

# Williamson & Rimstidt (1994) exponents: rate ∝ mO2^0.5 * mH+^-0.11
DEFAULT_PARMS = [1.0, 0.67, 0.5, -0.11]

SECONDS_PER_DAY = 86400.0
MAX_STEPS = 200

# FeS2 releases 2 mol S per mol Fe. A measured ratio far from 2 means Fe was
# removed by secondary precipitation, so dissolved Fe understates oxidation.
STOICHIOMETRIC_S_PER_FE = 2.0
STOICHIOMETRY_TOLERANCE = 0.15

# Below this pH change across the final half of the series the reaction has
# effectively stalled.
STALL_PH_DELTA = 0.02

# Above this fraction of pyrite consumed the run is depletion-limited.
DEPLETION_FRACTION = 0.99


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


def validate(request: dict) -> dict:
    pyrite = finite(request.get("pyrite_mol_per_kgw"), float("nan"))
    if not pyrite > 0.0:
        fail_invalid("pyrite_mol_per_kgw must be a positive finite number")

    initial_ph = finite(request.get("initial_ph"), 6.5)
    if not 0.0 < initial_ph < 14.0:
        fail_invalid("initial_ph must be between 0 and 14")

    o2 = finite(request.get("initial_o2_mmol"), 0.27)
    if o2 < 0.0:
        fail_invalid("initial_o2_mmol must not be negative")

    temperature = finite(request.get("temperature_c"), 25.0)
    if not -10.0 <= temperature <= 100.0:
        fail_invalid("temperature_c must be between -10 and 100")

    steps = request.get("steps_days") or [1, 30, 90, 180, 365]
    if not isinstance(steps, list) or not steps:
        fail_invalid("steps_days must be a non-empty list")
    if len(steps) > MAX_STEPS:
        fail_invalid("steps_days must contain at most {} entries".format(MAX_STEPS))
    parsed_steps = []
    previous = 0.0
    for index, step in enumerate(steps):
        value = finite(step, float("nan"))
        if not value > 0.0:
            fail_invalid("steps_days[{}] must be positive".format(index))
        if value <= previous:
            fail_invalid("steps_days must be strictly increasing (cumulative times)")
        previous = value
        parsed_steps.append(value)

    parms = request.get("parms") or DEFAULT_PARMS
    if not isinstance(parms, list) or len(parms) != 4:
        fail_invalid("parms must be a list of exactly 4 numbers")
    parms = [finite(p, float("nan")) for p in parms]
    if any(p != p for p in parms):
        fail_invalid("parms must contain only finite numbers")

    phases = request.get("neutralising_phases") or []
    if not isinstance(phases, list) or any(not isinstance(p, str) for p in phases):
        fail_invalid("neutralising_phases must be a list of phase names")

    database = str(request.get("database", DEFAULT_DATABASE))
    if "/" in database or "\\" in database or database.startswith("."):
        fail_invalid("database must be a bare file name inside the resources directory")

    return {
        "pyrite_mol_per_kgw": pyrite,
        "initial_ph": initial_ph,
        "initial_o2_mmol": o2,
        "replenish_o2": bool(request.get("replenish_o2", True)),
        "o2_partial_pressure_log10": finite(request.get("o2_partial_pressure_log10"), -0.68),
        "steps_days": parsed_steps,
        "parms": parms,
        "temperature_c": temperature,
        "neutralising_phases": phases,
        "database": database,
        "database_directory": Path(request.get("database_directory", DEFAULT_DATABASE_DIR)),
    }


def build_input(spec: dict) -> str:
    blocks = []
    blocks.append(
        "SOLUTION 1\n"
        "    units   mmol/kgw\n"
        "    temp    {temp}\n"
        "    pH      {ph}\n"
        "    O(0)    {o2}\n".format(
            temp=spec["temperature_c"],
            ph=spec["initial_ph"],
            o2=spec["initial_o2_mmol"],
        )
    )

    equilibrium = []
    if spec["replenish_o2"]:
        # A large O2(g) reservoir at fixed partial pressure: an open pit or waste
        # dump exposed to the atmosphere.
        equilibrium.append(
            "    O2(g)  {:.4f}  1000.0".format(spec["o2_partial_pressure_log10"])
        )
    for phase in spec["neutralising_phases"]:
        equilibrium.append("    {}  0.0  10.0".format(phase))
    if equilibrium:
        blocks.append("EQUILIBRIUM_PHASES 1\n" + "\n".join(equilibrium) + "\n")

    step_seconds = " ".join(
        "{:.6f}".format(day * SECONDS_PER_DAY) for day in spec["steps_days"]
    )
    blocks.append(
        "SELECTED_OUTPUT\n"
        "    -reset      false\n"
        "    -time       true\n"
        "    -ph         true\n"
        "    -pe         true\n"
        "    -totals     Fe S(6)\n"
        "    -kinetic_reactants Pyrite\n"
    )
    blocks.append(
        "KINETICS 1\n"
        "Pyrite\n"
        "    -formula  FeS2  1.0\n"
        "    -m0       {m0}\n"
        "    -parms    {p0} {p1} {p2} {p3}\n"
        "    -tol      1e-10\n"
        "    -steps    {steps}\n"
        "END\n".format(
            m0=spec["pyrite_mol_per_kgw"],
            p0=spec["parms"][0],
            p1=spec["parms"][1],
            p2=spec["parms"][2],
            p3=spec["parms"][3],
            steps=step_seconds,
        )
    )
    return "\n".join(blocks)


def run(request: dict) -> dict:
    from phreeqpython.viphreeqc import VIPhreeqc

    spec = validate(request)
    database_path = spec["database_directory"] / spec["database"]
    if not database_path.is_file():
        fail_invalid("database not found: {}".format(database_path))

    # A fresh interpreter per run: VIPhreeqc retains state between run_string
    # calls, so a reused instance leaks the previous run's kinetic amounts into
    # the first reported row.
    ip = VIPhreeqc()
    ip.load_database(database_path)
    ip.run_string(build_input(spec))

    columns = [ip.get_selected_output_value(0, c) for c in range(ip.column_count)]
    index = {name: position for position, name in enumerate(columns)}
    for required in ("time", "pH", "Fe(mol/kgw)", "S(6)(mol/kgw)"):
        if required not in index:
            fail_error("selected output missing column {}".format(required))
    pyrite_column = index.get("k_Pyrite")

    series = []
    for row in range(1, ip.row_count):
        values = [ip.get_selected_output_value(row, c) for c in range(ip.column_count)]
        time_seconds = finite(values[index["time"]], -1.0)
        # PHREEQC writes the pre-kinetics solution with time = -99; its kinetic
        # column is not yet meaningful for this run.
        if time_seconds < 0.0:
            continue
        remaining = (
            finite(values[pyrite_column], 0.0) if pyrite_column is not None else None
        )
        series.append(
            {
                "time_days": time_seconds / SECONDS_PER_DAY,
                "ph": finite(values[index["pH"]]),
                "pe": finite(values[index["pe"]]) if "pe" in index else None,
                "fe_mol_kgw": finite(values[index["Fe(mol/kgw)"]]),
                "sulfate_mol_kgw": finite(values[index["S(6)(mol/kgw)"]]),
                "pyrite_remaining_mol_kgw": remaining,
            }
        )

    if not series:
        fail_error("kinetic run produced no time steps")

    return {
        "status": "ok",
        "database": spec["database"],
        "database_sha256": sha256_of(database_path),
        "rate_law": "Williamson & Rimstidt (1994) pyrite oxidation, RATES block of the loaded database",
        "parms": spec["parms"],
        "replenish_o2": spec["replenish_o2"],
        "neutralising_phases": spec["neutralising_phases"],
        "initial_pyrite_mol_kgw": spec["pyrite_mol_per_kgw"],
        "series": series,
        "guards": build_guards(spec, series),
    }


def build_guards(spec: dict, series: list) -> dict:
    """Compute the honesty guards for a kinetic pyrite run."""
    final = series[-1]
    first = series[0]

    initial_pyrite = spec["pyrite_mol_per_kgw"]
    remaining = final.get("pyrite_remaining_mol_kgw")
    consumed_fraction = None
    if remaining is not None and initial_pyrite > 0.0:
        consumed_fraction = max(0.0, (initial_pyrite - remaining) / initial_pyrite)

    # Stall detection: compare the pH drop across the second half of the series.
    midpoint = series[len(series) // 2]
    late_ph_drop = midpoint["ph"] - final["ph"]
    stalled = abs(late_ph_drop) < STALL_PH_DELTA

    # Stoichiometry: FeS2 gives 2 S per Fe. A ratio far from 2 means dissolved Fe
    # no longer tracks how much pyrite oxidised.
    ratio = None
    if final["fe_mol_kgw"] > 0.0:
        ratio = final["sulfate_mol_kgw"] / final["fe_mol_kgw"]
    stoichiometry_ok = (
        ratio is not None
        and abs(ratio - STOICHIOMETRIC_S_PER_FE) <= STOICHIOMETRY_TOLERANCE
    )

    depleted = consumed_fraction is not None and consumed_fraction >= DEPLETION_FRACTION

    return {
        "oxygen_replenished": spec["replenish_o2"],
        "oxygen_limited": bool(stalled and not depleted),
        "late_ph_change": late_ph_drop,
        "pyrite_consumed_fraction": consumed_fraction,
        "pyrite_depleted": bool(depleted),
        "sulfate_to_iron_ratio": ratio,
        "stoichiometry_consistent": bool(stoichiometry_ok),
        "initial_ph": first["ph"],
        "final_ph": final["ph"],
        "simulated_days": final["time_days"],
        "rate_is_laboratory_derived": True,
    }


def main(argv: list) -> int:
    parser = argparse.ArgumentParser(
        description="Simulate pyrite oxidation kinetics from a JSON request on stdin.",
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
