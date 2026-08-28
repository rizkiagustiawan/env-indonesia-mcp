"""Regression checks for the Citarum Wflow NetCDF layout."""

from pathlib import Path

import numpy as np
import xarray as xr


ROOT = Path(__file__).resolve().parents[1]
STATICMAPS = ROOT / "data/benchmarks/citarum_hulu/wflow/staticmaps.nc"

# Wflow's internal indices are (x, y) after standardizing (lat, lon).
PCR_DIR = {
    1: (-1, -1),
    2: (0, -1),
    3: (1, -1),
    4: (-1, 0),
    5: (0, 0),
    6: (1, 0),
    7: (-1, 1),
    8: (0, 1),
    9: (1, 1),
}


def _graph_diagnostics():
    with xr.open_dataset(STATICMAPS) as ds:
        # xarray masks the subcatchment fill value as NaN, matching Wflow's
        # active-cell selection through NCDatasets allow_missing=true.
        active = ~np.isnan(ds["wflow_subcatch"].values)
        ldd = ds["wflow_ldd"].values.T
        active = active.T

    shape = ldd.shape
    downstream = {}
    outside = 0
    for x, y in zip(*np.where(active)):
        value = int(ldd[x, y])
        assert value in PCR_DIR, (x, y, value)
        if value == 5:
            downstream[(x, y)] = None
            continue
        dx, dy = PCR_DIR[value]
        target = (x + dx, y + dy)
        if not (0 <= target[0] < shape[0] and 0 <= target[1] < shape[1]) or not active[target]:
            outside += 1
            downstream[(x, y)] = None
        else:
            downstream[(x, y)] = target

    visited = set()
    cycles = 0
    for start in downstream:
        if start in visited:
            continue
        path = set()
        current = start
        while current is not None and current in downstream:
            if current in visited:
                break
            if current in path:
                cycles += 1
                break
            path.add(current)
            current = downstream[current]
        visited.update(path)

    return outside, cycles


def test_citarum_ldd_has_no_cycles_or_external_edges():
    assert STATICMAPS.is_file()
    outside, cycles = _graph_diagnostics()
    assert outside == 0
    assert cycles == 0


def test_citarum_soil_exponent_has_three_layers():
    with xr.open_dataset(STATICMAPS) as ds:
        assert "layer" in ds["c"].dims
        # Wflow adds one sentinel layer to soil_layer__thickness internally.
        assert ds.sizes["layer"] == 4
