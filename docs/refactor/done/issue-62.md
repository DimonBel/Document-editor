# Issue #62 -- [e2e] Re-implement existing APIs behind gateway -- rooms, documents, latex

**Status:** Done (PR auto-merged).

## What was done

Write an end-to-end test (`tests/e2e/`) that boots `infra/docker-compose.yml`, then via the gateway exercises every API the legacy app exposed:

- `POST /api/rooms`, `GET /api/rooms/{id}`, `GET /api/rooms`
- `POST /api/documents`, `GET /api/documents/{id}`, `GET /api/documents`
- `POST /api/latex/compile`, `POST /api/latex/to-docx`
- `WS /ws/{room_id}` (whiteboard CRDT convergence)
- `WS /ws/doc/{doc_id}` (rich-text CRDT convergence)

**Acceptance**
- `pytest tests/e2e` (Python, using Playwright for browser and httpx for HTTP) passes against the new stack.
- `cargo test -p room-service -p doc-service -p latex-service --all-features` passes.
- Frontend smoke: opening the SPA in Playwright shows the same UI as before.

## Files changed in the initial scaffolding commit

See commit `449281a` on branch `refactor/backend-services` (or `master` after merge).
