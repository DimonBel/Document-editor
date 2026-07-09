# Issue #58 -- [latex-service] Subscribe to `latex.compile-requested` (async API)

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

Implement an async API: client publishes `LatexCompileRequestedEvent`; service consumes, runs compilation, publishes `LatexCompileSucceededEvent` or `LatexCompileFailedEvent` with the correlation_id attached.

**Acceptance**
- Test: publish request; service picks it up; result event lands on `ed.events` within 30s.
- WebSocket-friendly: client subscribes to a per-request correlation-id prefix via SSE.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-58.md` recording the work for issue #58.
