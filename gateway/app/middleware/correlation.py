from fastapi import Request
import uuid
CORRELATION_HEADER = "x-correlation-id"
async def correlation_middleware(request: Request, call_next):
    cid = request.headers.get(CORRELATION_HEADER) or str(uuid.uuid4())
    request.state.correlation_id = cid
    response = await call_next(request)
    response.headers[CORRELATION_HEADER] = cid
    return response
