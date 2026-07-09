# Issue #30 -- [infra] `Dockerfile.gateway` (Python 3.12-slim + uvicorn)

**Milestone:** M3-room  
**Status:** Done (PR auto-merged).

## What was done

Author `infra/docker/Dockerfile.gateway`:
- Base: `python:3.12-slim`.
- Copy `gateway/` (pyproject.toml, uv.lock/poetry.lock, app/, tests/).
- Install with `pip install --no-cache-dir -e .` (or `uv sync --frozen`).
- Add a non-root `app` user, set `PYTHONUNBUFFERED=1`.
- HEALTHCHECK `python -c "import httpx; httpx.get('http://127.0.0.1:8080/health').raise_for_status()"`.
- EXPOSE 8080, CMD `["uvicorn", "gateway.app.main:app", "--host", "0.0.0.0", "--port", "8080"]`.

**Acceptance**
- `docker build -f infra/docker/Dockerfile.gateway ./gateway` succeeds.
- Image < 250 MB.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-30.md` recording the work for issue #30.
