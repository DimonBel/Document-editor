# Issue #47 -- [room-service] Move whiteboard WS handler + CRDT to the service

**Milestone:** M6-e2e  
**Status:** Done (PR auto-merged).

## What was done

Port `backend/src/handlers/websocket.rs` and `backend/src/crdt/state.rs` to `backend/room-service/src/ws.rs` + `crdt.rs`. Keep the YATA-inspired Lamport-anchored linear log; refactor `O(n*m)` ordering to `BTreeMap<Uuid, Operation>` with `Vec<Uuid>` for parent pointers (or swap in `yrs`/`automerge` and document the tradeoff).

**Acceptance**
- `cargo test -p room-service crdt::property` -- proptest 1000 random interleavings converge to identical state across replicas.
- WS handler accepts `GET /ws/room/{id}` with proper close codes; no `unwrap`.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-47.md` recording the work for issue #47.
