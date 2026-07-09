# Issue #21 -- [packages] Bootstrap `ed.errors` — unified AppError + ProblemDetails

**Milestone:** M2-gateway  
**Status:** Done (PR auto-merged).

## What was done

Create `packages/errors/`:

- `AppError` enum: `Domain(DomainError)`, `Infra(InfraError)`, `Broker(BrokerError)`, `Auth(AuthError)`, `Validation(ValidationError)`, `NotFound(NotFoundError)`, `Internal(InternalError)`.
- Each variant serializes to RFC-7807 ProblemDetails (`type`, `title`, `status`, `detail`, `instance`).
- `From<sqlx::Error>`, `From<mongodb::error::Error>`, `From<lapin::Error>`, `From<deadpool_redis::PoolError>` impls that map to `Infra`/`Broker`.
- `IntoResponse` impl for axum (feature flag `axum` so the crate stays optional-dep).

**Acceptance**
- `cargo test -p ed_errors` covers all variants and at least one sqlx/lapin into-mapping.
- `ProblemDetails` serialization round-trips through `serde_json`.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-21.md` recording the work for issue #21.
