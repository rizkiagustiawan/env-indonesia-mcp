import pytest
import os
from unittest.mock import patch, MagicMock
from src.microservices.geospatial.cog_reader import crop_cog

@pytest.mark.asyncio
async def test_crop_cog_returns_bytes():
    # Use a mock instead of a slow real S3 download for CI/CD speed
    with patch("src.microservices.geospatial.cog_reader.rioxarray.open_rasterio") as mock_open:
        # Create a deep mock for xarray DataArray chaining
        mock_da = MagicMock()
        mock_da.rio.crs = "EPSG:4326"
        mock_cropped = MagicMock()
        mock_da.rio.clip_box.return_value = mock_cropped
        
        # When to_raster is called, write fake tiff bytes to the buffer
        def fake_to_raster(buffer, driver):
            buffer.write(b'II*\x00fake_tiff_data')
            
        mock_cropped.rio.to_raster.side_effect = fake_to_raster
        mock_open.return_value = mock_da
        
        test_url = "dummy.tif"
        bbox = [-122.5, 37.5, -122.4, 37.6]
        
        result = await crop_cog(test_url, bbox)
        assert isinstance(result, bytes)
        assert result.startswith(b'II*\x00')
