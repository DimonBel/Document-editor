# Issue #44 -- [room-service] Bootstrap axum service + tracing

**Milestone:** M5-latex  
**Status:** Done (PR auto-merged).

## What was done

Create `backend/room-service/`:
- `Cargo.toml` depending on `ed_domain`, `ed_contracts`, `ed_errors`, `ed_observability`, `ed_auth`, `ed_cache`, `ed_persistence_postgres`, `ed_persistence_mongo`, `ed_messaging_rabbitmq`, `axum`, `tokio`, `tracing`.
- `src/main.rs` -- `tokio::main`, calls `ed_observability::init_tracing("room-service", ...)` then `app::run().await`.
- `src/app.rs` -- builds `axum::Router` with `/healthz`, `/api/rooms/{id}`, `/api/rooms`, plus the WS route placeholder.
- `src/config.rs` -- `Config::from_env()` (HOST, PORT, DATABASE_URL, MONGO_URL, REDIS_URL, RABBITMQ_URL).
- Register in root `Cargo.toml` workspace + root `backend/Cargo.toml` workspace.

**Acceptance**
- `cargo run -p room-service` boots, `/healthz` returns 200.
- `docker build --build-arg SERVICE=room-service -f infra/docker/Dockerfile.rust-service .` produces a runnable image.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-44.md` recording the work for issue #44.
