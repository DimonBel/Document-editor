# Issue #51 -- [doc-service] Migrate `DocumentManager` to Postgres + outbox

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

Port `backend/src/documents/manager.rs` to Postgres. Replace the JSON file persistence (`backend/src/documents/manager.rs:94-157`) with `DocumentRepo` using sqlx (documents are durable text, Mongo is not appropriate).

**Acceptance**
- Atomic mutation test (same as room-service) passes.
- Soft-delete + audit row-stamping via `RowStamp` interceptor.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-51.md` recording the work for issue #51.
