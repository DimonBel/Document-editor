# Issue #41 -- [gateway] Correlation / request-id middleware

**Milestone:** M5-latex  
**Status:** Done (PR auto-merged).

## What was done

Implement `app/middleware/correlation.py`:
- Read `X-Correlation-Id` if present, else generate UUID v4.
- Attach `request.state.correlation_id`.
- Inject into outbound requests to upstreams.
- Emit log record with `correlation_id` field via `structlog` (or stdlib logging with extra).
- Add response header `X-Correlation-Id`.

**Acceptance**
- `pytest` asserts header propagation end-to-end.
- Logs emit one JSON line per request with `correlation_id` populated.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-41.md` recording the work for issue #41.
