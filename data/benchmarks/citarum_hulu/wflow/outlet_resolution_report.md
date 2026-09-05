# Citarum Wflow Outlet Resolution and Timing Check
*Date: 2026-08-31*

## 1. Outlet Resolution
The provisional outlet coordinate `(107.62025, -6.994727)` was found to be an interior land cell. By analyzing the `wflow_ldd` and `wflow_river` arrays, the true terminus of the modeled river network was identified at the single drainage pit: **Row 68, Col 21 (107.36571, -6.90815)**. 

- **Contributing Area:** Tracing the LDD upstream from this pit yields exactly **7,532 cells**, which is 100% of the active model domain. This translates to **2,299.2 km²**, matching the published Citarum Hulu basin polygon (2,311.4 km²) to within 99.5%.
- **Location:** The pit falls geographically at the entrance to the Saguling reservoir, corresponding hydrologically to the Nanjung AWLR gauge station.

The `citarum_hulu_outlet.json` contract has been rewritten with explicit grid indices and its validation state promoted to `resolved`. The validator `citarum_outlet.py` was extended to strictly reject any outlet falling on a non-river or inactive cell when cross-checked against `staticmaps.nc`.

## 2. Reducer vs Explicit Extraction
The previous screening runs used `reducer = "maximum"` over the entire domain to extract discharge (Q). The configuration was updated to use explicit coordinate-based extraction at the pit (`coordinate.x = 107.36571`, `coordinate.y = -6.90815`).

Comparison showed that the explicit outlet discharge and the domain-wide maximum are nearly identical (differing on only 5 of 75 days by a maximum of 0.0025 m³/s). This confirms that flow accumulation correctly drives the maximum volume to the terminal pit.

## 3. Timing Discrepancy Gate
The Wflow hydrograph forced by CHIRPS daily data exhibits its top 5 peaks in February 2016 (Peak: Feb 11 at 465.6 m³/s). 
During the recorded major flood event window (March 12-14, 2016), the simulated discharge drops to a local low (188.3 m³/s on March 13).

**Conclusion:** The CHIRPS forcing fails to produce a hydrological peak matching the recorded March 13th flood event. Without independent AWLR discharge observations to verify whether this is a forcing error (CHIRPS missing localized rainfall) or a model parameter issue, calibration cannot proceed safely. The model remains strictly `screening_only`.