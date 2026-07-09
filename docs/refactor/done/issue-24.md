# Issue #24 -- [packages] Bootstrap `ed.cache` -- deadpool-redis + RateLimiter + Session

**Milestone:** M2-gateway  
**Status:** Done (PR auto-merged).

## What was done

Create `packages/cache/`:

- `Cache { pool: deadpool_redis::Pool }` with helpers `get<T: DeserializeOwned>`, `set_ex<T: Serialize>(key, value, ttl)`, `delete`, `delete_pattern`.
- `RateLimiter` -- token-bucket per key (subject): `try_acquire(key, capacity, refill_per_sec) -> bool`. Redis Lua script for atomicity.
- `Session<T>` -- typed session stored under cookie-id key with TTL.

**Acceptance**
- `cargo test -p ed_cache --features test-redis` runs against `redis:7-alpine` testcontainer.
- RateLimiter atomicity test: 100 concurrent acquires on `capacity=10` yield exactly 10 successes.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-24.md` recording the work for issue #24.
