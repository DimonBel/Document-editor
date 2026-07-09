# Issue #45 -- [room-service] Migrate `RoomManager` onto Mongo + Postgres outbox

**Milestone:** M6-e2e  
**Status:** Done (PR auto-merged).

## What was done

Port `backend/src/rooms/manager.rs` logic to `backend/room-service/src/repo.rs`:
- `RoomRepo` over Mongo -- replaces the in-memory `Arc<Mutex<RoomManager>>` (`backend/src/main.rs:22`).
- Atomic mutations: on `create_room` / `update_room` / `delete_room`, also `OutboxStore::append(envelope)` for the corresponding `room.*` event in the same DB transaction (Postgres).
- `DomainError` returned by every operation; never `unwrap`.

**Acceptance**
- `cargo test -p room-service` -- repo CRUD + outbox append in same txn (rollback test).
- Original `backend/src/main.rs:22` `Arc<Mutex<RoomManager>>` is no longer referenced from the new service.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-45.md` recording the work for issue #45.
