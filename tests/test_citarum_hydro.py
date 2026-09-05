import numpy as np
import pytest

from tools.wflow_env.citarum_hydro import (
    contributing_area_km2,
    drainage_path,
    upstream_cells,
)


def _array(values):
    return np.array(values, dtype=np.float64)


def test_single_pit_upstream_is_itself():
    ldd = _array([[5.0]])

    mask = upstream_cells(ldd, 0, 0)

    assert mask.dtype == np.bool_
    assert mask.tolist() == [[True]]


def test_chain_drains_to_pit():
    ldd = _array([[8.0], [8.0], [5.0]])

    mask = upstream_cells(ldd, 2, 0)

    assert mask.tolist() == [[True], [True], [True]]


def test_diagonal_flow_is_followed():
    # north-west source drains diagonally into the pit
    ldd = _array([[9.0, np.nan], [np.nan, 5.0]])
    active = np.array([[True, False], [False, True]])

    mask = upstream_cells(ldd, 1, 1, active_mask=active)

    assert mask.tolist() == [[True, False], [False, True]]


def test_branching_confluence_reaches_pit():
    # two arms of a Y merge at the pit
    ldd = _array(
        [
            [9.0, np.nan, np.nan],
            [np.nan, 5.0, np.nan],
            [np.nan, np.nan, 1.0],
        ]
    )

    mask = upstream_cells(ldd, 1, 1)

    assert mask[0, 0]
    assert mask[1, 1]
    assert mask[2, 2]
    assert not mask[0, 1]
    assert not mask[0, 2]
    assert not mask[1, 0]
    assert not mask[1, 2]
    assert not mask[2, 0]
    assert not mask[2, 1]


def test_cell_draining_off_domain_is_excluded():
    # top cell flows north off the grid; it never reaches the pit
    ldd = _array([[2.0], [5.0]])

    mask = upstream_cells(ldd, 1, 0)

    assert mask.tolist() == [[False], [True]]


def test_inactive_cells_do_not_contribute():
    # inactive neighbour would drain into the pit but must be ignored
    ldd = _array([[8.0, 8.0], [5.0, np.nan]])
    active = np.array([[True, False], [True, False]])

    mask = upstream_cells(ldd, 1, 0, active_mask=active)

    assert mask.tolist() == [[True, False], [True, False]]


def test_nan_ldd_cells_are_never_upstream():
    ldd = _array([[np.nan, 8.0], [np.nan, 5.0]])

    mask = upstream_cells(ldd, 1, 1)

    assert mask.tolist() == [[False, True], [False, True]]


def test_invalid_ldd_value_in_active_cell_raises():
    ldd = _array([[0.0, 8.0], [np.nan, 5.0]])
    active = np.array([[True, True], [False, True]])

    with pytest.raises(ValueError, match="ldd"):
        upstream_cells(ldd, 1, 1, active_mask=active)


def test_pit_location_must_hold_ldd_five():
    ldd = _array([[8.0], [5.0]])

    with pytest.raises(ValueError, match="pit"):
        upstream_cells(ldd, 0, 0)


def test_drainage_path_from_cell_to_pit():
    ldd = _array([[8.0, 4.0], [5.0, np.nan]])

    path = drainage_path(ldd, 0, 1)

    assert path == [(0, 1), (0, 0), (1, 0)]


def test_drainage_path_detects_cycle():
    ldd = _array([[6.0, 4.0], [np.nan, np.nan]])

    with pytest.raises(ValueError, match="cycle"):
        drainage_path(ldd, 0, 0)


def test_drainage_path_off_domain_reports_exit():
    # the southward flow leaves the active grid at the NaN cell
    ldd = _array([[8.0, np.nan], [np.nan, np.nan]])

    path = drainage_path(ldd, 0, 0)

    assert path == [(0, 0), (1, 0)]


def test_contributing_area_km2():
    assert contributing_area_km2(cell_count=100, cell_size_m=552.5) == pytest.approx(
        100 * 552.5 * 552.5 / 1e6
    )
