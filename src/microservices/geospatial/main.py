from fastapi import FastAPI, HTTPException, Response
from src.microservices.geospatial.cog_reader import crop_cog

def create_app() -> FastAPI:
    app = FastAPI(title="Sumbawa Digital Twin: STAC API")

    @app.get("/health")
    def health_check():
        return {"status": "ok", "service": "stac-geospatial-lake"}

    @app.get("/api/v1/cog/crop")
    async def crop_cog_endpoint(url: str, bbox: str):
        try:
            bbox_floats = [float(x) for x in bbox.split(",")]
            if len(bbox_floats) != 4:
                raise ValueError("bbox must contain exactly 4 floats: minx,miny,maxx,maxy")
        except ValueError as e:
            raise HTTPException(status_code=400, detail=str(e))
            
        try:
            image_bytes = await crop_cog(url, bbox_floats)
            return Response(content=image_bytes, media_type="image/tiff")
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Failed to process COG: {str(e)}")

    return app

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:create_app", host="0.0.0.0", port=8000, factory=True)
