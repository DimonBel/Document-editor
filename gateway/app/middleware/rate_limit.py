from fastapi import Request
from gateway.app.adapters import redis as r
RATE_LIMIT_BUCKET = {"/api/v1/room-service": (100, 60), "/api/v1/doc-service": (100, 60), "/api/v1/latex-service": (20, 60)}
async def rate_limit_middleware(request: Request, call_next):
    if not any(request.url.path.startswith(p) for p in RATE_LIMIT_BUCKET):
        return await call_next(request)
    cap, refill = next(v for k, v in RATE_LIMIT_BUCKET.items() if request.url.path.startswith(k))
    key = (request.headers.get("authorization") or request.client.host).split()[-1] if request.headers.get("authorization") else (request.client.host or "anon")
    decision = await r.try_acquire_rate_limit(key, cap, refill)
    if decision == "deny":
        from fastapi.responses import JSONResponse
        return JSONResponse(status_code=429, content={"type":"about:blank#429","title":"Rate limit exceeded","status":429,"detail":"too many requests"}, headers={"Retry-After": str(refill)}, media_type="application/problem+json")
    return await call_next(request)
