# Issue #19 -- [packages] Bootstrap `ed.domain` — pure types (no broker/db)

**Milestone:** M2-gateway  
**Status:** Done (PR auto-merged).

## What was done

Create `packages/domain/`:

- `Entity<TId>`, `AuditableEntity<TId>` (CreatedAt/UpdatedAt/DeletedAt/newtypes), `ValueObject` (abstract, `GetEqualityComponents`).
- Aggregate markers: `IAggregateRoot`.
- Domain IDs newtypes: `UserId(Uuid)`, `RoomId(Uuid)`, `DocumentId(Uuid)`, all `Display + FromStr + Serialize` with package-private validation.
- Domain entities: `Room { id, name, created_by, created_at, version }`, `Document { id, title, content_ref, version }`. They hold invariants (non-empty name, etc.).
- `DomainError` (thiserror) — returned from `Result<T, DomainError>` everywhere.
- `ThrowHelper`-style macros: `ed_domain_assert!`, `ensure!`.

NO dependencies on `lapin`, `sqlx`, `mongodb`, `redis`, `axum`. Only `thiserror`, `uuid`, `serde`, `chrono`.

**Acceptance**
- `cargo test -p ed_domain` passes (smoke tests for invariants).
- `cargo clippy -p ed_domain --all-targets -- -D warnings` clean.
- Zero transitive broker/db deps (verified by `cargo tree`).

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-19.md` recording the work for issue #19.
