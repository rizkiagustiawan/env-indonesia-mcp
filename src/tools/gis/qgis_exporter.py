#!/usr/bin/env python3
"""Export selected route polygons and a centroid LineString to GeoJSON."""

import json
import os
import sys

from osgeo import ogr


def export_route_to_geojson(shp_path, route, output_geojson):
    if os.path.exists(output_geojson):
        raise ValueError(f"refusing to overwrite existing output: {output_geojson}")
    route_nodes = [node.strip() for node in route.split("->") if node.strip()]
    if not route_nodes or len(route_nodes) != len(set(route_nodes)):
        raise ValueError("route must contain unique non-empty nodes separated by '->'")

    source = ogr.Open(shp_path, 0)
    if source is None:
        raise ValueError(f"could not open Shapefile: {shp_path}")
    layer = source.GetLayer()
    if layer.FindFieldIndex("NOMOR_PETA", 1) < 0:
        raise ValueError("Shapefile must contain NOMOR_PETA field")

    driver = ogr.GetDriverByName("GeoJSON")
    output = driver.CreateDataSource(output_geojson)
    if output is None:
        raise OSError(f"could not create GeoJSON: {output_geojson}")
    out_layer = output.CreateLayer("route_path", layer.GetSpatialRef(), ogr.wkbUnknown)
    for name, field_type in (("ID", ogr.OFTString), ("Step", ogr.OFTInteger), ("Type", ogr.OFTString)):
        out_layer.CreateField(ogr.FieldDefn(name, field_type))

    centroids = []
    for step, node_id in enumerate(route_nodes, start=1):
        layer.SetAttributeFilter("NOMOR_PETA = '{}'".format(node_id.replace("'", "''")))
        feature = layer.GetNextFeature()
        if feature is None:
            raise ValueError(f"route node not found in NOMOR_PETA: {node_id}")
        geometry = feature.GetGeometryRef()
        if geometry is None:
            raise ValueError(f"route node has no geometry: {node_id}")
        output_feature = ogr.Feature(out_layer.GetLayerDefn())
        output_feature.SetGeometry(geometry.Clone())
        output_feature.SetField("ID", node_id)
        output_feature.SetField("Step", step)
        output_feature.SetField("Type", "Polygon")
        out_layer.CreateFeature(output_feature)
        centroids.append(geometry.Centroid())
    layer.SetAttributeFilter(None)

    if len(centroids) > 1:
        line = ogr.Geometry(ogr.wkbLineString)
        for centroid in centroids:
            line.AddPoint(centroid.GetX(), centroid.GetY())
        line_feature = ogr.Feature(out_layer.GetLayerDefn())
        line_feature.SetGeometry(line)
        line_feature.SetField("ID", "Route_Line")
        line_feature.SetField("Step", 0)
        line_feature.SetField("Type", "LineString")
        out_layer.CreateFeature(line_feature)

    output = None
    source = None
    return {"status": "ok", "output_geojson": output_geojson, "route_nodes": len(route_nodes), "line_created": len(centroids) > 1, "limitations": ["Route is represented by source polygons and centroid connections; it is not a hydraulic path."]}


def main(argv):
    if len(argv) != 4:
        print(json.dumps({"status": "invalid_request", "error": "usage: qgis_exporter.py SHP ROUTE OUTPUT_GEOJSON"}))
        return 2
    try:
        print(json.dumps(export_route_to_geojson(*argv[1:])))
        return 0
    except (OSError, ValueError, RuntimeError) as error:
        print(json.dumps({"status": "error", "error": str(error)}))
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
