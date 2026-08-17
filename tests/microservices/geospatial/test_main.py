from fastapi.testclient import TestClient
from src.microservices.geospatial.main import create_app
from unittest.mock import patch

def test_health_check():
    app = create_app()
    client = TestClient(app)
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json() == {"status": "ok", "service": "stac-geospatial-lake"}

def test_crop_endpoint():
    app = create_app()
    client = TestClient(app)
    
    test_url = "dummy.tif"
    bbox = "-122.5,37.5,-122.4,37.6"
    
    # Mock the crop_cog function so we don't hit the network
    with patch('src.microservices.geospatial.main.crop_cog') as mock_crop:
        # Mock crop_cog is an async function, we can mock it using AsyncMock
        import asyncio
        from unittest.mock import AsyncMock
        mock_crop.side_effect = AsyncMock(return_value=b'II*\x00fake_tiff_data')
        
        response = client.get(f"/api/v1/cog/crop?url={test_url}&bbox={bbox}")
        
        assert response.status_code == 200
        assert response.headers["content-type"] == "image/tiff"
        assert len(response.content) > 0
        assert response.content.startswith(b'II*\x00')
        
        # Verify the mock was called correctly
        mock_crop.assert_called_once_with(test_url, [-122.5, 37.5, -122.4, 37.6])
