from fastapi import APIRouter, Request, Response
from gateway.app.config import settings
from gateway.app.adapters import upstream
import httpx
router = APIRouter(prefix="/api/v1")
@router.api_route("/{svc}/{path:path}", methods=["GET","POST","PUT","PATCH","DELETE","HEAD","OPTIONS"])
async def proxy(svc: str, path: str, request: Request) -> Response:
    if svc not in settings.SERVICES:
        from gateway.app.errors import AppError
        raise AppError(404, f"unknown upstream service '{svc}'", type_suffix="unknown-service")
    base = settings.SERVICES[svc]["base_url"]
    body = await request.body()
    async with httpx.AsyncClient(timeout=httpx.Timeout(30.0)) as client:
        r = await client.request(
            method=request.method, url=f"{base}/{path}",
            content=body, headers={k: v for k, v in request.headers.items() if k.lower() not in ("host", "content-length")},
            params=request.query_params,
        )
    return Response(content=r.content, status_code=r.status_code, headers={k: v for k, v in r.headers.items() if k.lower() not in ("content-encoding", "transfer-encoding", "content-length")})
