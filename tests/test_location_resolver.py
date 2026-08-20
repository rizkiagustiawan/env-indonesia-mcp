import unittest
import json

from src.tools.gis import location_resolver as lr


class TestResolveLocation(unittest.TestCase):
    def test_kota_bima_resolves_to_kota(self):
        r = lr.resolve_location("kota bima")
        self.assertEqual(r["type"], "Kota")
        self.assertEqual(r["name"], "Kota Bima")
        self.assertEqual(r["level"], 2)

    def test_kabupaten_bima_resolves_to_kabupaten(self):
        r = lr.resolve_location("kabupaten bima")
        self.assertEqual(r["type"], "Kabupaten")
        self.assertEqual(r["name"], "Bima")
        self.assertEqual(r["level"], 2)

    def test_bare_bima_is_ambiguous(self):
        with self.assertRaises(lr.LocationError) as ctx:
            lr.resolve_location("bima")
        msg = ctx.exception.message
        self.assertIn("kota", msg.lower())
        self.assertIn("kabupaten", msg.lower())

    def test_banjar_is_ambiguous_but_qualified_forms_resolve(self):
        with self.assertRaises(lr.LocationError):
            lr.resolve_location("banjar")
        self.assertEqual(lr.resolve_location("kota banjar")["type"], "Kota")
        self.assertEqual(lr.resolve_location("kabupaten banjar")["type"], "Kabupaten")

    def test_semarang_is_ambiguous_city_vs_regency(self):
        with self.assertRaises(lr.LocationError):
            lr.resolve_location("semarang")
        self.assertEqual(lr.resolve_location("kota semarang")["type"], "Kota")

    def test_representative_point_lies_inside_the_geometry(self):
        from shapely.geometry import shape
        r = lr.resolve_location("kota bandung")
        geom = shape(json.loads(r["geometry_geojson"]))
        pt = shape(json.loads(r["representative_point_geojson"]))
        self.assertTrue(geom.contains(pt))

    def test_bbox_is_ordered_ws_en(self):
        r = lr.resolve_location("kabupaten dompu")
        w, s, e, n = r["bbox"]
        self.assertLessEqual(w, e)
        self.assertLessEqual(s, n)
        self.assertGreater(e, w)

    def test_area_km2_is_positive_and_plausible(self):
        r = lr.resolve_location("kota semarang")
        # Kota Semarang luasnya ~373 km²; toleransi lebar untuk proyeksi lokal.
        self.assertGreater(r["area_km2"], 200)
        self.assertLess(r["area_km2"], 800)

    def test_buffer_suggestion_is_positive_and_capped(self):
        r = lr.resolve_location("kota yogyakarta")
        self.assertGreater(r["buffer_km_suggested"], 0)
        self.assertLessEqual(r["buffer_km_suggested"], 20)

    def test_every_kabupaten_kota_resolves_and_point_inside(self):
        from shapely.geometry import shape
        index = lr._load_index(2)
        types = set()
        for feat in index:
            name = feat["properties"]["name"]
            typ = feat["properties"]["type"]
            if typ not in ("Kabupaten", "Kota"):
                continue
            types.add(typ)
            qualified = (typ + " " + name).lower()
            r = lr.resolve_location(qualified)
            self.assertEqual(r["type"], typ)
            geom = shape(json.loads(r["geometry_geojson"]))
            pt = shape(json.loads(r["representative_point_geojson"]))
            self.assertTrue(geom.contains(pt), f"rep point outside {name}")
        self.assertEqual(types, {"Kabupaten", "Kota"})

    def test_province_resolves(self):
        r = lr.resolve_location("jawa barat")
        self.assertEqual(r["level"], 1)
        self.assertIn(r["type"], ("Provinsi", "Propinisi"))

    def test_province_alias_jakarta(self):
        r = lr.resolve_location("jakarta")
        self.assertEqual(r["level"], 1)

    def test_unknown_name_raises_with_candidates(self):
        with self.assertRaises(lr.LocationError) as ctx:
            lr.resolve_location("wakanda")
        self.assertIn("tidak ditemukan", ctx.exception.message.lower())

    def test_prefix_variants_are_accepted(self):
        self.assertEqual(lr.resolve_location("KAB. BIMA")["type"], "Kabupaten")
        self.assertEqual(lr.resolve_location("Kota  Bandung")["type"], "Kota")

    def test_resolve_to_point_returns_lon_lat_buffer(self):
        lon, lat, buffer_km, meta = lr.resolve_to_point("kota bima")
        self.assertEqual(meta["type"], "Kota")
        self.assertGreater(buffer_km, 0)
        self.assertGreater(lat, -11.5)  # inside Indonesia rough bound
        self.assertLess(lat, 6.5)
        self.assertGreater(lon, 95)
        self.assertLess(lon, 142)


if __name__ == "__main__":
    unittest.main()
