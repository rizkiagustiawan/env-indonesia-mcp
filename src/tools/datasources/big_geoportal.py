#!/usr/bin/env python3
"""BIG Geoportal Engine — Data Resmi Badan Informasi Geospasial Indonesia
Queries ArcGIS REST services at geoservices.big.go.id/rbi/
NO authentication required — public data.

Data sources:
- Administrasi_AR_KabKota_50K: Batas kabupaten/kota (1:50K resmi)
- Administrasi_AR_Kecamatan_10K: Batas kecamatan (1:10K resmi)
- Administrasi_AR_KelDesa_10K: Batas kelurahan/desa (1:10K resmi)
- GARISPANTAI_250K: Garis pantai nasional
- HIDROGRAFI: Sungai dan danau
- SIKAMBING API: Metadata catalog search

Ref: UU 4/2011 tentang Informasi Geospasial, Perpres 27/2014 tentang JIGN
"""
import sys
import os
import json
import requests

BASE_URL = "https://geoservices.big.go.id/rbi/rest/services"
SIKAMBING_URL = "https://geoportal.big.go.id/sikambing/api"
TIMEOUT = 30

# Provenance
try:
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'gis'))
    from provenance import create_provenance
except:
    create_provenance = None


def _make_bbox(lat, lon, buffer_km):
    """Create bbox from center + buffer."""
    d = buffer_km / 111.0
    import math
    dlon = d / math.cos(math.radians(lat))
    return f"{lon-dlon},{lat-d},{lon+dlon},{lat+d}"


def _query_arcgis(service_path, layer_id, bbox, out_fields="*",
                  return_geometry=True, max_features=500, f="geojson"):
    """Generic ArcGIS REST query."""
    url = f"{BASE_URL}/{service_path}/MapServer/{layer_id}/query"
    params = {
        'geometry': bbox,
        'geometryType': 'esriGeometryEnvelope',
        'spatialRel': 'esriSpatialRelIntersects',
        'outFields': out_fields,
        'returnGeometry': str(return_geometry).lower(),
        'resultRecordCount': max_features,
        'f': f,
    }
    try:
        resp = requests.get(url, params=params, timeout=TIMEOUT)
        resp.raise_for_status()
        return resp.json()
    except requests.exceptions.Timeout:
        return {"error": "Timeout — server BIG tidak merespons dalam 30 detik"}
    except Exception as e:
        return {"error": str(e)}


def query_admin_kabkota(lat, lon, buffer_km, output_path=None):
    """Query batas administrasi Kabupaten/Kota dari BIG (1:50K resmi).
    
    Fields: NAMOBJ (nama), WADMPR (provinsi), WADMKK (kab/kota),
            KDPBPS (kode BPS provinsi), KDBBPS (kode BPS kab/kota),
            LUASWH (luas dalam Ha)
    
    Ref: Peta RBI 1:50.000, BIG
    """
    bbox = _make_bbox(lat, lon, buffer_km)
    fields = "NAMOBJ,WADMPR,WADMKK,KDPBPS,KDBBPS,LUASWH,TIPADM"

    data = _query_arcgis("BATASWILAYAH/Administrasi_AR_KabKota_50K", 0,
                         bbox, out_fields=fields, return_geometry=True)

    if "error" in data:
        print(f"ERROR [E502]: BIG GeoServices — {data['error']}")
        return

    features = data.get('features', [])
    print(f"SUCCESS: BIG Admin Kabupaten/Kota (1:50K Resmi)")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Ditemukan: {len(features)} kabupaten/kota\n")

    for feat in features:
        p = feat.get('properties', {})
        tipe = "Kota" if p.get('TIPADM') == 2 else "Kabupaten"
        luas = p.get('LUASWH', 0) or 0
        print(f"  {p.get('NAMOBJ','?'):30} | {tipe:10} | Prov: {p.get('WADMPR','?'):20} | "
              f"BPS: {p.get('KDPBPS','').strip()}.{p.get('KDBBPS','').strip()} | "
              f"Luas: {luas:,.0f} Ha")

    print(f"\nSumber: Badan Informasi Geospasial (BIG)")
    print(f"Service: Administrasi_AR_KabKota_50K")
    print(f"Ref: UU 4/2011, Peta RBI 1:50.000")

    # Save GeoJSON if output path provided
    if output_path and features:
        with open(output_path, 'w') as f:
            json.dump(data, f)
        print(f"GeoJSON: {output_path} ({os.path.getsize(output_path)/1024:.1f} KB)")

    # Provenance
    if create_provenance and output_path:
        try:
            create_provenance(output_path,
                tool='big_admin_kabkota',
                data_source='BIG GeoServices Administrasi_AR_KabKota_50K',
                coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
                scale='1:50.000',
                features_count=len(features),
                references=['UU 4/2011', 'Perpres 27/2014'],
                crs='EPSG:4326')
        except: pass

    return json.dumps({"features_count": len(features), "bbox": bbox})


def query_admin_kecamatan(lat, lon, buffer_km, output_path=None):
    """Query batas administrasi Kecamatan dari BIG (1:10K resmi).
    
    Fields: NAMOBJ, WADMKC (kecamatan), WADMKK (kab/kota), WADMPR (provinsi),
            KDCBPS (kode BPS kecamatan)
    """
    bbox = _make_bbox(lat, lon, buffer_km)
    fields = "NAMOBJ,WADMKC,WADMKK,WADMPR,KDPBPS,KDBBPS,KDCBPS"

    data = _query_arcgis("BATASWILAYAH/Administrasi_AR_Kecamatan_10K", 0,
                         bbox, out_fields=fields, return_geometry=True)

    if "error" in data:
        print(f"ERROR [E502]: BIG GeoServices — {data['error']}")
        return

    features = data.get('features', [])
    print(f"SUCCESS: BIG Admin Kecamatan (1:10K Resmi)")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Ditemukan: {len(features)} kecamatan\n")

    # Group by kab/kota
    by_kab = {}
    for feat in features:
        p = feat.get('properties', {})
        kab = p.get('WADMKK', '?') or '?'
        kec = p.get('WADMKC', p.get('NAMOBJ', '?')) or '?'
        if kab not in by_kab:
            by_kab[kab] = []
        by_kab[kab].append(kec)

    for kab, kecs in sorted(by_kab.items()):
        print(f"  {kab}:")
        for kec in sorted(kecs):
            print(f"    - {kec}")

    print(f"\nTotal: {len(features)} kecamatan di {len(by_kab)} kab/kota")
    print(f"Sumber: BIG, Peta RBI 1:10.000")

    if output_path and features:
        with open(output_path, 'w') as f:
            json.dump(data, f)
        print(f"GeoJSON: {output_path} ({os.path.getsize(output_path)/1024:.1f} KB)")

    if create_provenance and output_path:
        try:
            create_provenance(output_path,
                tool='big_admin_kecamatan',
                data_source='BIG GeoServices Administrasi_AR_Kecamatan_10K',
                coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
                scale='1:10.000',
                features_count=len(features),
                references=['UU 4/2011'],
                crs='EPSG:4326')
        except: pass

    return json.dumps({"features_count": len(features), "bbox": bbox})


def query_coastline(lat, lon, buffer_km, output_path=None):
    """Query garis pantai dari BIG (1:250K nasional).
    
    Untuk analisis pesisir, coastal setback, dan cartography.
    """
    bbox = _make_bbox(lat, lon, buffer_km)

    data = _query_arcgis("GARISPANTAI/GARISPANTAI_250K", 0,
                         bbox, out_fields="NAMOBJ,FCODE,REMARK",
                         return_geometry=True, max_features=1000)

    if "error" in data:
        # Try alternative service
        data = _query_arcgis("GARISPANTAI/GarisPantai_25K", 0,
                             bbox, out_fields="NAMOBJ,FCODE,REMARK",
                             return_geometry=True, max_features=1000)

    if "error" in data:
        print(f"ERROR [E502]: BIG GeoServices — {data['error']}")
        return

    features = data.get('features', [])
    print(f"SUCCESS: BIG Garis Pantai (1:250K Resmi)")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Ditemukan: {len(features)} segmen garis pantai\n")

    # Calculate total length (approximate)
    total_vertices = 0
    for feat in features:
        geom = feat.get('geometry', {})
        gtype = geom.get('type', '')
        if gtype == 'MultiLineString':
            for line in geom.get('coordinates', []):
                total_vertices += len(line)
        elif gtype == 'LineString':
            total_vertices += len(geom.get('coordinates', []))

    print(f"Total vertices: {total_vertices}")
    print(f"Sumber: BIG, GARISPANTAI_250K")

    if output_path and features:
        with open(output_path, 'w') as f:
            json.dump(data, f)
        print(f"GeoJSON: {output_path} ({os.path.getsize(output_path)/1024:.1f} KB)")

    if create_provenance and output_path:
        try:
            create_provenance(output_path,
                tool='big_coastline',
                data_source='BIG GeoServices GARISPANTAI_250K',
                coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
                scale='1:250.000',
                features_count=len(features),
                crs='EPSG:4326')
        except: pass

    return json.dumps({"features_count": len(features), "vertices": total_vertices})


def query_rivers(lat, lon, buffer_km, output_path=None):
    """Query jaringan sungai dari BIG (Hidrografi 1M nasional).
    
    Untuk konteks water quality modeling dan analisis DAS.
    """
    bbox = _make_bbox(lat, lon, buffer_km)

    # Try national 1M services first
    for service in ["HIDROGRAFI/Sungai_1M", "HIDROGRAFI/Danau_1M"]:
        data = _query_arcgis(service.replace("/", "/"), 0,
                             bbox, out_fields="NAMOBJ,FCODE,REMARK",
                             return_geometry=True, max_features=500, f="geojson")
        if "error" not in data and data.get('features'):
            break

    if "error" in data or not data.get('features'):
        # Fallback: try query with simpler path
        url = f"{BASE_URL}/HIDROGRAFI/Sungai_1M/MapServer/0/query"
        try:
            resp = requests.get(url, params={
                'geometry': bbox,
                'geometryType': 'esriGeometryEnvelope',
                'spatialRel': 'esriSpatialRelIntersects',
                'outFields': 'NAMOBJ,FCODE',
                'returnGeometry': 'true',
                'resultRecordCount': 500,
                'f': 'geojson',
            }, timeout=TIMEOUT)
            data = resp.json()
        except Exception as e:
            data = {"error": str(e)}

    if "error" in data:
        print(f"ERROR [E502]: BIG Hidrografi — {data.get('error', 'unknown')}")
        print(f"CATATAN: Data hidrografi BIG terpecah per provinsi.")
        print(f"Coba akses langsung: {BASE_URL}/HIDROGRAFI/?f=pjson")
        return

    features = data.get('features', [])
    print(f"SUCCESS: BIG Hidrografi (Sungai)")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Ditemukan: {len(features)} segmen sungai\n")

    named = [f for f in features if f.get('properties', {}).get('NAMOBJ')]
    if named:
        print("Sungai teridentifikasi:")
        seen = set()
        for feat in named:
            name = feat['properties']['NAMOBJ']
            if name not in seen:
                seen.add(name)
                print(f"  - {name}")

    print(f"\nSumber: BIG, Hidrografi 1:1.000.000")

    if output_path and features:
        with open(output_path, 'w') as f:
            json.dump(data, f)
        print(f"GeoJSON: {output_path} ({os.path.getsize(output_path)/1024:.1f} KB)")

    return json.dumps({"features_count": len(features)})


def search_datasets(keyword=None, xmin=None, ymin=None, xmax=None, ymax=None):
    """Search BIG SIKAMBING metadata catalog.
    
    Search by keyword and/or bounding box.
    Returns dataset titles, types, distribution URLs.
    """
    results = []

    if xmin is not None and ymin is not None and xmax is not None and ymax is not None:
        url = f"{SIKAMBING_URL}/harvestings/bbox/{xmin}/{ymin}/{xmax}/{ymax}"
        try:
            resp = requests.get(url, timeout=TIMEOUT)
            data = resp.json()
            items = data.get('data', data) if isinstance(data, dict) else data
            if isinstance(items, list):
                results = items
        except Exception as e:
            print(f"ERROR: SIKAMBING bbox search — {e}")

    print(f"SUCCESS: SIKAMBING Dataset Search")
    if keyword:
        print(f"Keyword: {keyword}")
    if xmin is not None:
        print(f"BBOX: [{xmin}, {ymin}, {xmax}, {ymax}]")
    print(f"Ditemukan: {len(results)} dataset\n")

    for item in results:
        title = item.get('title', '?')
        dtype = item.get('data_type', '?')
        org = item.get('organizations', {})
        org_name = org.get('name', '?') if isinstance(org, dict) else '?'
        pub_date = item.get('publication_date', '?')

        print(f"  [{dtype:12}] {title}")
        print(f"               Organisasi: {org_name}")
        print(f"               Tanggal: {pub_date}")

        # Parse distributions for URLs
        dist_str = item.get('distributions', '[]')
        try:
            dists = json.loads(dist_str) if isinstance(dist_str, str) else (dist_str or [])
            for d in dists:
                protocol = d.get('protocol', '?')
                url = d.get('url', '')
                if url:
                    print(f"               → [{protocol}] {url[:80]}")
        except:
            pass
        print()

    print(f"Sumber: BIG SIKAMBING (geoportal.big.go.id)")

    return json.dumps({"count": len(results)})


def query_admin_desa(lat, lon, buffer_km, output_path=None):
    """Query batas administrasi Kelurahan/Desa dari BIG (1:10K resmi).
    Level administrasi terkecil — wajib untuk AMDAL.
    """
    bbox = _make_bbox(lat, lon, buffer_km)
    fields = "NAMOBJ,WADMKD,WADMKC,WADMKK,WADMPR,KDPBPS,KDBBPS,KDCBPS,KDEBPS,LUASWH,UUPP"

    data = _query_arcgis("BATASWILAYAH/Administrasi_AR_KelDesa_10K", 0,
                         bbox, out_fields=fields, return_geometry=True, max_features=500)

    if "error" in data:
        print(f"ERROR [E502]: BIG GeoServices — {data['error']}")
        return

    features = data.get('features', [])
    print(f"SUCCESS: BIG Admin Kelurahan/Desa (1:10K Resmi)")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Ditemukan: {len(features)} kelurahan/desa\n")

    by_kec = {}
    for feat in features:
        p = feat.get('properties', {})
        kec = p.get('WADMKC', '?') or '?'
        desa = p.get('WADMKD', p.get('NAMOBJ', '?')) or '?'
        luas = p.get('LUASWH', 0) or 0
        kode = f"{(p.get('KDPBPS') or '').strip()}.{(p.get('KDBBPS') or '').strip()}.{(p.get('KDCBPS') or '').strip()}.{(p.get('KDEBPS') or '').strip()}"
        if kec not in by_kec:
            by_kec[kec] = []
        by_kec[kec].append((desa, luas, kode))

    for kec, desas in sorted(by_kec.items()):
        print(f"  Kecamatan {kec}:")
        for desa, luas, kode in sorted(desas):
            print(f"    - {desa:25} | {luas:>8.0f} Ha | BPS: {kode}")

    print(f"\nTotal: {len(features)} kel/desa di {len(by_kec)} kecamatan")
    print(f"Sumber: BIG, Peta RBI 1:10.000")

    if output_path and features:
        with open(output_path, 'w') as f:
            json.dump(data, f)
        print(f"GeoJSON: {output_path} ({os.path.getsize(output_path)/1024:.1f} KB)")

    return json.dumps({"features_count": len(features)})


def query_maritime_boundaries(lat, lon, buffer_km, output_path=None):
    """Query batas laut Indonesia dari BIG: Laut Teritorial, Zona Tambahan,
    Landas Kontinen, ZEE. Ref: UNCLOS, UU 6/1996.
    """
    bbox = _make_bbox(lat, lon, buffer_km)

    layers = {
        0: "Laut Teritorial (12 nm)",
        1: "Zona Tambahan (24 nm)",
        2: "Landas Kontinen (200 nm)",
        3: "ZEE (200 nm)",
    }

    all_features = []
    print(f"SUCCESS: BIG Batas Laut Indonesia")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km\n")

    for layer_id, name in layers.items():
        data = _query_arcgis("BATASWILAYAH/BatasNegaraLaut", layer_id,
                             bbox, out_fields="*", return_geometry=True, max_features=100)
        feats = data.get('features', [])
        print(f"  {name}: {len(feats)} segmen")
        for feat in feats:
            feat['properties'] = feat.get('properties', {})
            feat['properties']['boundary_type'] = name
        all_features.extend(feats)

    print(f"\nTotal: {len(all_features)} segmen batas laut")
    print(f"Ref: UNCLOS 1982, UU 6/1996, BIG")

    if output_path and all_features:
        out_data = {"type": "FeatureCollection", "features": all_features}
        with open(output_path, 'w') as f:
            json.dump(out_data, f)
        print(f"GeoJSON: {output_path}")

    return json.dumps({"features_count": len(all_features)})


def query_outer_islands(lat, lon, buffer_km, output_path=None):
    """Query Pulau-Pulau Kecil Terluar (PPKT) dari BIG.
    111 pulau per Keppres No. 6 Tahun 2017.
    """
    bbox = _make_bbox(lat, lon, buffer_km)

    data = _query_arcgis("PERAIRAN/GRI_2024_PPKT", 0,
                         bbox, out_fields="*", return_geometry=True, max_features=200)

    if "error" in data:
        print(f"ERROR [E502]: BIG PPKT — {data['error']}")
        return

    features = data.get('features', [])
    print(f"SUCCESS: BIG Pulau Kecil Terluar (PPKT)")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Ditemukan: {len(features)} pulau terluar\n")

    for feat in features:
        p = feat.get('properties', {})
        name = p.get('NAMOBJ', p.get('nama', '?'))
        print(f"  - {name}")
        for k, v in p.items():
            if k not in ('OBJECTID', 'SHAPE', 'Shape') and v:
                print(f"      {k}: {v}")

    print(f"\nRef: Keppres No. 6 Tahun 2017, BIG")

    if output_path and features:
        with open(output_path, 'w') as f:
            json.dump(data, f)
        print(f"GeoJSON: {output_path}")

    return json.dumps({"features_count": len(features)})


def search_place_name(name, output_path=None):
    """Geocoding resmi — cari koordinat dari nama tempat via Gazetir RI 2023.
    10,081 nama rupabumi baku (NRB) yang ditetapkan SK BIG.
    """
    # Search in Gazetir (point features = layer 0)
    url = f"{BASE_URL}/TOPONIMI/Gasetir_RI_2023_Penetapan_2/MapServer/0/query"
    try:
        resp = requests.get(url, params={
            'where': f"UPPER(NAMGAZ) LIKE '%{name.upper()}%' OR UPPER(NAMSPE) LIKE '%{name.upper()}%'",
            'outFields': 'NAMGAZ,NAMSPE,NAMLOK,FTYPE,KOORDX,KOORDY,ELEVAS,ASLBHS',
            'returnGeometry': 'true',
            'resultRecordCount': 20,
            'f': 'json',
        }, timeout=TIMEOUT)
        raw = resp.json()
        # Convert ArcGIS JSON to pseudo-GeoJSON features
        features = []
        for feat in raw.get('features', []):
            a = feat.get('attributes', {})
            g = feat.get('geometry', {})
            features.append({
                'properties': a,
                'geometry': {'type': 'Point', 'coordinates': [g.get('x', a.get('KOORDX', 0)), g.get('y', a.get('KOORDY', 0))]}
            })
        data = {'features': features}
    except Exception as e:
        print(f"ERROR [E502]: Gazetir search — {e}")
        return

    features = data.get('features', [])
    print(f"SUCCESS: BIG Gazetir RI 2023 — Pencarian Nama Rupabumi")
    print(f"Query: '{name}'")
    print(f"Ditemukan: {len(features)} hasil\n")

    for feat in features:
        p = feat.get('properties', {})
        geom = feat.get('geometry', {})
        coords = geom.get('coordinates', [None, None])
        lon_v = coords[0] if len(coords) > 0 else p.get('KOORDX', '?')
        lat_v = coords[1] if len(coords) > 1 else p.get('KOORDY', '?')
        namgaz = p.get('NAMGAZ', p.get('NAMSPE', '?'))
        ftype = p.get('FTYPE', '')
        elev = p.get('ELEVAS', '')
        bahasa = p.get('ASLBHS', '')
        print(f"  {namgaz}")
        print(f"    Tipe: {ftype}")
        print(f"    Koordinat: {lat_v}, {lon_v}")
        if elev: print(f"    Elevasi: {elev} m")
        if bahasa: print(f"    Asal bahasa: {bahasa}")
        print()

    print(f"Ref: SK BIG, Gazetir RI 2023 Penetapan 2")

    if output_path and features:
        with open(output_path, 'w') as f:
            json.dump(data, f)
        print(f"GeoJSON: {output_path}")

    return json.dumps({"features_count": len(features)})


def query_sdgs_desa(lat, lon, buffer_km, output_path=None):
    """Query indikator SDGs tingkat desa dari BIG/IGT.
    20 indikator: air minum, sanitasi, listrik, kemiskinan, pendidikan, kesehatan.
    Untuk analisis sosial-lingkungan AMDAL/KLHS.
    """
    bbox = _make_bbox(lat, lon, buffer_km)

    # Layer 0 = main SDGs indicator layer
    data = _query_arcgis("IGT/SDGs_Desa", 0,
                         bbox, out_fields="*", return_geometry=False, max_features=50)

    if "error" in data:
        print(f"ERROR [E502]: BIG SDGs Desa — {data['error']}")
        return

    features = data.get('features', [])
    print(f"SUCCESS: BIG SDGs Desa (Indikator TPB/SDGs Tingkat Desa)")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Ditemukan: {len(features)} desa\n")

    for feat in features:
        p = feat.get('properties', feat.get('attributes', {}))
        nama = p.get('NAMOBJ', p.get('namobj', p.get('nama_desa', '?')))
        print(f"  Desa: {nama}")
        for k, v in p.items():
            if k.upper() not in ('OBJECTID', 'SHAPE', 'SHAPE.AREA', 'SHAPE.LEN') and v is not None:
                print(f"    {k}: {v}")
        print()

    print(f"Ref: BIG IGT, SDGs Desa")

    if output_path and features:
        with open(output_path, 'w') as f:
            json.dump(data, f)
        print(f"Output: {output_path}")

    return json.dumps({"features_count": len(features)})


def query_demnas_coverage(lat, lon, buffer_km=50):
    """Cek ketersediaan tile DEMNAS (DEM Nasional) dari BIG.
    DEMNAS resolusi 0.27 arcsec (~8m) — lebih baik dari SRTM 30m.
    """
    bbox = _make_bbox(lat, lon, buffer_km)

    data = _query_arcgis("INDEKS/DEM_Nasional", 0,
                         bbox, out_fields="*", return_geometry=False, max_features=100)

    if "error" in data:
        print(f"ERROR [E502]: BIG DEMNAS Index — {data['error']}")
        return

    features = data.get('features', [])
    print(f"SUCCESS: BIG DEMNAS Coverage Check")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Tile DEMNAS tersedia: {len(features)}\n")

    for feat in features:
        p = feat.get('properties', feat.get('attributes', {}))
        for k, v in sorted(p.items()):
            if k.upper() not in ('OBJECTID', 'SHAPE', 'SHAPE.AREA', 'SHAPE.LEN') and v is not None:
                print(f"  {k}: {v}")
        print()

    print(f"DEMNAS: 0.27 arcsec (~8m resolusi)")
    print(f"Download: tanahair.indonesia.go.id/demnas")
    print(f"Ref: BIG, Perpres 27/2014")

    return json.dumps({"tiles_count": len(features)})


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage:")
        print("  big_geoportal.py kabkota lat lon buffer_km [output.geojson]")
        print("  big_geoportal.py kecamatan lat lon buffer_km [output.geojson]")
        print("  big_geoportal.py desa lat lon buffer_km [output.geojson]")
        print("  big_geoportal.py coastline lat lon buffer_km [output.geojson]")
        print("  big_geoportal.py rivers lat lon buffer_km [output.geojson]")
        print("  big_geoportal.py maritime lat lon buffer_km [output.geojson]")
        print("  big_geoportal.py ppkt lat lon buffer_km [output.geojson]")
        print("  big_geoportal.py gazetir <nama_tempat>")
        print("  big_geoportal.py sdgs lat lon buffer_km")
        print("  big_geoportal.py demnas lat lon [buffer_km]")
        print("  big_geoportal.py search xmin ymin xmax ymax")
        sys.exit(1)

    mode = sys.argv[1]
    try:
        if mode == 'kabkota':
            out = sys.argv[5] if len(sys.argv) > 5 else None
            query_admin_kabkota(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), out)
        elif mode == 'kecamatan':
            out = sys.argv[5] if len(sys.argv) > 5 else None
            query_admin_kecamatan(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), out)
        elif mode == 'desa':
            out = sys.argv[5] if len(sys.argv) > 5 else None
            query_admin_desa(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), out)
        elif mode == 'coastline':
            out = sys.argv[5] if len(sys.argv) > 5 else None
            query_coastline(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), out)
        elif mode == 'rivers':
            out = sys.argv[5] if len(sys.argv) > 5 else None
            query_rivers(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), out)
        elif mode == 'maritime':
            out = sys.argv[5] if len(sys.argv) > 5 else None
            query_maritime_boundaries(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), out)
        elif mode == 'ppkt':
            out = sys.argv[5] if len(sys.argv) > 5 else None
            query_outer_islands(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), out)
        elif mode == 'gazetir':
            out = sys.argv[3] if len(sys.argv) > 3 else None
            search_place_name(sys.argv[2], out)
        elif mode == 'sdgs':
            query_sdgs_desa(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]))
        elif mode == 'demnas':
            buf = float(sys.argv[4]) if len(sys.argv) > 4 else 50
            query_demnas_coverage(float(sys.argv[2]), float(sys.argv[3]), buf)
        elif mode == 'search':
            search_datasets(xmin=float(sys.argv[2]), ymin=float(sys.argv[3]),
                          xmax=float(sys.argv[4]), ymax=float(sys.argv[5]))
        else:
            print(f"ERROR: Mode '{mode}' tidak dikenal.")
    except Exception as e:
        print(f"ERROR [E502]: {e}")
        import traceback
        traceback.print_exc()
