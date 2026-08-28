using Dates
using NCDatasets
using SHA
using Wflow

const ROOT = @__DIR__
const INPUT = joinpath(ROOT, "data", "input")
const OUTPUT = joinpath(ROOT, "data", "output")
mkpath(INPUT)
mkpath(OUTPUT)

function write_grid(path, variables)
    ds = NCDataset(path, "c")
    defDim(ds, "x", 2)
    defDim(ds, "y", 2)
    defDim(ds, "layer", 4)
    defDim(ds, "time", 3)
    defVar(ds, "x", Float64, ("x",))[:] = [0.0, 1000.0]
    defVar(ds, "y", Float64, ("y",))[:] = [0.0, 1000.0]
    for (name, data) in variables
        dims = ndims(data) == 2 ? ("y", "x") : (ndims(data) == 3 ? ("layer", "y", "x") : ("y", "x", "time"))
        defVar(ds, name, Float64, dims)[:] = data
    end
    close(ds)
end

static_vars = Dict{String, Array{Float64}}()
for (name, value) in (("wflow_ldd", 5.0), ("wflow_river", 0.0), ("wflow_subcatch", 1.0),
                      ("wflow_riverlength", 1000.0), ("wflow_riverwidth", 10.0),
                      ("RiverSlope", 0.001), ("N_River", 0.035), ("Slope", 0.01),
                      ("N", 0.05), ("WaterFrac", 0.0), ("TT", 0.0), ("TTI", 1.0),
                       ("TTM", 0.0), ("Cfmax", 3.75), ("cf_soil", 1.0),
                      ("KsatVer", 10.0), ("f", 1.0), ("InfiltCapPath", 10.0),
                      ("thetaR", 0.05), ("thetaS", 0.45), ("MaxLeakage", 0.0),
                      ("PathFrac", 0.0), ("rootdistpar", 1.0), ("SoilThickness", 1000.0),
                      ("EoverR", 0.2), ("Kext", 0.5), ("Sl", 0.1), ("Swood", 0.1),
                      ("RootingDepth", 500.0), ("CanopyGap", 0.5), ("KsatHorFrac", 1.0))
    static_vars[name] = fill(value, 2, 2)
end
static_vars["c"] = fill(0.2, 4, 2, 2)
static_vars["wflow_river"][1, 1] = 1.0
write_grid(joinpath(INPUT, "static.nc"), static_vars)

forcing_vars = Dict(
    "precip" => fill(0.001, 2, 2, 3),
    "pet" => fill(0.0002, 2, 2, 3),
    "temp" => fill(25.0, 2, 2, 3),
)
forcing_path = joinpath(INPUT, "forcing.nc")
ds = NCDataset(forcing_path, "c")
defDim(ds, "x", 2)
defDim(ds, "y", 2)
defDim(ds, "time", 3)
defVar(ds, "x", Float64, ("x",))[:] = [0.0, 1000.0]
defVar(ds, "y", Float64, ("y",))[:] = [0.0, 1000.0]
time = defVar(ds, "time", Float64, ("time",), attrib=Dict("units" => "days since 2000-01-01 00:00:00", "calendar" => "standard"))
time[:] = [0.0, 1.0, 2.0]
for (name, data) in forcing_vars
    defVar(ds, name, Float64, ("y", "x", "time"))[:] = data
end
close(ds)

config_lines = [
    "dir_input = \"data/input\"",
    "dir_output = \"data/output\"",
    "",
    "[time]",
    "starttime = 2000-01-01T00:00:00",
    "endtime = 2000-01-03T00:00:00",
    "timestepsecs = 86400",
    "",
    "[logging]",
    "silent = true",
    "",
    "[input]",
    "path_forcing = \"forcing.nc\"",
    "path_static = \"static.nc\"",
    "basin__local_drain_direction = \"wflow_ldd\"",
    "river_location__mask = \"wflow_river\"",
    "subbasin_location__count = \"wflow_subcatch\"",
    "",
    "[input.forcing]",
    "atmosphere_water__precipitation_volume_flux = \"precip\"",
    "land_surface_water__potential_evaporation_volume_flux = \"pet\"",
    "atmosphere_air__temperature = \"temp\"",
    "",
    "[input.static]",
    "atmosphere_air__snowfall_temperature_threshold = \"TT\"",
    "atmosphere_air__snowfall_temperature_interval = \"TTI\"",
    "snowpack__melting_temperature_threshold = \"TTM\"",
    "snowpack__degree_day_coefficient = \"Cfmax\"",
    "soil_layer_water__brooks_corey_exponent = \"c\"",
    "soil_surface_water__infiltration_reduction_parameter = \"cf_soil\"",
    "soil_surface_water__vertical_saturated_hydraulic_conductivity = \"KsatVer\"",
    "soil_water__vertical_saturated_hydraulic_conductivity_scale_parameter = \"f\"",
    "compacted_soil_surface_water__infiltration_capacity = \"InfiltCapPath\"",
    "soil_water__residual_volume_fraction = \"thetaR\"",
    "soil_water__saturated_volume_fraction = \"thetaS\"",
    "soil_water_saturated_zone_bottom__max_leakage_volume_flux = \"MaxLeakage\"",
    "compacted_soil__area_fraction = \"PathFrac\"",
    "soil_wet_root__sigmoid_function_shape_parameter = \"rootdistpar\"",
    "soil__thickness = \"SoilThickness\"",
    "vegetation_canopy_water__mean_evaporation_to_mean_precipitation_ratio = \"EoverR\"",
    "vegetation_canopy__light_extinction_coefficient = \"Kext\"",
    "vegetation__specific_leaf_storage = \"Sl\"",
    "vegetation_wood_water__storage_capacity = \"Swood\"",
    "vegetation_root__depth = \"RootingDepth\"",
    "vegetation_canopy__gap_fraction = \"CanopyGap\"",
    "river__length = \"wflow_riverlength\"",
    "river_water_flow__manning_n_parameter = \"N_River\"",
    "river__slope = \"RiverSlope\"",
    "river__width = \"wflow_riverwidth\"",
    "land_surface_water_flow__manning_n_parameter = \"N\"",
    "land_surface__slope = \"Slope\"",
    "subsurface_water__horizontal_to_vertical_saturated_hydraulic_conductivity_ratio = \"KsatHorFrac\"",
    "land_water_covered__area_fraction = \"WaterFrac\"",
    "",
    "[model]",
    "type = \"sbm\"",
    "soil_layer__thickness = [100, 300, 800]",
    "water_mass_balance__flag = true",
    "",
    "[output.csv]",
    "path = \"smoke.csv\"",
    "",
    "[[output.csv.column]]",
    "header = \"recharge\"",
    "parameter = \"soil_water_saturated_zone_top__recharge_volume_flux\"",
    "reducer = \"mean\"",
]
config_path = joinpath(ROOT, "smoke.toml")
write(config_path, join(config_lines, "\n") * "\n")

Wflow.run(config_path)
output_path = joinpath(OUTPUT, "smoke.csv")
@assert isfile(output_path) "Wflow did not produce smoke output"
lines = readlines(output_path)
@assert length(lines) >= 2 "Wflow smoke output has no data rows"
println("Wflow smoke test PASS")
println("version=", Base.pkgversion(Wflow))
println("output_sha256=", bytes2hex(SHA.sha256(read(output_path))))
println("rows=", length(lines) - 1)
