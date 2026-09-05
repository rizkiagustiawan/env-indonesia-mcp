"""Derive upstream contributing areas from a PCRaster LDD flow-direction grid."""

import numpy as np

_LDD_OFFSETS = {
    1: (-1, -1),
    2: (-1, 0),
    3: (-1, 1),
    4: (0, -1),
    5: (0, 0),
    6: (0, 1),
    7: (1, -1),
    8: (1, 0),
    9: (1, 1),
}


def _considered_mask(ldd, active_mask):
    shape = ldd.shape
    if active_mask is None:
        return ~np.isnan(ldd)
    active = np.asarray(active_mask)
    if active.shape != shape:
        raise ValueError(
            f"active_mask shape {active.shape} does not match ldd shape {shape}"
        )
    return active.astype(bool)


def _validate_ldd_values(ldd, considered):
    rows, cols = ldd.shape
    bad = 0
    first = None
    for r in range(rows):
        for c in range(cols):
            if not considered[r, c]:
                continue
            value = ldd[r, c]
            if np.isnan(value):
                bad += 1
                if first is None:
                    first = (r, c)
                continue
            if value not in _LDD_OFFSETS:
                bad += 1
                if first is None:
                    first = (r, c)
    if bad:
        r, c = first
        raise ValueError(
            f"invalid ldd value {ldd[r, c]} at (row={r}, col={c}); "
            f"{bad} active cells hold missing or unsupported ldd values"
        )


def _validate_pit(ldd, pit_row, pit_col, shape):
    rows, cols = shape
    if not (0 <= pit_row < rows and 0 <= pit_col < cols):
        raise ValueError(
            f"pit location (row={pit_row}, col={pit_col}) is outside the ldd grid"
        )
    if ldd[pit_row, pit_col] != 5:
        raise ValueError(
            f"pit cell (row={pit_row}, col={pit_col}) does not hold ldd value 5"
        )


def upstream_cells(ldd, pit_row, pit_col, active_mask=None):
    """Return a boolean mask of cells whose flow reaches the given pit.

    Only cells considered active contribute; without an explicit mask,
    every cell holding an ldd value counts. A cell drains to the offset
    target defined by its ldd value; cells flowing off the grid never
    reach the pit and are excluded.
    """
    grid = np.asarray(ldd, dtype=float)
    if grid.ndim != 2:
        raise ValueError(f"ldd must be 2D, got ndim={grid.ndim}")
    considered = _considered_mask(grid, active_mask)
    _validate_ldd_values(grid, considered)
    _validate_pit(grid, pit_row, pit_col, grid.shape)

    rows, cols = grid.shape
    incoming = {}
    for r in range(rows):
        for c in range(cols):
            if not considered[r, c]:
                continue
            dr, dc = _LDD_OFFSETS[int(grid[r, c])]
            nr, nc = r + dr, c + dc
            if 0 <= nr < rows and 0 <= nc < cols and considered[nr, nc]:
                incoming.setdefault((nr, nc), []).append((r, c))

    mask = np.zeros(grid.shape, dtype=bool)
    stack = [(pit_row, pit_col)]
    while stack:
        node = stack.pop()
        if mask[node]:
            continue
        mask[node] = True
        for source in incoming.get(node, ()):
            if not mask[source]:
                stack.append(source)
    return mask


def drainage_path(ldd, row, col):
    """Follow the downstream path from a cell until pit, exit, or cycle.

    Returns the visited cells in order. The path ends at a pit, when the
    next cell leaves the grid or holds no ldd value, or with a ValueError
    when the flow forms a cycle.
    """
    grid = np.asarray(ldd, dtype=float)
    rows, cols = grid.shape
    start_value = grid[row, col]
    if np.isnan(start_value) or start_value not in _LDD_OFFSETS:
        raise ValueError(
            f"start cell (row={row}, col={col}) holds no valid ldd value"
        )
    path = []
    visited = set()
    current = (row, col)
    while True:
        path.append(current)
        visited.add(current)
        if int(grid[current]) == 5:
            return path
        dr, dc = _LDD_OFFSETS[int(grid[current])]
        nr, nc = current[0] + dr, current[1] + dc
        if not (0 <= nr < rows and 0 <= nc < cols):
            return path
        if (nr, nc) in visited:
            raise ValueError(f"ldd cycle detected at (row={nr}, col={nc})")
        if np.isnan(grid[nr, nc]):
            path.append((nr, nc))
            return path
        current = (nr, nc)


def contributing_area_km2(cell_count, cell_size_m):
    """Convert an upstream cell count to area in square kilometres."""
    return cell_count * cell_size_m * cell_size_m / 1e6
