#!/usr/bin/env python3
"""NASA EMIT Hyperspectral Data Extractor & Visualizer"""

import sys
import argparse
import ee
import json
import numpy as np
import matplotlib.pyplot as plt

try:
    ee.Initialize()
except Exception:
    print("ERROR: Google Earth Engine belum diotentikasi.")
    sys.exit(1)

def extract_hyperspectral_signature(lon, lat, output_img_path):
    try:
        point = ee.Geometry.Point([lon, lat])
        
        emit_coll = ee.ImageCollection('NASA/EMIT/L2A/RFL') \
            .filterBounds(point) \
            .sort('system:time_start', False)
            
        count = emit_coll.size().getInfo()
        if count == 0:
            return "ERROR: Tidak ada data NASA EMIT yang tersedia untuk koordinat ini."
            
        latest_img = ee.Image(emit_coll.first())
        date_acquired = latest_img.date().format('YYYY-MM-dd HH:mm:ss').getInfo()
        
        pixel_values = latest_img.reduceRegion(
            reducer=ee.Reducer.first(),
            geometry=point,
            scale=60 
        ).getInfo()
        
        if not pixel_values or 'reflectance_0' not in pixel_values or pixel_values['reflectance_0'] is None:
            return "ERROR: Piksel kosong (nodata) atau tertutup awan pekat."

        bands_data = []
        wavelengths = np.linspace(381, 2493, 285) # Estimasi wavelength EMIT nm
        
        for i in range(285):
            b_name = f'reflectance_{i}'
            if b_name in pixel_values and pixel_values[b_name] is not None:
                val = pixel_values[b_name]
                bands_data.append(val)
            else:
                bands_data.append(np.nan)
                
        vals = np.array(bands_data)
        
        # Masking Water Absorption Bands 
        vals[130:146] = np.nan
        vals[190:211] = np.nan

        # === GENERATE GRAPHIC ===
        plt.figure(figsize=(10, 5))
        plt.plot(wavelengths, vals, color='blue', linewidth=1.5, label='Spectral Reflectance')
        
        # Highlight specific absorption regions
        plt.axvspan(2100, 2250, color='red', alpha=0.2, label='Clay/Carbonate Absorption (SWIR-2)')
        plt.axvspan(800, 950, color='orange', alpha=0.2, label='Iron Oxide Absorption')
        
        plt.title(f'NASA EMIT Hyperspectral Signature\nLat: {lat}, Lon: {lon} | Date: {date_acquired}', fontweight='bold')
        plt.xlabel('Wavelength (nm)')
        plt.ylabel('Reflectance')
        plt.grid(True, linestyle='--', alpha=0.6)
        plt.legend()
        plt.tight_layout()
        plt.savefig(output_img_path, dpi=200)
        plt.close()

        output = "=== NASA EMIT Hyperspectral Signature ===\n"
        output += f"Tanggal Akuisisi: {date_acquired}\n"
        output += f"Koordinat: {lat}, {lon}\n"
        output += "Ringkasan Spektrum (Reflektansi rata-rata):\n"
        
        vnir_avg = np.nanmean(vals[0:90])
        swir1_avg = np.nanmean(vals[90:130])
        swir2_avg = np.nanmean(vals[211:285])
        
        output += f"  - VNIR: {vnir_avg:.4f}\n"
        output += f"  - SWIR-1: {swir1_avg:.4f}\n"
        output += f"  - SWIR-2: {swir2_avg:.4f}\n\n"
        
        if swir1_avg > 0 and swir2_avg > 0 and (swir2_avg / swir1_avg) < 0.8:
            output += "🔍 INDIKASI: Penurunan reflektansi di SWIR-2 (indikasi mineral Al-OH / Mg-OH, kaolinite).\n"
        elif vnir_avg > 0 and swir1_avg > 0 and (vnir_avg / swir1_avg) > 1.5:
             output += "🌿 INDIKASI: Profil vegetasi hidup (Red Edge tajam).\n"
        else:
            output += "🔍 INDIKASI: Spektrum relatif datar.\n"
            
        output += f"\nSUCCESS: Peta layout standar berhasil disimpan di {output_img_path}"
        return output

    except Exception as e:
        return f"ERROR: Terjadi kesalahan: {str(e)}"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--lon", type=float, required=True)
    parser.add_argument("--lat", type=float, required=True)
    parser.add_argument("--output", type=str, required=True)
    args = parser.parse_args()
    
    print(extract_hyperspectral_signature(args.lon, args.lat, args.output))
