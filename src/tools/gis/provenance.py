#!/usr/bin/env python3
"""Provenance Metadata Generator — logs processing chain for reproducibility.
Every analysis output should be accompanied by a .meta.json file.
"""
import json, os, sys, platform
from datetime import datetime

__version__ = '1.0.0'

def create_provenance(output_path, **kwargs):
    """Generate provenance metadata JSON for an analysis output.
    
    Args:
        output_path: Path of the analysis output file
        **kwargs: Key-value pairs for metadata, e.g.:
            tool='burned_area_mapping',
            gee_collection='COPERNICUS/S2_SR_HARMONIZED',
            date_range=['2023-07-01', '2023-11-30'],
            parameters={'fire_date': '2023-09-15', 'buffer_km': 25},
            coordinates={'lat': -2.2, 'lon': 114.0},
            algorithms=['dNBR', 'RdNBR', 'USGS severity classification'],
            references=['Key & Benson 2006', 'Miller & Thode 2007'],
            masking='Cloud Score+ (cs_cdf >= 0.60)',
            validation={'method': 'Olofsson 2014', 'ci': '95%'},
            crs='EPSG:4326',
            scale_m=10
    """
    meta = {
        'provenance': {
            'generator': 'env-indonesia-mcp',
            'version': '1.0.0',
            'timestamp': datetime.now().isoformat(),
            'platform': platform.system(),
            'python_version': platform.python_version(),
            'output_file': os.path.basename(output_path),
            'output_path': os.path.abspath(output_path),
        },
        'processing': {}
    }
    
    # Add all kwargs to processing section
    for key, value in kwargs.items():
        meta['processing'][key] = value
    
    # Write metadata
    meta_path = output_path + '.meta.json'
    with open(meta_path, 'w') as f:
        json.dump(meta, f, indent=2, default=str)
    
    return meta_path


def read_provenance(output_path):
    """Read provenance metadata for an output file."""
    meta_path = output_path + '.meta.json'
    if os.path.exists(meta_path):
        with open(meta_path) as f:
            return json.load(f)
    return None


if __name__ == '__main__':
    if len(sys.argv) >= 3 and sys.argv[1] == 'read':
        meta = read_provenance(sys.argv[2])
        if meta:
            print(json.dumps(meta, indent=2))
        else:
            print(f"No provenance metadata found for {sys.argv[2]}")
    else:
        print("Usage: provenance.py read <output_file>")
