from fastapi import APIRouter
from gateway.app.config import settings
router = APIRouter()
@router.get("/healthz")
async def healthz():
    return {"status": "ok", "service": settings.SERVICE_NAME, "version": "0.1.0"}
@router.get("/.well-known/jwks.json")
async def jwks():
    return {"keys": []}  # populated on first key use
