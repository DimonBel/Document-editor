from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from gateway.app.config import settings
import httpx
import asyncio
router = APIRouter()
@router.websocket("/ws/{svc}/{path:path}")
async def proxy_ws(ws: WebSocket, svc: str, path: str):
    if svc not in settings.SERVICES:
        await ws.close(code=1008, reason="unknown service"); return
    base = settings.SERVICES[svc]["base_url"].replace("http", "ws", 1)
    target = f"{base}/{path}"
    await ws.accept()
    try:
        async with httpx.AsyncClient(timeout=httpx.Timeout(30.0)) as client:
            upstream = await client.ws_connect(target, params=ws.query_params)
            async def client_to_upstream():
                try:
                    while True:
                        msg = await ws.receive()
                        if msg["type"] == "websocket.receive":
                            data = msg.get("text") or msg.get("bytes")
                            if data is None: continue
                            await upstream.send(data)
                        elif msg["type"] == "websocket.disconnect":
                            await upstream.close(); break
                except WebSocketDisconnect:
                    await upstream.close()
            async def upstream_to_client():
                try:
                    async for msg in upstream.iter_text():
                        await ws.send_text(msg)
                except Exception:
                    pass
            await asyncio.gather(client_to_upstream(), upstream_to_client())
    except Exception as e:
        try: await ws.close(code=1011, reason=str(e)[:100])
        except: pass
