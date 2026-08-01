#!/usr/bin/env python3
"""Provenance Metadata Generator — logs processing chain for reproducibility.
Every analysis output should be accompanied by a .meta.json file.
"""
import json, os, sys, platform
from datetime import datetime
from typing import Dict, List, Optional, Any

__version__ = '1.0.0'

class Provenance:
    def __init__(self, source_kind: str, source_identifier: str, acquisition_timestamp: str, fallback_reason: Optional[str] = None, max_age_days: Optional[int] = None):
        if source_kind.lower() == 'fallback' and not fallback_reason:
            raise ValueError("Fallback sources require an explicit fallback reason")
        self.source_kind = source_kind
        self.source_identifier = source_identifier
        self.acquisition_timestamp = acquisition_timestamp
        self.fallback_reason = fallback_reason
        self.max_age_days = max_age_days

    def to_dict(self) -> Dict[str, Any]:
        return {k: v for k, v in self.__dict__.items() if v is not None}

class Uncertainty:
    def __init__(self, uncertainty_type: str, lower: float, upper: float, method: str, confidence_level: Optional[float] = None, seed: Optional[int] = None):
        if lower > upper:
            raise ValueError("Uncertainty lower bound cannot be greater than upper bound")
        if uncertainty_type in ['confidence_interval', 'credible_interval'] and seed is None:
            raise ValueError("Stochastic uncertainty requires a reproducible seed")
            
        self.uncertainty_type = uncertainty_type
        self.lower = lower
        self.upper = upper
        self.method = method
        self.confidence_level = confidence_level
        self.seed = seed

    def to_dict(self) -> Dict[str, Any]:
        return {k: v for k, v in self.__dict__.items() if v is not None}

class ScientificResult:
    def __init__(self, parameter: str, value: float, unit: str, status: str = "valid", uncertainty: Optional[Uncertainty] = None, provenance: Optional[Provenance] = None):
        import math
        if not math.isfinite(value):
            raise ValueError("Value must be finite")
            
        self.parameter = parameter
        self.value = value
        self.unit = unit
        self.status = status
        self.uncertainty = uncertainty
        self.provenance = provenance
        self.claims: List[Dict[str, str]] = []

    def add_claim(self, claim_type: str, description: str):
        if self.status == "screening_only" and claim_type.lower() in ["compliant", "approved", "safe", "legal"]:
            raise ValueError(f"Regulatory claim '{claim_type}' forbidden for screening-only results")
        self.claims.append({"claim_type": claim_type, "description": description})

    def to_dict(self) -> Dict[str, Any]:
        d = {
            "parameter": self.parameter,
            "value": self.value,
            "unit": self.unit,
            "status": self.status,
            "claims": self.claims
        }
        if self.uncertainty:
            d["uncertainty"] = self.uncertainty.to_dict()
        if self.provenance:
            d["provenance"] = self.provenance.to_dict()
        return d

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
