#!/usr/bin/env python3
"""Land Cover Classification & Change Detection Engine
Uses Google Dynamic World 10m for classification (SNI 7645:2014 mapping)
"""
import sys, json
import numpy as np

# Dynamic World → SNI 7645:2014 mapping
DW_TO_SNI = {
    0: {'dw': 'water', 'sni_code': '5000', 'sni_name': 'Tubuh Air', 'ipcc': 'Wetland'},
    1: {'dw': 'trees', 'sni_code': '2001', 'sni_name': 'Hutan Lahan Kering Primer/Sekunder', 'ipcc': 'Forest Land'},
    2: {'dw': 'grass', 'sni_code': '3000', 'sni_name': 'Padang Rumput/Sabana', 'ipcc': 'Grassland'},
    3: {'dw': 'flooded_vegetation', 'sni_code': '2006', 'sni_name': 'Hutan Rawa', 'ipcc': 'Wetland'},
    4: {'dw': 'crops', 'sni_code': '20091', 'sni_name': 'Pertanian', 'ipcc': 'Cropland'},
    5: {'dw': 'shrub_and_scrub', 'sni_code': '2007', 'sni_name': 'Semak/Belukar', 'ipcc': 'Grassland'},
    6: {'dw': 'built', 'sni_code': '6000', 'sni_name': 'Permukiman/Lahan Terbangun', 'ipcc': 'Settlement'},
    7: {'dw': 'bare', 'sni_code': '7000', 'sni_name': 'Tanah Terbuka', 'ipcc': 'Other Land'},
    8: {'dw': 'snow_and_ice', 'sni_code': '5002', 'sni_name': 'Es/Salju', 'ipcc': 'Other Land'},
}


def classify(lat, lon, buffer_km, start_date, end_date, output_path):
    """Land cover classification using Dynamic World 10m via GEE"""
    import ee
    ee.Initialize()
    import requests

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    dw = ee.ImageCollection('GOOGLE/DYNAMICWORLD/V1') \
        .filterDate(start_date, end_date) \
        .filterBounds(roi)

    count = dw.size().getInfo()
    if count == 0:
        print(f"ERROR: Tidak ada data Dynamic World untuk lokasi ini pada periode {start_date} - {end_date}")
        return

    # Mode (most frequent class) composite
    mode_img = dw.select('label').mode().clip(roi)

    # Area per class
    pixel_area = ee.Image.pixelArea()
    class_areas = pixel_area.addBands(mode_img).reduceRegion(
        reducer=ee.Reducer.sum().group(groupField=1, groupName='class'),
        geometry=roi, scale=10, maxPixels=1e9
    ).getInfo()

    total_area = sum(g['sum'] for g in class_areas.get('groups', []))

    # Thumbnail
    palette = ['419bdf', '397d49', '88b053', '7a87c6', 'e49635',
               'dfc35a', 'c4281b', 'a59b8f', 'b39fe1']
    thumb_url = mode_img.getThumbURL({
        'region': roi, 'dimensions': 800,
        'min': 0, 'max': 8, 'palette': palette
    })
    img = requests.get(thumb_url, timeout=30).content
    with open(output_path, 'wb') as f:
        f.write(img)

    # GeoTIFF
    tif_path = output_path.replace('.png', '.tif')
    url = mode_img.getDownloadURL({
        'scale': 10, 'region': roi, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
    })
    r = requests.get(url, timeout=60)
    with open(tif_path, 'wb') as f:
        f.write(r.content)

    print(f"SUCCESS: Land cover classification completed. Output: {output_path}")
    print(f"GeoTIFF: {tif_path}")
    print(f"Source: Google Dynamic World 10m | Period: {start_date} to {end_date}")
    print(f"Scenes used: {count}")
    print(f"Total area: {total_area/1e6:.2f} km2 ({total_area/1e4:.1f} ha)")
    print(f"\nKlasifikasi Penutup Lahan (SNI 7645:2014 mapping):")
    print(f"{'Kode SNI':<10} {'Kelas':<35} {'IPCC':<15} {'Area (ha)':<12} {'%':<6}")
    print("-" * 80)
    for g in sorted(class_areas.get('groups', []), key=lambda x: x['sum'], reverse=True):
        cls = g['class']
        area_ha = g['sum'] / 1e4
        pct = (g['sum'] / total_area * 100) if total_area > 0 else 0
        info = DW_TO_SNI.get(cls, {'sni_code': '?', 'sni_name': 'Unknown', 'ipcc': '?'})
        if pct > 0.1:
            print(f"{info['sni_code']:<10} {info['sni_name']:<35} {info['ipcc']:<15} {area_ha:<12.1f} {pct:<6.1f}")
    print(f"\nCatatan: Dynamic World memiliki 9 kelas. SNI 7645:2014 memiliki 23+ kelas.")
    print(f"Mapping bersifat aproksimasi. Untuk klasifikasi detail, gunakan supervised classification.")


def change_detection(lat, lon, buffer_km, date1_start, date1_end, date2_start, date2_end, output_path):
    """Land use change detection between 2 periods"""
    import ee
    ee.Initialize()
    import requests

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    def get_composite(start, end):
        return ee.ImageCollection('GOOGLE/DYNAMICWORLD/V1') \
            .filterDate(start, end).filterBounds(roi) \
            .select('label').mode().clip(roi)

    lc1 = get_composite(date1_start, date1_end)
    lc2 = get_composite(date2_start, date2_end)

    # Change map: where classes differ
    change = lc1.neq(lc2).selfMask()

    # Area of change
    pixel_area = ee.Image.pixelArea()
    change_area = change.multiply(pixel_area).reduceRegion(
        reducer=ee.Reducer.sum(), geometry=roi, scale=10, maxPixels=1e9
    ).getInfo()

    total_area_img = pixel_area.reduceRegion(
        reducer=ee.Reducer.sum(), geometry=roi, scale=10, maxPixels=1e9
    ).getInfo()

    # Per-class areas for both periods
    def get_class_areas(img):
        return pixel_area.addBands(img).reduceRegion(
            reducer=ee.Reducer.sum().group(groupField=1, groupName='class'),
            geometry=roi, scale=10, maxPixels=1e9
        ).getInfo()

    areas1 = get_class_areas(lc1)
    areas2 = get_class_areas(lc2)

    # Thumbnail of change
    thumb_url = change.getThumbURL({
        'region': roi, 'dimensions': 800,
        'min': 0, 'max': 1, 'palette': ['000000', 'ff0000']
    })
    img = requests.get(thumb_url, timeout=30).content
    with open(output_path, 'wb') as f:
        f.write(img)

    total_m2 = list(total_area_img.values())[0] if total_area_img else 0
    changed_m2 = list(change_area.values())[0] if change_area else 0

    print(f"SUCCESS: Land use change detection completed. Output: {output_path}")
    print(f"Period 1: {date1_start} to {date1_end}")
    print(f"Period 2: {date2_start} to {date2_end}")
    print(f"Total area: {total_m2/1e6:.2f} km2")
    if total_m2 > 0:
        print(f"Changed area: {changed_m2/1e4:.1f} ha ({changed_m2/total_m2*100:.1f}%)")

    # Change matrix
    print(f"\nPerubahan per kelas (ha):")
    print(f"{'Kelas (SNI)':<35} {'Period 1 (ha)':<15} {'Period 2 (ha)':<15} {'Perubahan (ha)':<15}")
    print("-" * 80)

    d1 = {g['class']: g['sum'] / 1e4 for g in areas1.get('groups', [])}
    d2 = {g['class']: g['sum'] / 1e4 for g in areas2.get('groups', [])}
    all_classes = sorted(set(list(d1.keys()) + list(d2.keys())))
    for cls in all_classes:
        a1 = d1.get(cls, 0)
        a2 = d2.get(cls, 0)
        info = DW_TO_SNI.get(cls, {'sni_name': f'Class {cls}'})
        diff = a2 - a1
        sign = '+' if diff > 0 else ''
        if abs(diff) > 0.1:
            print(f"{info['sni_name']:<35} {a1:<15.1f} {a2:<15.1f} {sign}{diff:<15.1f}")


def accuracy_assessment(predicted_json, actual_json):
    """Compute confusion matrix, OA, kappa, producer/user accuracy"""
    predicted = json.loads(predicted_json)
    actual = json.loads(actual_json)

    if len(predicted) != len(actual):
        print("ERROR: Jumlah predicted dan actual harus sama")
        return

    classes = sorted(set(predicted + actual))
    n_classes = len(classes)
    class_to_idx = {c: i for i, c in enumerate(classes)}

    # Build confusion matrix
    cm = np.zeros((n_classes, n_classes), dtype=int)
    for p, a in zip(predicted, actual):
        cm[class_to_idx[a]][class_to_idx[p]] += 1

    n = len(predicted)
    oa = np.trace(cm) / n * 100

    # Kappa
    row_sums = cm.sum(axis=1)
    col_sums = cm.sum(axis=0)
    pe = sum(row_sums[i] * col_sums[i] for i in range(n_classes)) / (n * n)
    po = np.trace(cm) / n
    kappa = (po - pe) / (1 - pe) if pe < 1 else 0

    # Producer/User accuracy
    print(f"SUCCESS: Accuracy Assessment")
    print(f"Total samples: {n} | Classes: {n_classes}")
    print(f"\nConfusion Matrix:")
    header = f"{'Actual\\Predicted':<15}" + "".join(f"{c:<10}" for c in classes) + f"{'Total':<10}"
    print(header)
    print("-" * len(header))
    for i, c in enumerate(classes):
        row = f"{c:<15}" + "".join(f"{cm[i][j]:<10}" for j in range(n_classes)) + f"{row_sums[i]:<10}"
        print(row)
    print("-" * len(header))
    total_row = f"{'Total':<15}" + "".join(f"{col_sums[j]:<10}" for j in range(n_classes)) + f"{n:<10}"
    print(total_row)

    print(f"\nOverall Accuracy (OA): {oa:.1f}%")
    print(f"Kappa Coefficient: {kappa:.4f}")
    print(f"Status: {'MEMENUHI (OA >= 85%)' if oa >= 85 else 'TIDAK MEMENUHI (OA < 85%)'}")
    print(f"Ref: SNI 8202:2015 | Min OA: 85% | Min Kappa: 0.75")

    print(f"\nPer-Class Accuracy:")
    print(f"{'Kelas':<15} {'Producer (%)':<15} {'User (%)':<15} {'F1-Score':<10}")
    for i, c in enumerate(classes):
        pa = cm[i][i] / row_sums[i] * 100 if row_sums[i] > 0 else 0
        ua = cm[i][i] / col_sums[i] * 100 if col_sums[i] > 0 else 0
        f1 = 2 * pa * ua / (pa + ua) if (pa + ua) > 0 else 0
        print(f"{c:<15} {pa:<15.1f} {ua:<15.1f} {f1:<10.1f}")


def supervised_rf(lat, lon, buffer_km, training_geojson_str, start_date, end_date, n_trees, output_path):
    """Random Forest supervised classification via GEE smileRandomForest.
    Ref: Nur et al. 2025, Amiren et al. 2024
    """
    import ee
    ee.Initialize()
    import requests

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    # Parse training GeoJSON
    training_geojson = json.loads(training_geojson_str)
    features = [ee.Feature(ee.Geometry(f['geometry']), f['properties']) for f in training_geojson['features']]
    training_fc = ee.FeatureCollection(features)

    # Cloud Score+ masked S2 composite
    csPlus = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED')
    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate(start_date, end_date).filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30)) \
        .linkCollection(csPlus, ['cs_cdf']) \
        .map(lambda img: img.updateMask(img.select('cs_cdf').gte(0.60))) \
        .median().clip(roi)

    # Compute indices
    ndvi = s2.normalizedDifference(['B8', 'B4']).rename('NDVI')
    ndwi = s2.normalizedDifference(['B3', 'B8']).rename('NDWI')
    ndbi = s2.normalizedDifference(['B11', 'B8']).rename('NDBI')

    bands = ['B2','B3','B4','B5','B6','B7','B8','B8A','B11','B12']
    composite = s2.select(bands).addBands([ndvi, ndwi, ndbi])
    all_bands = bands + ['NDVI', 'NDWI', 'NDBI']

    # Sample training regions
    training = composite.sampleRegions(
        collection=training_fc, properties=['class'], scale=10, tileScale=4
    )

    # 70/30 split
    training = training.randomColumn(seed=42)
    train_set = training.filter(ee.Filter.lt('random', 0.7))
    test_set = training.filter(ee.Filter.gte('random', 0.7))

    # Train RF
    classifier = ee.Classifier.smileRandomForest(
        numberOfTrees=n_trees, seed=42
    ).train(features=train_set, classProperty='class', inputProperties=all_bands)

    # Classify
    classified = composite.classify(classifier).clip(roi)

    # Accuracy
    train_cm = classifier.confusionMatrix()
    train_oa = train_cm.accuracy().getInfo()
    train_kappa = train_cm.kappa().getInfo()

    validated = test_set.classify(classifier)
    test_cm = validated.errorMatrix('class', 'classification')
    test_oa = test_cm.accuracy().getInfo()
    test_kappa = test_cm.kappa().getInfo()
    test_matrix = test_cm.array().getInfo()

    # Feature importance
    explanation = classifier.explain().getInfo()
    importance = explanation.get('importance', {})
    oob = explanation.get('outOfBagErrorEstimate', None)

    # Get unique classes
    classes = sorted(set(f['properties']['class'] for f in training_geojson['features']))
    n_classes = len(classes)

    # Thumbnail
    palette = ['0000FF','00FF00','FF0000','FFFF00','FF00FF','00FFFF','808080','FFA500'][:n_classes]
    thumb = classified.getThumbURL({
        'region': roi, 'dimensions': 800,
        'min': min(classes), 'max': max(classes), 'palette': palette
    })
    img_data = requests.get(thumb, timeout=30).content
    with open(output_path, 'wb') as f:
        f.write(img_data)

    # GeoTIFF
    tif_path = output_path.replace('.png', '.tif')
    try:
        url = classified.toByte().getDownloadURL({'scale': 10, 'region': roi, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'})
        r = requests.get(url, timeout=120)
        with open(tif_path, 'wb') as f:
            f.write(r.content)
    except Exception:
        tif_path = "N/A (area terlalu besar, gunakan Export.image.toDrive)"

    # Output
    print(f"SUCCESS: Random Forest Classification completed. Output: {output_path}")
    print(f"GeoTIFF: {tif_path}")
    print(f"Trees: {n_trees} | Bands: {len(all_bands)} | Classes: {n_classes}")
    print(f"\nTraining Accuracy: OA={train_oa*100:.1f}% | Kappa={train_kappa:.4f}")
    print(f"Validation Accuracy: OA={test_oa*100:.1f}% | Kappa={test_kappa:.4f}")
    if oob is not None:
        print(f"OOB Error Estimate: {oob*100:.1f}%")
    print(f"\nConfusion Matrix (validation):")
    print(f"  {test_matrix}")
    print(f"\nFeature Importance (top 5):")
    sorted_imp = sorted(importance.items(), key=lambda x: x[1], reverse=True)[:5]
    for name, val in sorted_imp:
        print(f"  {name}: {val:.2f}")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("ERROR: Usage: landcover_engine.py <mode> [args...]")
        sys.exit(1)

    mode = sys.argv[1]
    try:
        if mode == 'classify':
            classify(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                     sys.argv[5], sys.argv[6], sys.argv[7])
        elif mode == 'change':
            change_detection(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                             sys.argv[5], sys.argv[6], sys.argv[7], sys.argv[8], sys.argv[9])
        elif mode == 'accuracy':
            accuracy_assessment(sys.argv[2], sys.argv[3])
        elif mode == 'supervised':
            # args: lat lon buffer_km training_geojson start_date end_date n_trees output_path
            supervised_rf(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                          sys.argv[5], sys.argv[6], sys.argv[7], int(sys.argv[8]), sys.argv[9])
        else:
            print(f"ERROR: Unknown mode '{mode}'")
    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc(file=sys.stderr)
