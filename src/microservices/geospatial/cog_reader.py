import xarray as xr
import rioxarray
import io

async def crop_cog(s3_url: str, bbox: list[float]) -> bytes:
    """
    Reads a subset of a Cloud-Optimized GeoTIFF without downloading the whole file.
    bbox: [minx, miny, maxx, maxy] in EPSG:4326
    """
    minx, miny, maxx, maxy = bbox
    
    # Open the COG remotely using rioxarray which leverages rasterio's VSI curl
    data = rioxarray.open_rasterio(s3_url)
    
    # Ensure CRS is 4326 for the bbox slicing to work accurately
    if data.rio.crs != "EPSG:4326":
        data = data.rio.reproject("EPSG:4326")

    # Crop the data
    cropped = data.rio.clip_box(minx=minx, miny=miny, maxx=maxx, maxy=maxy)
    
    # Write to in-memory bytes
    buffer = io.BytesIO()
    cropped.rio.to_raster(buffer, driver="GTiff")
    return buffer.getvalue()
