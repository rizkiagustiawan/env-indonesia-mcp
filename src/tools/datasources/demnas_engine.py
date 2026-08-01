#!/usr/bin/env python3
"""DEMNAS Engine — Download & manage DEM Nasional (8m) from BIG.
Resolution: 0.27 arcsec (~8.1m) — superior to SRTM 30m.
Source: tanahair.indonesia.go.id (Badan Informasi Geospasial)
Auth: reCAPTCHA v3 → JWT login → tile download.

4781 tiles covering all Indonesia. ~15-25 MB per tile compressed.
Tiles cached in ~/.demnas_cache/ — never re-downloaded.

Ref: Perpres 27/2014 tentang JIGN, UU 4/2011 tentang IG
"""
import sys
import os
import json
import math
import time
import requests
from pathlib import Path

# Paths
CACHE_DIR = Path.home() / ".demnas_cache"
TILES_DIR = CACHE_DIR / "tiles"
INDEX_FILE = CACHE_DIR / "demnas_index.json"
TOKEN_FILE = CACHE_DIR / "jwt_token.json"
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# URLs
INDEX_URL = "https://tanahair.indonesia.go.id/portal-web/demnas.json"
API_URL = "https://tanahair.indonesia.go.id/api-inageo"
LOGIN_URL = f"{API_URL}/auth/signin"
DOWNLOAD_URL = f"{API_URL}/unduh/demnas"
RECAPTCHA_SITEKEY = "6LeH8XwhAAAAALw43tTI0iPcLqx8vrlMvkyRwuB6"
LOGIN_PAGE = "https://tanahair.indonesia.go.id/portal-web/login"

# Credentials must come from the process environment. Never commit DEMNAS
# account secrets to source or persist them in the repository.
EMAIL_ENV = "DEMNAS_EMAIL"
PASSWORD_ENV = "DEMNAS_PASSWORD"

# Provenance
try:
    sys.path.insert(0, os.path.join(SCRIPT_DIR, '..', 'gis'))
    from provenance import create_provenance
except:
    create_provenance = None


def _ensure_dirs():
    CACHE_DIR.mkdir(exist_ok=True)
    TILES_DIR.mkdir(exist_ok=True)


def _load_tile_index():
    """Load DEMNAS tile index (4781 tiles). Cache locally."""
    _ensure_dirs()

    # Use cached index if < 7 days old
    if INDEX_FILE.exists():
        age_days = (time.time() - INDEX_FILE.stat().st_mtime) / 86400
        if age_days < 7:
            with open(INDEX_FILE) as f:
                return json.load(f)

    print("Mengunduh DEMNAS tile index dari BIG...")
    try:
        resp = requests.get(INDEX_URL, timeout=30)
        resp.raise_for_status()
        data = resp.json()
        with open(INDEX_FILE, 'w') as f:
            json.dump(data, f)
        print(f"Index: {len(data.get('features', []))} tiles cached")
        return data
    except Exception as e:
        print(f"ERROR: Gagal download index — {e}")
        if INDEX_FILE.exists():
            with open(INDEX_FILE) as f:
                return json.load(f)
        return None


def _find_tiles(lat, lon, buffer_km):
    """Find DEMNAS tiles covering the area."""
    index = _load_tile_index()
    if not index:
        return []

    d = buffer_km / 111.0
    dlon = d / math.cos(math.radians(lat))
    bbox = (lon - dlon, lat - d, lon + dlon, lat + d)

    matching = []
    for feat in index.get('features', []):
        geom = feat.get('geometry', {})
        props = feat.get('properties', {})
        name_file = props.get('NAME_FILE')

        # Skip tiles without file
        if not name_file:
            continue

        # Get tile bbox from geometry
        coords = geom.get('coordinates', [])
        if not coords:
            continue

        # Extract bbox from polygon coordinates
        try:
            if geom.get('type') == 'Polygon':
                ring = coords[0]
            elif geom.get('type') == 'MultiPolygon':
                ring = coords[0][0]
            else:
                continue

            xs = [p[0] for p in ring]
            ys = [p[1] for p in ring]
            tile_bbox = (min(xs), min(ys), max(xs), max(ys))

            # Check intersection
            if (tile_bbox[0] <= bbox[2] and tile_bbox[2] >= bbox[0] and
                    tile_bbox[1] <= bbox[3] and tile_bbox[3] >= bbox[1]):
                matching.append({
                    'namobj': props.get('NAMOBJ', '?'),
                    'name_file': name_file,
                    'region': props.get('REGION', '?'),
                    'sensor': props.get('SENSOR', '?'),
                    'skala': props.get('SKALA', '?'),
                    'tahun': props.get('Tahun', '?'),
                    'bbox': tile_bbox,
                })
        except:
            continue

    return matching


def _get_jwt_token():
    """Get JWT access token. Cache for 50 minutes."""
    _ensure_dirs()

    # Check cached token
    if TOKEN_FILE.exists():
        with open(TOKEN_FILE) as f:
            cached = json.load(f)
        age_min = (time.time() - cached.get('timestamp', 0)) / 60
        if age_min < 50:  # JWT valid ~1 hour, refresh at 50 min
            return cached.get('token')

    email = os.environ.get(EMAIL_ENV, "").strip()
    password = os.environ.get(PASSWORD_ENV, "")
    if not email or not password:
        print(
            f"ERROR: Kredensial DEMNAS belum diatur. Set {EMAIL_ENV} dan {PASSWORD_ENV}."
        )
        return None

    print("Login ke tanahair.indonesia.go.id...")

    # Get reCAPTCHA token via captcha_client
    sys.path.insert(0, SCRIPT_DIR)
    from captcha_client import solve_recaptcha_v3
    recaptcha_token = solve_recaptcha_v3(LOGIN_PAGE, RECAPTCHA_SITEKEY, "submit")

    if not recaptcha_token:
        print("ERROR: Gagal mendapatkan reCAPTCHA token")
        return None

    # Login
    try:
        from curl_cffi import requests as cffi_requests
        resp = cffi_requests.post(LOGIN_URL, json={
            "email": email,
            "password": password,
            "token": recaptcha_token
        }, impersonate="chrome", timeout=60)
        data = resp.json()
    except Exception:
        resp = requests.post(LOGIN_URL, json={
            "email": email,
            "password": password,
            "token": recaptcha_token
        }, timeout=60)
        data = resp.json()

    jwt = data.get('accessToken')
    if not jwt:
        print(f"ERROR: Login gagal — {data.get('message', data)}")
        return None

    print(f"Login berhasil: {data.get('fullname', '?')}")

    # Cache token
    with open(TOKEN_FILE, 'w') as f:
        json.dump({'token': jwt, 'timestamp': time.time()}, f)

    return jwt


def _download_tile(tile_info, jwt):
    """Download single DEMNAS tile. Skip if cached."""
    namobj = tile_info['namobj']
    filename = f"DEMNAS_{namobj}_v1.0.tif"
    tile_path = TILES_DIR / filename

    if tile_path.exists():
        size_mb = tile_path.stat().st_size / (1024 * 1024)
        print(f"  Cache hit: {filename} ({size_mb:.1f} MB)")
        return str(tile_path)

    print(f"  Downloading: {filename}...")
    url = f"{DOWNLOAD_URL}?token={jwt}&filename={filename}"

    try:
        from curl_cffi import requests as cffi_requests
        resp = cffi_requests.get(url, impersonate="chrome", timeout=300)
    except Exception:
        resp = requests.get(url, timeout=300)

    if resp.status_code != 200:
        print(f"  ERROR: HTTP {resp.status_code}")
        return None

    content = resp.content
    size_mb = len(content) / (1024 * 1024)

    if size_mb < 0.1:
        print(f"  ERROR: File terlalu kecil ({size_mb:.2f} MB) — mungkin error response")
        return None

    with open(tile_path, 'wb') as f:
        f.write(content)

    print(f"  Downloaded: {filename} ({size_mb:.1f} MB)")
    return str(tile_path)


def _merge_tiles(tile_paths, output_path):
    """Merge multiple DEMNAS tiles into single GeoTIFF using rasterio."""
    if len(tile_paths) == 1:
        import shutil
        shutil.copy2(tile_paths[0], output_path)
        return output_path

    try:
        import rasterio
        from rasterio.merge import merge

        datasets = [rasterio.open(p) for p in tile_paths]
        merged, transform = merge(datasets)

        profile = datasets[0].profile.copy()
        profile.update({
            'height': merged.shape[1],
            'width': merged.shape[2],
            'transform': transform,
            'driver': 'GTiff',
            'compress': 'deflate',
        })

        with rasterio.open(output_path, 'w', **profile) as dst:
            dst.write(merged)

        for ds in datasets:
            ds.close()

        return output_path
    except ImportError:
        print("WARNING: rasterio tidak tersedia — tile tidak di-merge")
        print(f"Tile files: {tile_paths}")
        return tile_paths[0]
    except Exception as e:
        print(f"ERROR merge: {e}")
        return tile_paths[0]


def get_demnas_info(lat, lon, buffer_km):
    """Get info about available DEMNAS tiles for an area (no download)."""
    tiles = _find_tiles(lat, lon, buffer_km)

    print(f"=== DEMNAS Coverage Info ===")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Tiles ditemukan: {len(tiles)}\n")

    total_size_est = 0
    for t in tiles:
        cached = (TILES_DIR / f"DEMNAS_{t['namobj']}_v1.0.tif").exists()
        status = "CACHED" if cached else "belum download"
        print(f"  {t['namobj']:12} | {t['region']:12} | {t['sensor']:12} | "
              f"{t['skala']:5} | {t['tahun']:5} | {status}")
        total_size_est += 20  # ~20 MB per tile estimate

    cached_count = sum(1 for t in tiles if (TILES_DIR / f"DEMNAS_{t['namobj']}_v1.0.tif").exists())

    print(f"\nEstimasi total download: ~{total_size_est} MB")
    print(f"Sudah di-cache: {cached_count}/{len(tiles)} tiles")
    print(f"Cache dir: {CACHE_DIR}")
    print(f"Resolusi: 0.27 arcsec (~8.1m)")
    print(f"Sumber: BIG (tanahair.indonesia.go.id)")

    return json.dumps({
        "tiles_count": len(tiles),
        "cached_count": cached_count,
        "estimated_size_mb": total_size_est,
        "tiles": [t['namobj'] for t in tiles],
    })


def download_demnas(lat, lon, buffer_km, output_path=None):
    """Download DEMNAS tiles for an area, merge into single GeoTIFF."""
    tiles = _find_tiles(lat, lon, buffer_km)

    if not tiles:
        print(f"ERROR: Tidak ada DEMNAS tile untuk ({lat}, {lon}) buffer {buffer_km}km")
        return

    print(f"=== DEMNAS Download ===")
    print(f"Koordinat: {lat}, {lon} | Buffer: {buffer_km} km")
    print(f"Tiles: {len(tiles)} | Est. size: ~{len(tiles)*20} MB\n")

    # Get JWT
    jwt = _get_jwt_token()
    if not jwt:
        print("ERROR: Gagal login — tidak bisa download")
        return

    # Download tiles
    downloaded = []
    for i, tile in enumerate(tiles):
        print(f"[{i+1}/{len(tiles)}]", end=" ")
        path = _download_tile(tile, jwt)
        if path:
            downloaded.append(path)
        else:
            # JWT might have expired, try re-login
            print("  Re-login...")
            if TOKEN_FILE.exists():
                TOKEN_FILE.unlink()
            jwt = _get_jwt_token()
            if jwt:
                path = _download_tile(tile, jwt)
                if path:
                    downloaded.append(path)

    if not downloaded:
        print("ERROR: Tidak ada tile yang berhasil didownload")
        return

    print(f"\nBerhasil download: {len(downloaded)}/{len(tiles)} tiles")

    # Merge if output path specified
    if output_path and len(downloaded) > 0:
        print(f"Merging {len(downloaded)} tiles...")
        result = _merge_tiles(downloaded, output_path)
        if result:
            size_mb = os.path.getsize(result) / (1024 * 1024)
            print(f"\nSUCCESS: DEMNAS merged DEM: {result} ({size_mb:.1f} MB)")
            print(f"Resolusi: 0.27 arcsec (~8.1m)")
            print(f"CRS: EPSG:4326 (WGS-84)")
            print(f"Sumber: BIG DEMNAS (data resmi)")

            if create_provenance:
                try:
                    create_provenance(output_path,
                        tool='demnas_download',
                        data_source='BIG DEMNAS (tanahair.indonesia.go.id)',
                        coordinates={'lat': lat, 'lon': lon, 'buffer_km': buffer_km},
                        tiles_downloaded=len(downloaded),
                        tile_ids=[os.path.basename(p) for p in downloaded],
                        resolution='0.27 arcsec (~8.1m)',
                        references=['Perpres 27/2014', 'UU 4/2011'],
                        crs='EPSG:4326')
                except:
                    pass
    else:
        print(f"\nTiles tersedia di: {TILES_DIR}")
        for p in downloaded:
            print(f"  {p}")

    return json.dumps({
        "downloaded": len(downloaded),
        "total": len(tiles),
        "files": downloaded,
        "output": output_path,
    })


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage:")
        print("  demnas_engine.py info lat lon buffer_km")
        print("  demnas_engine.py download lat lon buffer_km [output.tif]")
        sys.exit(1)

    mode = sys.argv[1]
    try:
        if mode == 'info':
            get_demnas_info(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]))
        elif mode == 'download':
            out = sys.argv[5] if len(sys.argv) > 5 else None
            download_demnas(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), out)
        else:
            print(f"ERROR: Mode '{mode}' tidak dikenal. Gunakan: info, download")
    except Exception as e:
        print(f"ERROR [E502]: {e}")
        import traceback
        traceback.print_exc()
