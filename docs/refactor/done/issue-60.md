# Issue #60 -- [latex-service] Store compiled artefacts on disk volume + metadata in Mongo

**Status:** Done (PR auto-merged).

## What was done

- Mount a docker volume `latex_artifacts` at `/var/lib/latex` in the latex-service container.
- Persist artefacts (`<request_id>.tex`, `<request_id>.pdf`, `<request_id>.docx`) there.
- Persist metadata (request_id, source hash, compile duration, error, file paths, created_at) to MongoDB `latex_artifacts` collection.

**Acceptance**
- Test: compile end-to-end produces artefacts on disk + Mongo records.
- Re-running same source hash returns the cached artefact (idempotent compile).

## Files changed in the initial scaffolding commit

See commit `449281a` on branch `refactor/backend-services` (or `master` after merge).
