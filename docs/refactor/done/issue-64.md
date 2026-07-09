# Issue #64 -- [e2e] Cutover: `infra/docker-compose.yml` becomes default; archive legacy

**Status:** Done (PR auto-merged).

## What was done

- Move the legacy root `docker-compose.yml` to `legacy/docker-compose.yml.bak` (kept for one release).
- Promote `infra/docker-compose.yml` to root (or document a `make up` alias).
- Update `SETUP.md` and any badges/READMEs to reference the new architecture.
- Update `.github/workflows/*.yml` so CI builds the new workspaces (`cargo build --workspace`, `pytest gateway/tests`, `pytest tests/e2e`).

**Acceptance**
- `git diff master..refactor/backend-services` shows no behaviour removed -- only added.
- `docker compose up` brings the new stack online end-to-end without manual steps.
- PR `refactor/backend-services -> master` is opened.

## Files changed in the initial scaffolding commit

See commit `449281a` on branch `refactor/backend-services` (or `master` after merge).
