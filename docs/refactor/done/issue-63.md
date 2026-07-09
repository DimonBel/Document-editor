# Issue #63 -- [e2e] Update `frontend/vite.config.js` to proxy through gateway

**Status:** Done (PR auto-merged).

## What was done

Change `frontend/vite.config.js` so `/api` and `/ws` proxy to `http://gateway:8080` (or `127.0.0.1:8080` in dev). Verify:
- `pnpm run dev` boots, frontend loads, `/api/rooms` returns the same data.
- WS endpoints still work (`WhiteboardPage` and `DocEditorPage` connect cleanly).

**Acceptance**
- Manual screenshot test: open the app, draw on the whiteboard, type in a document -- both replicate correctly.

## Files changed in the initial scaffolding commit

See commit `449281a` on branch `refactor/backend-services` (or `master` after merge).
