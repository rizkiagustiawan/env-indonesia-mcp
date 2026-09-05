import sys
import json
import math
import argparse
from pathlib import Path

try:
    import cadquery as cq
except ImportError:
    print(json.dumps({"error": "CadQuery is not installed in the current python environment."}))
    sys.exit(1)

def generate_spillway(length, width, channel_depth, wall_thickness, slope_ratio, output_file):
    """
    Parametric Spillway / Open Channel 3D Model
    Slope ratio H:V (e.g. 2 means 2 Horizontal to 1 Vertical)
    """
    bottom_width = width
    top_width = width + 2 * (channel_depth * slope_ratio)
    
    # Create the cross section profile
    # Origin at bottom center
    pts = [
        (-bottom_width/2 - wall_thickness, channel_depth),
        (-bottom_width/2, channel_depth),
        (-bottom_width/2, 0),
        (bottom_width/2, 0),
        (bottom_width/2, channel_depth),
        (bottom_width/2 + wall_thickness, channel_depth),
        (top_width/2 + wall_thickness, channel_depth + channel_depth*0.5), # Outer wall slope
        (-top_width/2 - wall_thickness, channel_depth + channel_depth*0.5)
    ]
    
    # Simple rectangular channel for MVP robustness (cadquery can be finicky with open profiles)
    outer_width = width + 2*wall_thickness
    outer_depth = channel_depth + wall_thickness
    
    # Base Box
    spillway = cq.Workplane("XY").box(length, outer_width, outer_depth)
    # Cutout channel
    cutout = cq.Workplane("XY").translate((0, 0, wall_thickness/2)).box(length + 1, width, channel_depth)
    
    result = spillway.cut(cutout)
    
    if output_file.endswith('.step') or output_file.endswith('.stp'):
        cq.exporters.export(result, output_file)
    elif output_file.endswith('.stl'):
        cq.exporters.export(result, output_file)
    else:
        # Default step
        output_file += ".step"
        cq.exporters.export(result, output_file)
        
    return {
        "status": "success",
        "output_file": str(Path(output_file).resolve()),
        "volume_m3": round(result.val().Volume() / 1e9, 2)
    }

def generate_wwtp_tank(length, width, height, thickness, baffles, output_file):
    """
    Parametric Wastewater Treatment Tank (RO/MBR)
    """
    outer = cq.Workplane("XY").box(length, width, height)
    inner = cq.Workplane("XY").translate((0, 0, thickness)).box(length - 2*thickness, width - 2*thickness, height)
    tank = outer.cut(inner)
    
    # Add baffles
    if baffles > 0:
        baffle_spacing = (length - 2*thickness) / (baffles + 1)
        for i in range(baffles):
            # Alternate baffle sides
            y_offset = thickness if i % 2 == 0 else -thickness
            baffle_len = width - 2*thickness - (width * 0.2) # Leave 20% gap for flow
            b_x = -length/2 + thickness + (i+1)*baffle_spacing
            
            baffle = cq.Workplane("XY").center(b_x, y_offset).box(thickness, baffle_len, height - thickness)
            tank = tank.union(baffle)

    cq.exporters.export(tank, output_file)
    
    return {
        "status": "success",
        "output_file": str(Path(output_file).resolve()),
        "volume_m3": round(tank.val().Volume() / 1e9, 2),
    }

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--type", choices=["spillway", "wwtp_tank", "raw_script"], required=True)
    parser.add_argument("--length", type=float, default=10000.0) # mm
    parser.add_argument("--width", type=float, default=2000.0)
    parser.add_argument("--depth", type=float, default=1500.0)
    parser.add_argument("--thickness", type=float, default=200.0)
    parser.add_argument("--baffles", type=int, default=3)
    parser.add_argument("--output", type=str, required=True)
    
    args = parser.parse_args()
    
    try:
        if args.type == "spillway":
            res = generate_spillway(args.length, args.width, args.depth, args.thickness, 2.0, args.output)
        elif args.type == "wwtp_tank":
            res = generate_wwtp_tank(args.length, args.width, args.depth, args.thickness, args.baffles, args.output)
        
        print(json.dumps(res))
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)

if __name__ == "__main__":
    main()
