"""Hydrological conditioning primitives for DEM rasters.

The implementation keeps the basin validity mask separate from elevation
values. Nodata outside an AOI is never treated as a low elevation or filled.
"""

import heapq

import numpy as np


def priority_flood_fill(dem, valid_mask=None, connectivity=8):
    """Fill closed depressions to their lowest valid spill elevation.

    Parameters
    ----------
    dem:
        Two-dimensional finite elevation array.
    valid_mask:
        Optional boolean array. False cells are excluded and remain unchanged.
        Valid cells touching the mask boundary are treated as outlets.
    connectivity:
        Neighbor connectivity, either 4 or 8.
    """
    elevation = np.asarray(dem, dtype=float)
    if elevation.ndim != 2 or elevation.size == 0:
        raise ValueError("DEM must be a non-empty two-dimensional array")
    if not np.all(np.isfinite(elevation)):
        raise ValueError("DEM must contain only finite values")
    neighbor_offsets = _neighbor_offsets(connectivity)

    valid = np.ones(elevation.shape, dtype=bool) if valid_mask is None else np.asarray(valid_mask, dtype=bool)
    if valid.shape != elevation.shape:
        raise ValueError("valid_mask must have the same shape as DEM")
    if not valid.any():
        raise ValueError("valid_mask must contain at least one valid cell")

    filled = elevation.copy()
    visited = np.zeros(elevation.shape, dtype=bool)
    queue = []
    rows, cols = elevation.shape

    for row in range(rows):
        for col in range(cols):
            if not valid[row, col] or not _is_boundary_cell(valid, row, col, neighbor_offsets):
                continue
            visited[row, col] = True
            heapq.heappush(queue, (filled[row, col], row, col))

    while queue:
        level, row, col = heapq.heappop(queue)
        for next_row, next_col in _neighbors(row, col, rows, cols, neighbor_offsets):
            if not valid[next_row, next_col] or visited[next_row, next_col]:
                continue
            visited[next_row, next_col] = True
            filled[next_row, next_col] = max(filled[next_row, next_col], level)
            heapq.heappush(queue, (filled[next_row, next_col], next_row, next_col))

    if not np.all(visited[valid]):
        raise ValueError("valid DEM cells are not connected to a valid boundary")
    return filled


def condition_dem(dem, stream_mask, burn_depth_m=5.0, valid_mask=None, connectivity=8):
    """Lower stream-mask cells, then fill depressions in the burned DEM."""
    elevation = np.asarray(dem, dtype=float)
    mask = np.asarray(stream_mask)
    if elevation.shape != mask.shape:
        raise ValueError("stream_mask must have the same shape as DEM")
    if not np.isfinite(burn_depth_m) or burn_depth_m < 0:
        raise ValueError("burn_depth_m must be a finite non-negative number")
    if not np.all(np.isin(mask, [0, 1])):
        raise ValueError("stream_mask must contain only 0 and 1")
    _neighbor_offsets(connectivity)

    burned = np.where(mask == 1, elevation - burn_depth_m, elevation)
    filled = priority_flood_fill(burned, valid_mask=valid_mask, connectivity=connectivity)
    conditioned = filled
    if valid_mask is not None:
        conditioned = np.where(np.asarray(valid_mask, dtype=bool), conditioned, elevation)
    return conditioned


def count_interior_pits(dem, valid_mask=None, connectivity=8):
    """Count valid cells lower than every valid neighbor."""
    elevation = np.asarray(dem, dtype=float)
    if elevation.ndim != 2 or elevation.size == 0:
        raise ValueError("DEM must be a non-empty two-dimensional array")
    if not np.all(np.isfinite(elevation)):
        raise ValueError("DEM must contain only finite values")
    offsets = _neighbor_offsets(connectivity)
    valid = np.ones(elevation.shape, dtype=bool) if valid_mask is None else np.asarray(valid_mask, dtype=bool)
    if valid.shape != elevation.shape:
        raise ValueError("valid_mask must have the same shape as DEM")
    padded = np.pad(elevation, 1, mode="constant", constant_values=np.inf)
    valid_padded = np.pad(valid, 1, mode="constant", constant_values=False)
    lower_than_neighbors = valid.copy()
    rows, cols = elevation.shape
    for row_delta, col_delta in offsets:
        neighbor = padded[1 + row_delta:1 + row_delta + elevation.shape[0], 1 + col_delta:1 + col_delta + elevation.shape[1]]
        neighbor_valid = valid_padded[1 + row_delta:1 + row_delta + elevation.shape[0], 1 + col_delta:1 + col_delta + elevation.shape[1]]
        lower_than_neighbors &= (~neighbor_valid) | (elevation < neighbor)
    boundary = np.zeros(elevation.shape, dtype=bool)
    for row in range(rows):
        for col in range(cols):
            boundary[row, col] = _is_boundary_cell(valid, row, col, offsets)
    lower_than_neighbors &= ~boundary
    return int(lower_than_neighbors.sum())


def _neighbor_offsets(connectivity):
    if connectivity == 4:
        return ((-1, 0), (1, 0), (0, -1), (0, 1))
    if connectivity == 8:
        return (
            (-1, -1), (-1, 0), (-1, 1), (0, -1),
            (0, 1), (1, -1), (1, 0), (1, 1),
        )
    raise ValueError("connectivity must be 4 or 8")


def _neighbors(row, col, rows, cols, offsets):
    for row_delta, col_delta in offsets:
        next_row = row + row_delta
        next_col = col + col_delta
        if 0 <= next_row < rows and 0 <= next_col < cols:
            yield next_row, next_col


def _is_boundary_cell(valid, row, col, offsets):
    rows, cols = valid.shape
    if row in (0, rows - 1) or col in (0, cols - 1):
        return True
    return any(
        not (0 <= next_row < rows and 0 <= next_col < cols) or not valid[next_row, next_col]
        for next_row, next_col in _neighbors(row, col, rows, cols, offsets)
    )
