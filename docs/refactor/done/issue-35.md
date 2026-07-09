# Issue #35 -- [gateway] FastAPI bootstrap (pyproject, uvicorn, Settings)

**Milestone:** M4-doc  
**Status:** Done (PR auto-merged).

## What was done

Create `gateway/` with:
- `pyproject.toml` declaring FastAPI 0.115+, uvicorn, pydantic 2, httpx 0.27+, aio-pika, motor, redis, python-jose[cryptography], python-multipart.
- `app/main.py` building the FastAPI app, registering routers (auth, api, ws, realtime, health), wiring lifespan (init Rabbit connection, init Mongo, init Redis, init tracer).
- `app/config.py` with `pydantic-settings.BaseSettings` mirroring `infra/.env.example`.
- `app/errors.py` with `app_error_handler(request, exc: AppError)` returning RFC-7807 ProblemDetails.
- `Dockerfile.gateway` (per M1#13).

**Acceptance**
- `uvicorn gateway.app.main:app --reload` boots successfully.
- `curl http://127.0.0.1:8080/healthz` returns 200.
- `pytest` (smoke) passes.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-35.md` recording the work for issue #35.
