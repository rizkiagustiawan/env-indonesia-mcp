"""Location resolver — target bebas untuk seluruh Indonesia.

Mengubah nama wilayah (kota/kabupaten/provinsi), kode, atau koordinat menjadi
geometri yang bisa dipakai pipeline banjir/DTBP. Sumber: GADM 4.1 lokal
(`resources/indonesia_kabupaten_gadm41.geojson` dan
`resources/indonesia_admin_gadm41.geojson`), jadi resolusi offline dan tidak
memanggil API eksternal.

Keputusan desain yang jujur:
- Nama telanjang yang menunjuk ke Kota DAN Kabupaten ditolak (ambigu), bukan
  ditebak. Contoh: "bima", "semarang", "banjar".
- Titik wakil memakai `representative_point()` Shapely (dijamin di dalam
  poligon), bukan centroid mentah yang bisa jatuh di laut/luar wilayah.
"""

import json
import math
import os
import re
import unicodedata
from functools import lru_cache

from shapely.geometry import MultiPolygon, Point, Polygon, shape

_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
_PROJECT_ROOT = os.path.abspath(os.path.join(_SCRIPT_DIR, "..", "..", ".."))
_KAB_JSON = os.path.join(_PROJECT_ROOT, "resources", "indonesia_kabupaten_gadm41.geojson")
_PROV_JSON = os.path.join(_PROJECT_ROOT, "resources", "indonesia_admin_gadm41.geojson")

# Tipe GADM level 1 yang dianggap provinsi.
_PROVINCE_TYPES = {"Provinsi", "Propinisi"}
# Tipe level 2 yang layak dijadikan target analisis (buang badan air).
_TARGET_TYPES = {"Kabupaten", "Kota"}

# Alias umum nama provinsi.
_PROVINCE_ALIASES = {
    "jakarta": "jakarta raya",
    "dki": "jakarta raya",
    "dki jakarta": "jakarta raya",
    "jabar": "jawa barat",
    "jateng": "jawa tengah",
    "jawa tengah": "jawa tengah",
    "jatim": "jawa timur",
    "diy": "yogyakarta",
    "daerah istimewa yogyakarta": "yogyakarta",
    "ntb": "nusa tenggara barat",
    "ntt": "nusa tenggara timur",
    "sumbar": "sumatera barat",
    "sumut": "sumatera utara",
    "sumsel": "sumatera selatan",
    "sulsel": "sulawesi selatan",
    "sulteng": "sulawesi tengah",
    "sulut": "sulawesi utara",
    "sultra": "sulawesi tenggara",
    "sulbar": "sulawesi barat",
    "kaltim": "kalimantan timur",
    "kalbar": "kalimantan barat",
    "kalsel": "kalimantan selatan",
    "kaltara": "kalimantan utara",
    "kalteng": "kalimantan tengah",
    "kepri": "kepulauan riau",
}


class LocationError(ValueError):
    """Kesalahan resolusi lokasi dengan pesan yang layak ditampilkan ke pengguna."""

    def __init__(self, message):
        super().__init__(message)
        self.message = message


def _normalize(name):
    """Normalisasi nama: huruf kecil, buang aksen, kolaps spasi, strip prefix tipe."""
    s = unicodedata.normalize("NFKD", name)
    s = "".join(c for c in s if not unicodedata.combining(c))
    s = s.lower().strip()
    s = re.sub(r"^(kota|kabupaten|kab\.|kab|provinsi|propinsi|prov\.)\s+", "", s)
    return re.sub(r"\s+", " ", s).strip()


def _parse_qualifier(name):
    """Ekstrak qualifier tipe eksplisit dari input, bila ada. Mengembalikan (tipe, sisa)."""
    low = name.lower().strip()
    for token, typ in [
        ("kota", "Kota"),
        ("kabupaten", "Kabupaten"),
        ("kab.", "Kabupaten"),
        ("kab", "Kabupaten"),
        ("provinsi", "Provinsi"),
        ("propinsi", "Propinisi"),
        ("prov.", "Provinsi"),
    ]:
        if low == token or low.startswith(token + " "):
            return typ, name[len(token):].strip()
    return None, name


@lru_cache(maxsize=1)
def _load_index(level):
    """Muat index GADM level 1 (provinsi) atau 2 (kabupaten/kota), di-cache."""
    path = _PROV_JSON if level == 1 else _KAB_JSON
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    return tuple(data["features"])


def _geometry_of(feature):
    return shape(feature["geometry"])


def _representative_point(geom):
    """Titik wakil yang dijamin di dalam geometri."""
    if geom.geom_type == "Polygon":
        return geom.representative_point()
    if geom.geom_type == "MultiPolygon":
        largest = max(geom.geoms, key=lambda p: p.area)
        return largest.representative_point()
    return geom.representative_point()


def _approx_area_km2(geom):
    """Perkiraan luas dalam km² memakai proyeksi equirectangular lokal."""
    minx, miny, maxx, maxy = geom.bounds
    lat_mid = (miny + maxy) / 2.0
    lon_scale = 111.320 * math.cos(math.radians(lat_mid))
    lat_scale = 110.574

    def project(x, y):
        return ((x - minx) * lon_scale, (y - miny) * lat_scale)

    try:
        from shapely.ops import transform
        projected = transform(project, geom)
        # `project` memetakan derajat -> kilometer, jadi area sudah dalam km².
        return projected.area
    except Exception:
        span_km = math.hypot((maxx - minx) * lon_scale, (maxy - miny) * lat_scale)
        return (span_km / 2.0) ** 2 * math.pi


def _feature_to_result(feat, level):
    props = feat["properties"]
    geom = _geometry_of(feat)
    rep = _representative_point(geom)
    minx, miny, maxx, maxy = geom.bounds
    area = _approx_area_km2(geom)

    # Saran buffer: setengah sisi bbox terpendek, clamp [1, 20] km. Buffer besar
    # untuk kabupaten tidak masuk akal untuk banjir titik, jadi dibatasi.
    lon_scale = 111.320 * math.cos(math.radians((miny + maxy) / 2.0))
    span_x_km = (maxx - minx) * lon_scale
    span_y_km = (maxy - miny) * 110.574
    buffer = max(1.0, min(20.0, min(span_x_km, span_y_km) / 2.0))

    return {
        "name": props["name"],
        "type": props.get("type"),
        "level": level,
        "gadm_id": props.get("gadm_id"),
        "province": props.get("province"),
        "representative_point": {"lat": rep.y, "lon": rep.x},
        "bbox": [minx, miny, maxx, maxy],
        "area_km2": round(area, 1),
        "buffer_km_suggested": round(buffer, 2),
        "geometry_geojson": json.dumps(feat["geometry"]),
        "representative_point_geojson": json.dumps(
            {"type": "Point", "coordinates": [rep.x, rep.y]}
        ),
    }


def _candidates_for(name, level, typ_filter=None):
    target = _normalize(name)
    out = []
    for feat in _load_index(level):
        typ = feat["properties"].get("type")
        if typ_filter and typ not in typ_filter:
            continue
        if _normalize(feat["properties"]["name"]) == target:
            out.append(feat)
    return out


def resolve_location(name):
    """Resolusi nama wilayah -> dict geometri.

    Melempar `LocationError` bila ambigu, tak ditemukan, atau bukan target valid.
    """
    raw = (name or "").strip()
    if not raw:
        raise LocationError("Nama lokasi kosong.")

    qualifier, rest = _parse_qualifier(raw)

    # Koordinat mentah "lat,lon" dilewatkan apa adanya sebagai target titik.
    m = re.fullmatch(r"\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*", raw)
    if m:
        lat, lon = float(m.group(1)), float(m.group(2))
        return {
            "name": f"{lat},{lon}",
            "type": "Point",
            "level": 0,
            "gadm_id": None,
            "province": None,
            "representative_point": {"lat": lat, "lon": lon},
            "bbox": [lon, lat, lon, lat],
            "area_km2": 0.0,
            "buffer_km_suggested": 10.0,
            "geometry_geojson": json.dumps(
                {"type": "Point", "coordinates": [lon, lat]}
            ),
            "representative_point_geojson": json.dumps(
                {"type": "Point", "coordinates": [lon, lat]}
            ),
        }

    # Tipe eksplisit dari qualifier, atau dari alias provinsi.
    if qualifier in ("Provinsi", "Propinisi"):
        cands = _candidates_for(rest, 1, _PROVINCE_TYPES)
        if not cands:
            raise LocationError(f"Provinsi '{rest}' tidak ditemukan.")
        return _feature_to_result(cands[0], 1)

    if qualifier == "Kota":
        cands = _candidates_for(rest, 2, {"Kota"})
        if not cands:
            raise LocationError(f"Kota '{rest}' tidak ditemukan.")
        return _feature_to_result(cands[0], 2)

    if qualifier == "Kabupaten":
        cands = _candidates_for(rest, 2, {"Kabupaten"})
        if not cands:
            raise LocationError(f"Kabupaten '{rest}' tidak ditemukan.")
        return _feature_to_result(cands[0], 2)

    # Tanpa qualifier: cari di level 2 dulu.
    cands = _candidates_for(rest, 2, _TARGET_TYPES)
    types = {c["properties"]["type"] for c in cands}
    if len(types) > 1:
        raise LocationError(
            f"Nama '{raw}' ambigu: ada Kota '{raw}' dan Kabupaten '{raw}'. "
            f"Sebutkan secara eksplisit: 'kota {raw}' atau 'kabupaten {raw}'."
        )
    if len(types) == 1:
        return _feature_to_result(cands[0], 2)

    # Cari level 1 (provinsi), termasuk alias.
    prov_name = _PROVINCE_ALIASES.get(_normalize(rest), _normalize(rest))
    cands = _candidates_for(prov_name, 1, _PROVINCE_TYPES)
    if cands:
        return _feature_to_result(cands[0], 1)

    raise LocationError(f"Lokasi '{raw}' tidak ditemukan di basis data wilayah (GADM).")


def resolve_to_point(name, buffer_km=None):
    """Resolusi nama -> (lon, lat, buffer_km, meta) untuk pipeline banjir."""
    r = resolve_location(name)
    lon = r["representative_point"]["lon"]
    lat = r["representative_point"]["lat"]
    buf = buffer_km if buffer_km is not None else r["buffer_km_suggested"]
    if buf <= 0:
        raise LocationError(f"Buffer harus positif, diberi {buffer_km}.")
    return lon, lat, float(buf), r


if __name__ == "__main__":
    import sys

    if len(sys.argv) >= 3 and sys.argv[1] == "--location":
        try:
            print(json.dumps(resolve_location(sys.argv[2]), ensure_ascii=False))
        except LocationError as e:
            print(json.dumps({"error": e.message}, ensure_ascii=False))
            sys.exit(1)
    else:
        print("Usage: location_resolver.py --location '<nama kota/kabupaten/provinsi>'")
        sys.exit(2)
