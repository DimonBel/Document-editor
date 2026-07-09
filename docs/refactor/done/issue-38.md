# Issue #38 -- [gateway] WebSocket proxy through the gateway

**Milestone:** M4-doc  
**Status:** Done (PR auto-merged).

## What was done

Implement `app/routers/ws.py` and `app/proxy/ws_forwarding.py`:
- Accept `WebSocket` on `/ws/{svc}/{path:path}`.
- Open upstream `httpx_ws.AsyncWebSocket` or `websockets.connect` to `base_url/{path}`.
- Bidirectional byte forwarding (`await upstream.send_text(msg)` etc.) with ping/pong every 20s.
- Auth handshake: read `Authorization` from subprotocol/headers, verify before accepting.
- Lifecycle: cancel both legs on disconnect; close codes propagated.

**Acceptance**
- `pytest tests/test_ws_proxy.py` uses `httpx_ws` client + `websockets` server; full echo works.
- Backpressure: a slow client does not stall the upstream.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-38.md` recording the work for issue #38.
