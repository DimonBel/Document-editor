# Issue #49 -- [room-service] Testcontainers-rs tests (Postgres, Mongo, RabbitMQ)

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

- `tests/api_test.rs` -- full axum tower (`tower::ServiceExt::oneshot`) covering `POST /api/rooms`, `GET /api/rooms/{id}`, `GET /api/rooms`.
- `tests/messaging_test.rs` -- `testcontainers-modules` boots Postgres, Mongo, RabbitMQ; verifies outbox -> publish -> consumer round-trip.
- `tests/ws_test.rs` -- `tokio-tungstenite` connects to `/ws/room/{id}`, exchanges 100 ops with the server, checks convergence.

**Acceptance**
- `cargo test -p room-service --all-features` passes in CI.
- Total runtime < 5 min.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-49.md` recording the work for issue #49.
