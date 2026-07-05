#!/usr/bin/env python3
import sys, json, subprocess, time, os

BINARY = "/home/awan/Documents/env-indonesia-mcp/target/release/env-indonesia-mcp"
SUCCESS, FAILED, TIMEOUT = 0, 0, 0

# Sample valid inputs for different param types
PARAMS_MAP = {
    # LatLonRequired
    "LatLonRequired": '{"lat": -6.2, "lon": 106.85}',
    "OceanBathyParam": '{"lat": -6.2, "lon": 106.85, "output_path": "/tmp/test.png", "title": "Test"}',
    "TimelapseParam": '{"lat": -6.2, "lon": 106.85, "buffer_km": 5, "start_year": 2020, "end_year": 2021, "sensor": "optik_s2", "output_path": "/tmp/test.gif"}',
    
    # Calculators
    "RusleParam": '{"r": 1000, "k": 0.2, "ls": 1.5, "c": 0.1, "p": 1.0}',
    "ScsCnParam": '{"rainfall_mm": 50, "cn": 75}',
    "PenmanParam": '{"t_mean_c": 28, "rh_pct": 80, "wind_ms": 2, "rn_mj": 15}',
    "StreeterPhelpsParam": '{"k1": 0.15, "k2": 0.3, "l0": 40, "d0": 2, "velocity_ms": 0.5, "distance_km": 20, "temp_c": 28}',
    
    # Compliance
    "BakuMutuUdaraParam": '{"parameter": "PM10", "concentration": 120, "averaging_time": "24_hour"}',
    "BakuMutuEmisiParam": '{"industry": "pltu_batubara", "parameter": "SO2", "concentration": 150}',
    "RiskClassParam": '{"sector": "industri", "scale_description": "1000ha", "has_hazardous_waste": false, "near_protected_area": false}',
    "AmdalScreeningParam": '{"sector": "industri", "activity": "semen", "scale_value": 50000, "scale_unit": "ton/tahun"}',
    
    # Others
    "QueryParam": '{"query": "amdal"}',
    "DaysParam": '{"days": 1}',
    "LocationParam": '{"location": "jakarta"}',
    "Empty": '{}'
}

# 1. Extract tools and their param structs
tools = []
with open("/home/awan/Documents/env-indonesia-mcp/src/server.rs", "r") as f:
    lines = f.readlines()
    for i, line in enumerate(lines):
        if "#[tool(" in line:
            fn_line = lines[i+1].strip()
            if fn_line.startswith("fn ") or fn_line.startswith("async fn "):
                name = fn_line.split("fn ")[1].split("(")[0]
                if "Parameters<" in fn_line:
                    struct = fn_line.split("Parameters<")[1].split(">")[0]
                else:
                    struct = "Empty"
                tools.append((name, struct))

print(f"Discovered {len(tools)} tools.")

# We'll just test a subset (20 tools) to save time, covering different modules
test_tools = tools[:20] 

print(f"Testing {len(test_tools)} sample tools...")

for name, struct in test_tools:
    # Use mapped params or generic fallback
    params = PARAMS_MAP.get(struct, PARAMS_MAP.get("Empty"))
    
    # Init MCP
    init = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n'
    initialized = '{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
    call = f'{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"{name}","arguments":{params}}}}}\n'
    
    payload = init + initialized + call
    
    try:
        p = subprocess.Popen([BINARY], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        out, err = p.communicate(input=payload, timeout=5)
        
        success = False
        msg = ""
        for line in out.splitlines():
            if '"id":2' in line:
                try:
                    resp = json.loads(line)
                    if "result" in resp and "error" not in resp:
                        content = resp["result"]["content"][0]["text"]
                        if "ERROR [" in content or "ERROR:" in content:
                            msg = content.split('\n')[0][:50]
                        else:
                            success = True
                    elif "error" in resp:
                        msg = resp["error"].get("message", "RPC error")
                except:
                    msg = "JSON parse error"
                    
        if success:
            print(f"✅ {name:<30} PASSED")
            SUCCESS += 1
        else:
            print(f"❌ {name:<30} FAILED: {msg}")
            FAILED += 1
            
    except subprocess.TimeoutExpired:
        p.kill()
        print(f"⏱️ {name:<30} TIMEOUT")
        TIMEOUT += 1

print(f"\n--- SUMMARY ---")
print(f"Passed : {SUCCESS}")
print(f"Failed : {FAILED}")
print(f"Timeout: {TIMEOUT}")
