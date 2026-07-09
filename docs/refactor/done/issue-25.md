# Issue #25 -- [packages] Bootstrap `ed.persistence-postgres` -- sqlx + outbox table

**Milestone:** M2-gateway  
**Status:** Done (PR auto-merged).

## What was done

Create `packages/persistence-postgres/`:

- `PlatformDb { pool: PgPool }` with helpers `acquire`, `begin`, `transaction`.
- `OutboxStore`:
  - `OutboxMessage` row (id, occurred_at, topic, aggregate_type, aggregate_id, correlation_id, payload JSONB, status, attempt_count, last_error, next_attempt_at, sent_at, created_at) + SQLx migration in `migrations/<timestamp>_outbox.sql`.
  - Status enum: `Pending`, `Retrying`, `Sent`, `DeadLettered`.
  - `append(message) -> Result<()>` (caller commits txn).
  - `claim_pending(limit) -> Vec<OutboxMessage>` using `SELECT ... FOR UPDATE SKIP LOCKED`.
  - `mark_sent(id)`, `mark_failed(id, err, backoff)`, `mark_dead_lettered(id)`.
- `RowStamp` interceptor for sqlx (Added -> set CreatedAt; Modified -> set UpdatedAt; Deleted -> soft-delete UPDATE unless `ISoftDeleteContext::is_hard_delete_requested(entity)`).
- Idempotent bootstrap migration that creates `outbox_messages` table if absent.

**Acceptance**
- `cargo test -p ed_persistence_postgres --features test-pg` runs against `postgres:16-alpine` testcontainer.
- CRDT-friendly: rows can be soft-deleted and later hard-deleted via `HardDelete` flag.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-25.md` recording the work for issue #25.
