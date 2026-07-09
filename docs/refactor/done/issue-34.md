# Issue #34 -- [infra] Update root `.gitignore` for new artifacts

**Milestone:** M4-doc  
**Status:** Done (PR auto-merged).

## What was done

Extend `.gitignore` to cover new build artefacts:
- `target/`, `Cargo.lock.bak`
- `.venv/`, `__pycache__/`, `*.pyc`, `*.egg-info`, `.pytest_cache`, `.mypy_cache`
- `dist/` (already partly covered)
- `.env`, `.env.*` (keep `.env.example`)
- `infra/data/{pg,mongo,redis,rabbit}/` (local volumes)
- `gateway/.cache/`, `gateway/.coverage`, `gateway/htmlcov/`

**Acceptance**
- `git status` clean when run after `docker compose up` exits and dev work happens.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-34.md` recording the work for issue #34.
