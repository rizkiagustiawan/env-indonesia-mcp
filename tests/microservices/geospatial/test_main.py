from fastapi.testclient import TestClient
from src.microservices.geospatial.main import create_app

def test_health_check():
    app = create_app()
    client = TestClient(app)
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json() == {"status": "ok", "service": "stac-geospatial-lake"}
