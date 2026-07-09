# Issue #40 -- [gateway] Idempotency-Key middleware (Redis-backed)

**Milestone:** M5-latex  
**Status:** Done (PR auto-merged).

## What was done

Implement `app/middleware/idempotency.py`:
- Read header `Idempotency-Key` on non-GET requests.
- Hash `(user_id, route, idempotency_key)`; look up in Redis.
- If hit: replay stored response (status, body, headers) without calling upstream.
- If miss: forward request, persist response under TTL 24h on success.

**Acceptance**
- `pytest tests/test_idempotency.py` -- same key replayed twice returns identical response; in-flight race resolved via Redis `SET NX`.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-40.md` recording the work for issue #40.
