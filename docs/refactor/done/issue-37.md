# Issue #37 -- [gateway] Reverse-proxy `/api/v1/{svc}/*` to upstream services

**Milestone:** M4-doc  
**Status:** Done (PR auto-merged).

## What was done

Implement `app/proxy/upstream.py` and `app/routers/api.py`:
- `service_registry.py` -- Settings field `services: dict[str, UpstreamConfig(name, base_url, healthz_path)]`.
- `upstream.py` -- `httpx.AsyncClient` (long-lived, one per registry entry) with `event_hooks` to inject `X-Request-Id`, `X-Correlation-Id`, `Authorization: Internal <internal-token>`.
- `routers/api.py` -- `POST /api/v1/{svc}/{path:path}` etc. Catch-all forwards to the right upstream. Method-pass-through, body-pass-through (stream for large bodies), response-pass-through.
- 502 mapping for upstream connect errors, 504 for upstream timeouts (with retries via `httpx-retries`).

**Acceptance**
- `pytest tests/test_proxy.py` -- assert `Authorization`, `X-Correlation-Id`, body echoing.
- Manual: `curl http://127.0.0.1:8080/api/v1/room-service/api/rooms` returns the rooms list when the upstream is mocked.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-37.md` recording the work for issue #37.
