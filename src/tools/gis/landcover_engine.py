#!/usr/bin/env python3
"""Land Cover Classification & Change Detection Engine
Uses Google Dynamic World 10m for classification (SNI 7645:2014 mapping)
"""
import sys, json, math, os
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from provenance import create_provenance


def olofsson_area_ci(mapped_areas, confusion_matrix, class_names, z=1.96):
    """Olofsson et al. 2014 area-weighted accuracy and unbiased area estimates.
    Equations 4, 9, 10 from the paper.
    Ref: Remote Sensing of Environment, 148, 42-57.
    """
    n_classes = len(class_names)
    W = np.array(mapped_areas) / sum(mapped_areas)  # area proportions

    # Eq 4: Estimated area proportions
    p_hat = np.zeros((n_classes, n_classes))
    for i in range(n_classes):
        n_i = sum(confusion_matrix[i])
        if n_i > 0:
            for j in range(n_classes):
                p_hat[i][j] = W[i] * confusion_matrix[i][j] / n_i

    # Eq 9: Unbiased area estimate per class
    A_total = sum(mapped_areas)
    area_est = []
    for j in range(n_classes):
        p_j = sum(p_hat[i][j] for i in range(n_classes))
        area_est.append(A_total * p_j)

    # Eq 10: Standard error of area
    se_area = []
    for j in range(n_classes):
        var_j = 0
        for i in range(n_classes):
            n_i = sum(confusion_matrix[i])
            if n_i > 1:
                p_ij = confusion_matrix[i][j] / n_i
                var_j += (W[i] * (p_ij - p_hat[i][j]))**2 / (n_i - 1)
        se_area.append(A_total * math.sqrt(var_j))

    # CI + results dict
    results = {}
    for j, name in enumerate(class_names):
        ci_low = max(0, area_est[j] - z * se_area[j])
        ci_high = area_est[j] + z * se_area[j]
        results[name] = {
            'mapped_area': mapped_areas[j],
            'adjusted_area': area_est[j],
            'se': se_area[j],
            'ci_lower': ci_low,
            'ci_upper': ci_high
        }

    # Overall accuracy
    oa = sum(p_hat[i][i] for i in range(n_classes))

    return results, oa

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

    # Per-class change table
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

    # === Olofsson area-weighted accuracy (simulated confusion matrix) ===
    # Simulated based on Dynamic World tropical accuracy ~0.85 OA (Brown et al. 2022)
    # Diagonal = 85% correct, off-diagonal errors distributed uniformly
    present_classes = sorted(all_classes)
    n_cls = len(present_classes)
    if n_cls > 1:
        sim_samples_per_class = 100
        sim_cm = []
        for i in range(n_cls):
            row = []
            correct = int(sim_samples_per_class * 0.85)
            errors_total = sim_samples_per_class - correct
            for j in range(n_cls):
                if i == j:
                    row.append(correct)
                else:
                    row.append(max(1, errors_total // (n_cls - 1)))
            sim_cm.append(row)

        # Use Period 2 mapped areas (ha) for Olofsson
        mapped_ha = [d2.get(cls, 0) for cls in present_classes]
        cls_names = [DW_TO_SNI.get(cls, {'sni_name': f'Class_{cls}'})['sni_name'] for cls in present_classes]

        olof_results, olof_oa = olofsson_area_ci(mapped_ha, sim_cm, cls_names)

        print(f"\n══ Olofsson Area-Weighted Accuracy (Olofsson et al. 2014) ══")
        print(f"Overall Accuracy (area-weighted): {olof_oa*100:.1f}%")
        print(f"Catatan: Confusion matrix disimulasikan (OA=85%, Brown et al. 2022 untuk Dynamic World tropis)")
        print("CATATAN: Confusion matrix disimulasikan (OA=85%). Untuk akurasi sebenarnya, gunakan ground truth sampling. Ref: CEOS WGCV LPV Protocol v0.1 (Olofsson et al. 2025).")
        print(f"\n{'Kelas':<30} {'Mapped (ha)':<14} {'Adjusted (ha)':<16} {'SE (ha)':<12} {'95% CI (ha)':<20}")
        print("-" * 92)
        for name in cls_names:
            r = olof_results[name]
            print(f"{name:<30} {r['mapped_area']:<14.1f} {r['adjusted_area']:<16.1f} {r['se']:<12.1f} [{r['ci_lower']:.1f}, {r['ci_upper']:.1f}]")

    # === Full Transition Matrix (dari → ke) ===
    # Penting untuk AMDAL: menunjukkan konversi hutan→perkebunan, mangrove→tambak, dll.
    transition = lc1.multiply(100).add(lc2).rename('transition')
    transition_sample = transition.reduceRegion(
        reducer=ee.Reducer.frequencyHistogram(),
        geometry=roi, scale=10, maxPixels=1e9
    ).get('transition')
    transition_hist = ee.Dictionary(transition_sample).getInfo()

    # Pixel area at 10m resolution
    pixel_area_ha = 10 * 10 / 1e4  # 0.01 ha per pixel

    # Parse into from-to matrix
    matrix = {}
    for code_str, count in transition_hist.items():
        code = int(float(code_str))
        from_class = code // 100
        to_class = code % 100
        from_name = DW_TO_SNI.get(from_class, {'sni_name': f'Class_{from_class}'})['sni_name']
        to_name = DW_TO_SNI.get(to_class, {'sni_name': f'Class_{to_class}'})['sni_name']
        if from_name not in matrix:
            matrix[from_name] = {}
        matrix[from_name][to_name] = count * pixel_area_ha

    # Print transition matrix
    print(f"\nMatriks Transisi Penggunaan Lahan (Ha):")
    all_trans_classes = sorted(set(k for row in matrix.values() for k in row))
    transition_header = "Dari \\ Ke"
    print(f"{transition_header:<30}", end='')
    for c in all_trans_classes:
        print(f"{c[:12]:>14}", end='')
    print()
    print("-" * (30 + 14 * len(all_trans_classes)))
    for from_c in sorted(matrix.keys()):
        print(f"{from_c:<30}", end='')
        for to_c in all_trans_classes:
            val = matrix[from_c].get(to_c, 0)
            print(f"{val:>14.1f}", end='')
        print()

    # Provenance metadata
    create_provenance(output_path,
        tool='change_detection',
        gee_collection='GOOGLE/DYNAMICWORLD/V1',
        date_range=[date1_start, date2_end],
        coordinates={'lat': lat, 'lon': lon},
        parameters={'buffer_km': buffer_km,
                    'period1': [date1_start, date1_end],
                    'period2': [date2_start, date2_end]},
        algorithms=['Dynamic World mode composite', 'Transition matrix', 'Olofsson area-weighted accuracy'],
        references=['Brown et al. 2022 (Dynamic World)', 'Olofsson et al. 2014'],
        classification='SNI 7645:2014 mapping',
        crs='EPSG:4326',
        scale_m=10)


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
    matrix_header = "Actual\\Predicted"
    header = f"{matrix_header:<15}" + "".join(f"{c:<10}" for c in classes) + f"{'Total':<10}"
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


def make_roi(lat, lon, buffer_km):
    """Helper: create ROI from lat/lon/buffer."""
    import ee
    return ee.Geometry.Point([lon, lat]).buffer(buffer_km * 1000)


def ensemble_classification(lat, lon, buffer_km, training_geojson, start_date, end_date, output_path):
    """Ensemble classification: Random Forest + Gradient Boosted Trees + SVM.
    Majority voting for robust land cover classification.
    Ref: Belgiu & Dragut 2016, Breiman 2001, Cortes & Vapnik 1995
    """
    import ee, requests, os
    ee.Initialize()

    roi = make_roi(lat, lon, buffer_km)

    # S2 composite with Cloud Score+
    cs = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED') \
        .filterDate(start_date, end_date).filterBounds(roi)
    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate(start_date, end_date).filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 30)) \
        .linkCollection(cs, ['cs_cdf']) \
        .map(lambda img: img.updateMask(img.select('cs_cdf').gte(0.60))) \
        .median().clip(roi)

    # Feature stack: 13 features
    ndvi = s2.normalizedDifference(['B8', 'B4']).rename('NDVI')
    ndwi = s2.normalizedDifference(['B3', 'B8']).rename('NDWI')
    ndbi = s2.normalizedDifference(['B11', 'B8']).rename('NDBI')
    features = s2.select(['B2','B3','B4','B5','B6','B7','B8','B8A','B11','B12']) \
        .addBands([ndvi, ndwi, ndbi])
    feature_names = features.bandNames()

    # Load training data — file path or inline JSON string
    if isinstance(training_geojson, str):
        if training_geojson.startswith('{'):
            training_data = json.loads(training_geojson)
        else:
            with open(training_geojson, 'r') as f:
                training_data = json.load(f)
        training_fc = ee.FeatureCollection(training_data)
    else:
        training_fc = ee.FeatureCollection(training_geojson)

    training = features.sampleRegions(
        collection=training_fc, properties=['class'], scale=10
    )

    # Train 3 classifiers
    rf = ee.Classifier.smileRandomForest(numberOfTrees=100, seed=42) \
        .train(training, 'class', feature_names)
    gbt = ee.Classifier.smileGradientTreeBoost(numberOfTrees=100, seed=42) \
        .train(training, 'class', feature_names)
    svm = ee.Classifier.libsvm(kernelType='RBF', cost=10, gamma=0.1) \
        .train(training, 'class', feature_names)

    # Classify with each
    c_rf = features.classify(rf).rename('class_rf')
    c_gbt = features.classify(gbt).rename('class_gbt')
    c_svm = features.classify(svm).rename('class_svm')

    # Majority voting
    ensemble = ee.ImageCollection([c_rf, c_gbt, c_svm]).mode().rename('class_ensemble')

    # Agreement map (how many classifiers agree)
    agreement = c_rf.eq(ensemble).add(c_gbt.eq(ensemble)).add(c_svm.eq(ensemble)).rename('agreement')

    result = ensemble.addBands([c_rf, c_gbt, c_svm, agreement]).clip(roi)

    # Per-class area
    area_img = ee.Image.pixelArea().divide(10000)  # hectares
    classes = ensemble.reduceRegion(
        reducer=ee.Reducer.frequencyHistogram(),
        geometry=roi, scale=10, maxPixels=1e9
    ).get('class_ensemble')
    class_hist = ee.Dictionary(classes).getInfo()

    # Agreement stats
    agree_stats = agreement.reduceRegion(
        reducer=ee.Reducer.mean(),
        geometry=roi, scale=10, maxPixels=1e9
    ).getInfo()

    # Download GeoTIFF untuk Thematic Cartography
    temp_tif = output_path.replace('.png', '_temp.tif')
    try:
        url = result.getDownloadURL({'region': roi, 'scale': 10, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'})
        tif_data = requests.get(url, timeout=120).content
        with open(temp_tif, 'wb') as f: f.write(tif_data)
    except Exception as e:
        print(f"ERROR Download TIF: {e}")
        temp_tif = None

    # Discrete Legend Labels (SNI 7645:2014)
    discrete_labels = {
        '#006400': 'Hutan Primer',
        '#228B22': 'Hutan Sekunder',
        '#FFD700': 'Padang Rumput',
        '#FF8C00': 'Semak/Belukar',
        '#DC143C': 'Tanah Terbuka',
        '#4169E1': 'Tubuh Air',
        '#808080': 'Permukiman',
        '#8B4513': 'Pertanian',
    }

    # Build conclusion text
    top_classes = sorted(class_hist.items(), key=lambda x: -x[1])[:3] if class_hist else []
    kesimpulan_lines = []
    for cls, ha in top_classes:
        kesimpulan_lines.append(f"• {cls[:20]}: {ha:.0f} Ha")
    kesimpulan = "\n".join(kesimpulan_lines) if kesimpulan_lines else "Data tidak tersedia"

    meta_stats = {'Algoritma': 'Dynamic World', 'Skala': '10m', 'Total Area': f'{total_area:.0f} Ha'}

    # Generate Thematic Cartography
    import json, math
    d = buffer_km / 111.0
    dlon = d / math.cos(math.radians(lat))
    geojson_data = {
        "type": "FeatureCollection",
        "features": [{
            "type": "Feature", "properties": {},
            "geometry": {"type": "Polygon", "coordinates": [[[lon-dlon, lat-d], [lon+dlon, lat-d], [lon+dlon, lat+d], [lon-dlon, lat+d], [lon-dlon, lat-d]]]}
        }]
    }

    from cartography import generate_sni_map
    generate_sni_map(
        json.dumps(geojson_data), output_path,
        title="PETA KLASIFIKASI TUTUPAN LAHAN (SNI 7645)",
        realtime=True, author="Rizki Agustiawan x ZeroClaw AI",
        overlay_raster=temp_tif,
        analysis_type='discrete',
        cmap='tab10',
        discrete_labels=discrete_labels,
        analysis_stats=meta_stats,
        conclusion_text=kesimpulan
    )
    if temp_tif and os.path.exists(temp_tif): os.remove(temp_tif)

    # GeoTIFF
    geotiff_path = output_path.replace('.png', '.tif')
    try:
        url = result.getDownloadURL({
            'region': roi, 'scale': 10, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
        })
        tif_data = requests.get(url, timeout=120).content
        with open(geotiff_path, 'wb') as f:
            f.write(tif_data)
        if not json_output:
            print(f"GeoTIFF: {geotiff_path} ({len(tif_data)/1024:.1f} KB)")
    except Exception as e:
        if not json_output:
            print(f"GeoTIFF export gagal: {e}")

    if json_output:
        # For 'classify' we don't have validation data to run olofsson, so we emit warning
        print(format_scientific_result("landcover_area", class_hist, sensor="Dynamic World", resolution_m=10))
    else:
        print(f"SUCCESS: Ensemble classification. Output: {output_path}")
        print(f"Classifier: RF(100) + GBT(100) + SVM(RBF)")
        agree_mean = agree_stats.get('agreement', agree_stats.get('agreement_mean', 'N/A'))
        if isinstance(agree_mean, (int, float)):
            print(f"Agreement rata-rata: {agree_mean:.2f}/3.0")
        else:
            print(f"Agreement rata-rata: {agree_mean}")
        print(f"Distribusi kelas: {class_hist}")
        print(f"Majority voting: mode() dari 3 classifier")
        print(f"Band output: class_ensemble, class_rf, class_gbt, class_svm, agreement")


def ccdc_change_detection(lat, lon, buffer_km, start_date, end_date, output_path):
    """CCDC — Continuous Change Detection and Classification.
    Uses GEE native ee.Algorithms.TemporalSegmentation.Ccdc().
    Detects WHEN change happened per pixel using harmonic regression.
    Ref: Zhu & Woodcock 2014, RSE 144:152-171
    """
    import ee, requests
    ee.Initialize()

    point = ee.Geometry.Point([lon, lat])
    roi = point.buffer(buffer_km * 1000)

    # Build Sentinel-2 collection with Cloud Score+ masking
    cs = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED') \
        .filterDate(start_date, end_date).filterBounds(roi)

    s2 = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate(start_date, end_date) \
        .filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 50)) \
        .linkCollection(cs, ['cs_cdf']) \
        .map(lambda img: img.updateMask(img.select('cs_cdf').gte(0.60))) \
        .select(['B2', 'B3', 'B4', 'B8', 'B11', 'B12'],
                ['BLUE', 'GREEN', 'RED', 'NIR', 'SWIR1', 'SWIR2'])

    # Run CCDC — 'lambda' is Python reserved word, pass via **kwargs
    ccdc = ee.Algorithms.TemporalSegmentation.Ccdc(
        collection=s2,
        breakpointBands=['GREEN', 'RED', 'NIR', 'SWIR1', 'SWIR2'],
        tmaskBands=['GREEN', 'SWIR1'],
        minObservations=4,  # lowered for cloudy tropics (Biju 2025)
        chiSquareProbability=0.995,  # higher sensitivity for tropical change (Zhou et al. 2025)
        minNumOfYearsScaler=1.33,
        dateFormat=1,  # fractional years
        maxIterations=25000,
        **{'lambda': 20.0}
    )

    # Extract number of segments (= breaks + 1)
    # CCDC returns array image; get number of segments per pixel
    try:
        n_segments = ccdc.select('tBreak').arrayLength(0).add(1).rename('n_segments')

        # Get first break date
        first_break = ccdc.select('tBreak').arrayGet(0).rename('first_break_year')

        # Get last break date (most recent change)
        last_break = ccdc.select('tBreak').arrayReduce(ee.Reducer.last(), [0]) \
            .arrayGet([0]).rename('last_break_year')

        # Change probability
        change_prob = ccdc.select('changeProb').arrayReduce(ee.Reducer.max(), [0]) \
            .arrayGet([0]).rename('change_probability')

        # Combine into result image
        result = n_segments.addBands([first_break, last_break, change_prob]).clip(roi)

        # Stats
        stats = result.reduceRegion(
            reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True) \
                .combine(ee.Reducer.max(), '', True),
            geometry=roi, scale=30, maxPixels=1e9
        ).getInfo()

        # Visualize number of segments (1 = stable, 2+ = changed)
        vis_params = {
            'min': 1, 'max': 5,
            'palette': ['#1a9850', '#91cf60', '#fee08b', '#fc8d59', '#d73027'],
            'region': roi, 'dimensions': 800
        }
        thumb_url = n_segments.getThumbURL(vis_params)
        img_data = requests.get(thumb_url, timeout=60).content
        with open(output_path, 'wb') as f:
            f.write(img_data)

        # GeoTIFF
        geotiff_path = output_path.replace('.png', '.tif')
        try:
            download_url = result.getDownloadURL({
                'region': roi, 'scale': 30, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
            })
            tif_data = requests.get(download_url, timeout=120).content
            with open(geotiff_path, 'wb') as f:
                f.write(tif_data)
            print(f"GeoTIFF: {geotiff_path} ({len(tif_data)/1024:.1f} KB)")
        except Exception as e:
            print(f"GeoTIFF export failed: {e}")

        # Print results
        print(f"SUCCESS: CCDC Change Detection. Output: {output_path}")
        print(f"Periode: {start_date} - {end_date}")
        print(f"Jumlah segmen rata-rata: {stats.get('n_segments_mean', 'N/A')}")
        print(f"Break pertama rata-rata: {stats.get('first_break_year_mean', 'N/A')}")
        print(f"Break terakhir rata-rata: {stats.get('last_break_year_mean', 'N/A')}")
        print(f"Probabilitas perubahan maks: {stats.get('change_probability_max', 'N/A')}")
        print(f"Hijau=stabil, Kuning=1 break, Merah=banyak break")
        print(f"Metode: CCDC (Zhu & Woodcock 2014)")
        print(f"Parameter: minObs=4, chi2=0.995, lambda=20, harmonic=3rd order")
        print(f"DISCLAIMER: Adaptasi GEE dari metode asli. Bukan exact CCDC (Zhu & Woodcock 2014). Untuk riset, gunakan implementasi R/Python asli.")

    except Exception as e:
        error_msg = str(e)
        print(f"ERROR: CCDC computation failed: {error_msg}")
        if 'memory' in error_msg.lower() or 'deadline' in error_msg.lower() or 'computation' in error_msg.lower():
            print("SARAN: Kurangi buffer_km atau gunakan Export.toDrive untuk area besar.")
        raise


def bfast_monitor(lat, lon, buffer_km, history_start, history_end, monitor_start, monitor_end, output_path):
    """BFAST Monitor-like near-real-time change detection.
    Uses harmonic regression on history period, then detects anomalies in monitoring period.

    Approach:
    1. Fit harmonic model (trend + 2 harmonics) on stable history period
    2. Predict expected NDVI for monitoring period
    3. Detect pixels where observed NDVI consistently deviates from predicted

    Not exact BFAST (OLS-MOSUM), but captures the core concept using GEE.
    Ref: Verbesselt et al. 2012 (BFASTmonitor), adapted for GEE
    """
    import ee, requests
    ee.Initialize()

    roi = make_roi(lat, lon, buffer_km)

    # History period — build harmonic model
    cs_hist = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED') \
        .filterDate(history_start, history_end).filterBounds(roi)
    s2_hist = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate(history_start, history_end).filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 40)) \
        .linkCollection(cs_hist, ['cs_cdf']) \
        .map(lambda img: img.updateMask(img.select('cs_cdf').gte(0.60)))

    # Compute NDVI for history
    def add_ndvi_time(img):
        ndvi = img.normalizedDifference(['B8', 'B4']).rename('NDVI')
        date = img.date()
        t = date.difference(ee.Date(history_start), 'year')
        return ndvi.addBands([
            ee.Image.constant(t).float().rename('t'),
            ee.Image.constant(t.multiply(2 * 3.14159).cos()).float().rename('cos1'),
            ee.Image.constant(t.multiply(2 * 3.14159).sin()).float().rename('sin1'),
            ee.Image.constant(t.multiply(4 * 3.14159).cos()).float().rename('cos2'),
            ee.Image.constant(t.multiply(4 * 3.14159).sin()).float().rename('sin2'),
            ee.Image.constant(1).rename('constant')
        ]).set('system:time_start', img.get('system:time_start'))

    hist_collection = s2_hist.map(add_ndvi_time)

    # Fit harmonic regression: NDVI = a0 + a1*t + a2*cos(2πt) + a3*sin(2πt) + a4*cos(4πt) + a5*sin(4πt)
    independents = ['constant', 't', 'cos1', 'sin1', 'cos2', 'sin2']
    dependent = 'NDVI'

    trend = hist_collection.select(independents + [dependent]) \
        .reduce(ee.Reducer.linearRegression(numX=len(independents), numY=1))

    coefficients = trend.select('coefficients').arrayProject([0]).arrayFlatten([independents])
    residuals_img = trend.select('residuals').arrayGet([0]).rename('RMSE_history')

    # Monitoring period — compute observed NDVI
    cs_mon = ee.ImageCollection('GOOGLE/CLOUD_SCORE_PLUS/V1/S2_HARMONIZED') \
        .filterDate(monitor_start, monitor_end).filterBounds(roi)
    s2_mon = ee.ImageCollection('COPERNICUS/S2_SR_HARMONIZED') \
        .filterDate(monitor_start, monitor_end).filterBounds(roi) \
        .filter(ee.Filter.lt('CLOUDY_PIXEL_PERCENTAGE', 40)) \
        .linkCollection(cs_mon, ['cs_cdf']) \
        .map(lambda img: img.updateMask(img.select('cs_cdf').gte(0.60)))

    observed_ndvi = s2_mon.map(lambda img: img.normalizedDifference(['B8', 'B4']).rename('NDVI')).median()

    # Predict NDVI for monitoring period midpoint using fitted harmonic model
    t_mid = ee.Date(monitor_start).difference(ee.Date(history_start), 'year') \
        .add(ee.Date(monitor_end).difference(ee.Date(monitor_start), 'year').divide(2))

    predicted = coefficients.select('constant') \
        .add(coefficients.select('t').multiply(t_mid)) \
        .add(coefficients.select('cos1').multiply(t_mid.multiply(2*3.14159).cos())) \
        .add(coefficients.select('sin1').multiply(t_mid.multiply(2*3.14159).sin())) \
        .add(coefficients.select('cos2').multiply(t_mid.multiply(4*3.14159).cos())) \
        .add(coefficients.select('sin2').multiply(t_mid.multiply(4*3.14159).sin())) \
        .rename('predicted_NDVI')

    # Anomaly = observed - predicted
    anomaly = observed_ndvi.subtract(predicted).rename('anomaly')

    # Magnitude of change (normalized by history RMSE)
    rmse = residuals_img.sqrt()
    magnitude = anomaly.divide(rmse.max(0.01)).rename('magnitude')

    # Significant change: |magnitude| > 2 (2-sigma)
    sig_change = magnitude.abs().gt(2).rename('significant_change')
    degradation = magnitude.lt(-2).rename('degradation')  # negative = vegetation loss

    result = anomaly.addBands([magnitude, sig_change.toFloat(), degradation.toFloat(),
                                predicted, observed_ndvi.rename('observed_NDVI')]).clip(roi)

    # Stats
    stats = result.reduceRegion(
        reducer=ee.Reducer.mean().combine(ee.Reducer.min(), '', True)
            .combine(ee.Reducer.max(), '', True),
        geometry=roi, scale=30, maxPixels=1e9
    ).getInfo()

    deg_stats = degradation.reduceRegion(
        reducer=ee.Reducer.sum().combine(ee.Reducer.count(), '', True),
        geometry=roi, scale=30, maxPixels=1e9
    ).getInfo()

    # Visualization — magnitude map
    vis = {
        'min': -5, 'max': 5,
        'palette': ['#d73027', '#f46d43', '#fdae61', '#fee08b', '#ffffbf',
                    '#d9ef8b', '#a6d96a', '#66bd63', '#1a9850'],
        'region': roi, 'dimensions': 800
    }
    thumb = magnitude.getThumbURL(vis)
    img_data = requests.get(thumb, timeout=60).content
    with open(output_path, 'wb') as f:
        f.write(img_data)

    # GeoTIFF
    geotiff_path = output_path.replace('.png', '.tif')
    try:
        url = result.getDownloadURL({
            'region': roi, 'scale': 30, 'format': 'GEO_TIFF', 'crs': 'EPSG:4326'
        })
        tif_data = requests.get(url, timeout=120).content
        with open(geotiff_path, 'wb') as f:
            f.write(tif_data)
        print(f"GeoTIFF: {geotiff_path} ({len(tif_data)/1024:.1f} KB)")
    except Exception as e:
        print(f"GeoTIFF export gagal: {e}")

    deg_sum = deg_stats.get('degradation_sum', 0) or 0
    deg_count = deg_stats.get('degradation_count', 1) or 1

    print(f"SUCCESS: BFAST Monitor-like Change Detection. Output: {output_path}")
    print("DISCLAIMER: Adaptasi GEE dari metode asli. Bukan exact BFAST (Verbesselt 2012). Untuk riset, gunakan implementasi R/Python asli.")
    print(f"Periode histori: {history_start} - {history_end}")
    print(f"Periode monitoring: {monitor_start} - {monitor_end}")
    print(f"Anomali NDVI (observasi - prediksi):")
    print(f"  Rata-rata: {stats.get('anomaly_mean', 'N/A')}")
    print(f"  Min: {stats.get('anomaly_min', 'N/A')}")
    print(f"  Maks: {stats.get('anomaly_max', 'N/A')}")
    print(f"Magnitudo (anomali / RMSE): rata-rata={stats.get('magnitude_mean', 'N/A')}")
    print(f"Degradasi signifikan (|mag|>2): {deg_sum:.0f} piksel ({100*deg_sum/deg_count:.1f}%)")
    print(f"Merah=degradasi, Hijau=peningkatan, Kuning=stabil")
    print(f"Metode: Harmonic regression (2 harmonics) + residual anomaly detection")
    print(f"Ref: Verbesselt et al. 2012 (BFASTmonitor concept, GEE adaptation)")


import sys
import json
from datetime import datetime

def format_scientific_result(parameter, class_hist, adjusted_areas=None, sensor="Dynamic World", resolution_m=10):
    # Construct M2 compliant ScientificResult JSON
    # We will pick the dominant class area for the 'value' field, but store all in claims/assumptions
    if not class_hist:
        return json.dumps({"status": "insufficient_data"})
        
    dominant_class = max(class_hist.items(), key=lambda x: x[1])
    value = dominant_class[1]
    
    prov = {
        "source_kind": "api",
        "source_identifier": sensor.lower().replace(" ", "_"),
        "acquisition_timestamp": datetime.utcnow().isoformat() + "Z",
        "sensor": sensor,
        "resolution": resolution_m
    }
    
    claims = []
    claims.append({"claim_type": "distribution", "description": json.dumps(class_hist)})
    
    uncertainty = None
    if adjusted_areas and str(dominant_class[0]) in adjusted_areas:
        adj, ci = adjusted_areas[str(dominant_class[0])]
        value = adj # Use unbiased area
        uncertainty = {
            "uncertainty_type": "confidence_interval",
            "lower": max(0, adj - ci),
            "upper": adj + ci,
            "method": "olofsson_2014",
            "confidence_level": 0.95
        }
        claims.append({"claim_type": "accuracy", "description": "Area adjusted via Olofsson 2014 robust estimator."})
    else:
        claims.append({"claim_type": "warning", "description": "Raw pixel count used. Subject to allocation disagreement bias."})
        
    res = {
        "parameter": parameter,
        "value": value,
        "unit": "ha",
        "status": "valid",
        "provenance": prov,
        "claims": claims
    }
    if uncertainty:
        res["uncertainty"] = uncertainty
        
    return json.dumps(res)

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: python3 landcover_engine.py <mode> <args...> [--json-result]")
        sys.exit(1)
        
    # Check for json flag
    json_output = False
    if '--json-result' in sys.argv:
        json_output = True
        sys.argv.remove('--json-result')


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
        elif mode == 'ensemble':
            ensemble_classification(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                                    sys.argv[5], sys.argv[6], sys.argv[7], sys.argv[8])
        elif mode == 'ccdc':
            ccdc_change_detection(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                                  sys.argv[5], sys.argv[6], sys.argv[7])
        elif mode == 'bfast':
            bfast_monitor(float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]),
                          sys.argv[5], sys.argv[6], sys.argv[7], sys.argv[8], sys.argv[9])
        else:
            print(f"ERROR: Mode '{mode}' tidak dikenal")
    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc(file=sys.stderr)
