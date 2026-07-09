# Issue #39 -- [gateway] Rate limiting (Redis token-bucket)

**Milestone:** M5-latex  
**Status:** Done (PR auto-merged).

## What was done

Implement `app/middleware/rate_limit.py`:
- Configurable `routes: { prefix: (capacity, refill_per_sec) }`.
- Uses `ed.cache::RateLimiter` semantics (transcribe to Redis Lua script in Python).
- Per user-id (from JWT) when authenticated, per IP otherwise.
- Adds `X-RateLimit-Remaining`, `X-RateLimit-Reset` headers; 429 on exhaustion.

**Acceptance**
- `pytest tests/test_rate_limit.py` against `redis:7-alpine` testcontainer.
- Burst: 100 concurrent requests with capacity=10, 10 succeed and the rest 429.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-39.md` recording the work for issue #39.
