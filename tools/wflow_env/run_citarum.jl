#!/usr/bin/env julia --project=tools/wflow_env
using Dates, SHA, Wflow

const BENCH = realpath(joinpath(@__DIR__, "..", "..", "data", "benchmarks", "citarum_hulu", "wflow"))
config_path = joinpath(BENCH, "citarum_sbm.toml")
println("Working dir: ", BENCH)
println("Config: ", config_path)
cd(BENCH) do
    try
        Wflow.run(config_path)
        println("Wflow Citarum run completed")
    catch e
        println("Wflow run FAILED: ", e)
        rethrow(e)
    end
end

output_csv = joinpath(BENCH, "output.csv")
if isfile(output_csv)
    lines = readlines(output_csv)
    println("Output rows: ", length(lines) - 1)
    println("Header: ", lines[1])
    if length(lines) > 1
        println("Last row: ", lines[end])
    end
    println("output_sha256=", bytes2hex(sha256(read(output_csv))))
else
    println("No output CSV found")
    # Check for log
    logfile = joinpath(BENCH, "wflow.log")
    if isfile(logfile)
        println("--- LOG ---")
        println(read(logfile, String))
    end
end