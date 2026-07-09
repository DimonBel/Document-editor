# Issue #43 -- [gateway] Smoke tests (pytest + httpx.AsyncClient)

**Milestone:** M5-latex  
**Status:** Done (PR auto-merged).

## What was done

Create `gateway/tests/`:
- `conftest.py` -- fixtures for `app` (FastAPI TestClient), `redis_clean`, `rabbit_container`, `mongo_container`.
- `test_health.py` -- `/healthz` returns 200 with version info.
- `test_main.py` -- full app boot with all adapters mocked.
- `test_lifespan.py` -- lifespan starts/stops cleanly with real Rabbit/Mongo/Redis testcontainers.

**Acceptance**
- `pytest --cov=gateway --cov-fail-under=70` passes in CI.
- All tests are hermetic -- no network calls outside testcontainers.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-43.md` recording the work for issue #43.
