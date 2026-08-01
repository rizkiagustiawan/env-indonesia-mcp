#!/usr/bin/env python3
"""GEE Export Infrastructure — handles Export.image.toDrive() with task polling.
Enables province-scale analysis beyond getDownloadURL() 32MB limit.
"""
import sys, json, time, os

try:
    import ee
    ee.Initialize()
except:
    pass

def export_to_drive(image, description, region, scale=30, crs='EPSG:4326',
                    folder='env_indonesia_exports', max_wait=600):
    """Export GEE image to Google Drive with polling.
    
    Args:
        image: ee.Image to export
        description: Export task name
        region: ee.Geometry
        scale: Pixel size in meters
        crs: Coordinate reference system
        folder: Google Drive folder name
        max_wait: Maximum wait time in seconds (default 10 min)
    
    Returns:
        dict with status, task_id, file info
    """
    task = ee.batch.Export.image.toDrive(
        image=image,
        description=description,
        folder=folder,
        region=region,
        scale=scale,
        crs=crs,
        maxPixels=1e10,
        fileFormat='GeoTIFF',
        formatOptions={'cloudOptimized': True, 'noData': -9999}
    )
    task.start()

    task_id = task.id
    print(f"Export dimulai: {description} (task_id: {task_id})")
    print(f"Folder Google Drive: {folder}")
    print(f"Scale: {scale}m, CRS: {crs}")

    # Poll for completion
    start_time = time.time()
    while task.active():
        elapsed = time.time() - start_time
        status = task.status()
        state = status.get('state', 'UNKNOWN')
        print(f"  Status: {state} ({elapsed:.0f}s)")

        if elapsed > max_wait:
            print(f"TIMEOUT: Export melebihi {max_wait}s. Task tetap berjalan di GEE.")
            return {
                'status': 'RUNNING',
                'task_id': task_id,
                'description': description,
                'message': f'Export masih berjalan. Cek Google Drive folder "{folder}" secara manual.'
            }

        time.sleep(15)  # poll every 15 seconds

    # Final status
    final = task.status()
    state = final.get('state', 'UNKNOWN')

    if state == 'COMPLETED':
        print(f"SUCCESS: Export selesai! File tersedia di Google Drive/{folder}/{description}.tif")
        return {
            'status': 'COMPLETED',
            'task_id': task_id,
            'description': description,
            'folder': folder,
            'file': f'{description}.tif',
            'message': f'File tersedia di Google Drive/{folder}/{description}.tif'
        }
    else:
        error = final.get('error_message', 'Unknown error')
        print(f"ERROR: Export gagal — {error}")
        return {
            'status': 'FAILED',
            'task_id': task_id,
            'error': error
        }


def check_export_status(task_id):
    """Check status of a running GEE export task."""
    tasks = ee.batch.Task.list()
    for task in tasks:
        if task.id == task_id:
            status = task.status()
            return json.dumps(status, indent=2, default=str)
    return json.dumps({'error': f'Task {task_id} tidak ditemukan'})


def list_active_exports():
    """List all active GEE export tasks."""
    tasks = ee.batch.Task.list()
    active = [t.status() for t in tasks if t.active()]
    return json.dumps(active, indent=2, default=str)


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: gee_export.py [status|list] [task_id]")
        sys.exit(1)

    mode = sys.argv[1]
    if mode == 'status' and len(sys.argv) >= 3:
        print(check_export_status(sys.argv[2]))
    elif mode == 'list':
        print(list_active_exports())
    else:
        print(f"Mode tidak dikenal: {mode}")
