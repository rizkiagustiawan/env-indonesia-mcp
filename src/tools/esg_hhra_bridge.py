#!/usr/bin/env python3
import sys
import argparse
import json

def calculate_hhra(contaminant, route, concentration, ir, ef, ed, bw, at, csf):
    # CDI = (C * IR * EF * ED) / (BW * AT * 365)
    cdi = (concentration * ir * ef * ed) / (bw * at * 365.0)
    ilcr = cdi * csf

    # Perbandingan dengan standar Indonesia (Kemenkes: 55 kg)
    bw_id = 55.0
    cdi_id = (concentration * ir * ef * ed) / (bw_id * at * 365.0)
    ilcr_id = cdi_id * csf

    out = f"=== HUMAN HEALTH RISK ASSESSMENT (HHRA) ===\n"
    out += f"Kontaminan: {contaminant} | Jalur: {route}\n"
    out += f"Konsentrasi: {concentration} mg/m3 | Durasi Paparan: {ed} tahun\n\n"
    out += f"1. HASIL DENGAN BERAT BADAN INPUT ({bw} kg):\n"
    out += f"   - CDI  : {cdi:.6e} mg/kg/hari\n"
    out += f"   - ILCR : {ilcr:.6e} (Karsinogenik Risiko)\n\n"
    out += f"2. HASIL DENGAN STANDAR KEMENKES INDONESIA ({bw_id} kg):\n"
    out += f"   - CDI  : {cdi_id:.6e} mg/kg/hari\n"
    out += f"   - ILCR : {ilcr_id:.6e} (Karsinogenik Risiko)\n\n"
    
    diff = ((cdi_id - cdi) / cdi) * 100
    out += f"KESIMPULAN FATAL (DEMOGRAPHIC BIAS):\n"
    if bw > bw_id:
        out += f"Menggunakan standar berat badan asing ({bw} kg) meremehkan paparan racun (CDI) pada warga lokal sebesar {diff:.1f}%.\n"
    
    if ilcr_id > 1e-4:
        out += "STATUS RISIKO LOKAL: TIDAK DAPAT DITERIMA (Unacceptable > 10^-4). Remediasi Wajib!\n"
    elif ilcr_id > 1e-6:
        out += "STATUS RISIKO LOKAL: DAPAT DIKELOLA (Manageable 10^-6 - 10^-4).\n"
    else:
        out += "STATUS RISIKO LOKAL: AMAN (< 10^-6).\n"

    return out


def evaluate_tcfd(company, sector, asset, carbon, lat, lon):
    out = f"=== TCFD CLIMATE RISK & ESG ASSESSMENT ===\n"
    out += f"Company: {company} | Sector: {sector}\n"
    out += f"Location: {lat}, {lon}\n"
    out += f"Asset Value at Risk: ${asset:,.2f} USD\n"
    out += f"Carbon Footprint (Scope 1&2): {carbon:,.2f} tCO2e\n\n"

    out += f"1. PHYSICAL RISKS (IPCC SSP5-8.5 Worst-Case)\n"
    if "mining" in sector.lower() or "tambang" in sector.lower():
        out += f"   - Drought & Water Scarcity (HIGH): Konsentrator tambang butuh air masif. El Nino berkepanjangan akan menghentikan produksi.\n"
        out += f"   - Extreme Precipitation (MEDIUM): Risiko luapan Acid Mine Drainage (AMD) dan banjir open-pit saat La Nina.\n"
        out += f"   - Earthquake/Tsunami (HIGH): Risiko likuifaksi pada bendungan tailing akibat aktivitas seismik regional.\n"
    else:
        out += f"   - Sea Level Rise (HIGH): Kenaikan muka air laut menenggelamkan infrastruktur pelabuhan.\n"
        out += f"   - Urban Heat Island (MEDIUM): Peningkatan suhu mengurangi efisiensi termal alat berat.\n"

    out += f"\n2. TRANSITION RISKS (Carbon Tax Exposure)\n"
    tax_id = 2.0  # $2 USD (IDR 30k)
    tax_global = 50.0  # $50 USD
    out += f"   - Regulative Risk (Domestik) : Pajak Karbon Indonesia ($2/t) -> Kewajiban ${carbon * tax_id:,.2f} USD/tahun.\n"
    out += f"   - Regulative Risk (Global)   : Shadow Carbon Price ($50/t) -> Potensi Eksposur ${carbon * tax_global:,.2f} USD/tahun.\n"
    out += f"   - Stranded Asset Risk: Medium-High. Tekanan investor untuk mempensiunkan captive power plant (PLTU batubara) lebih awal.\n"

    out += f"\n3. TCFD DISCLOSURE RECOMMENDATIONS\n"
    out += f"   - GOVERNANCE: Komite Direksi wajib mengawasi keamanan bendungan tailing dan manajemen air.\n"
    out += f"   - STRATEGY: Rencanakan transisi dari captive PLTU ke Solar/Geothermal untuk menghindari eksposur karbon ${carbon * tax_global:,.2f} USD.\n"

    return out

def main():
    parser = argparse.ArgumentParser(description="ESG TCFD & HHRA Bridge Tool")
    parser.add_argument("--mode", required=True, choices=["tcfd", "hhra"], help="Mode operasi: tcfd atau hhra")
    parser.add_argument("--json", required=True, help="Input parameter JSON string")

    args = parser.parse_args()
    data = json.loads(args.json)

    if args.mode == "tcfd":
        res = evaluate_tcfd(
            company=data.get("company", "Unknown"),
            sector=data.get("sector", "General"),
            asset=float(data.get("asset_usd", 0.0)),
            carbon=float(data.get("carbon_tonnes", 0.0)),
            lat=float(data.get("lat", 0.0)),
            lon=float(data.get("lon", 0.0))
        )
    elif args.mode == "hhra":
        res = calculate_hhra(
            contaminant=data.get("contaminant", "Benzene"),
            route=data.get("route", "Inhalation"),
            concentration=float(data.get("concentration", 0.0)),
            ir=float(data.get("intake_rate", 20.0)),
            ef=float(data.get("exposure_freq_days", 350.0)),
            ed=float(data.get("exposure_dur_years", 30.0)),
            bw=float(data.get("body_weight_kg", 70.0)),
            at=float(data.get("avg_time_years", 70.0)),
            csf=float(data.get("csf", 0.029))
        )
    print(res)

if __name__ == "__main__":
    main()
