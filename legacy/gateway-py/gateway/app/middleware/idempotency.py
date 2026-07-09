from fastapi import Request
from fastapi.responses import Response
from gateway.app.adapters import redis as r
import json
async def idempotency_middleware(request: Request, call_next):
    if request.method == "GET":
        return await call_next(request)
    key = request.headers.get("idempotency-key")
    if not key:
        return await call_next(request)
    user = request.headers.get("authorization", "anon")
    redis_key = f"idem:{user}:{request.url.path}:{key}"
    cached = await r.client().get(redis_key)
    if cached:
        c = json.loads(cached)
        return Response(content=c["body"], status_code=c["status"], headers=c.get("headers", {}))
    response = await call_next(request)
    body = b""
    async for chunk in response.body_iterator: body += chunk
    await r.client().set(redis_key, json.dumps({"body": body.decode("utf-8", "replace"), "status": response.status_code, "headers": dict(response.headers)}), ex=24*60*60)
    return Response(content=body, status_code=response.status_code, headers=dict(response.headers))
