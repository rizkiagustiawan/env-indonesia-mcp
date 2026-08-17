from fastapi import FastAPI

def create_app() -> FastAPI:
    app = FastAPI(title="Sumbawa Digital Twin: STAC API")

    @app.get("/health")
    def health_check():
        return {"status": "ok", "service": "stac-geospatial-lake"}

    return app

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:create_app", host="0.0.0.0", port=8000, factory=True)
