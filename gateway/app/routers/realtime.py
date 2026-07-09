from fastapi import APIRouter, Request
from fastapi.responses import StreamingResponse
from gateway.app.adapters import rabbit
import asyncio, json
router = APIRouter(prefix="/api/realtime")
@router.get("/sse")
async def sse(request: Request):
    q = await rabbit.register_sse_consumer()
    async def gen():
        try:
            yield f"event: hello\ndata: {{"status":"connected"}}\n\n"
            while True:
                if await request.is_disconnected(): break
                try:
                    evt = await asyncio.wait_for(q.get(), timeout=15)
                    yield f"event: room\ndata: {json.dumps(evt)}\n\n"
                except asyncio.TimeoutError:
                    yield ": keepalive\n\n"
        finally:
            await rabbit.unregister_sse_consumer(q)
    return StreamingResponse(gen(), media_type="text/event-stream")
