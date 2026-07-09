# Issue #52 -- [doc-service] Wire in existing `TextDocument` CRDT

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

Currently `backend/src/documents/crdt.rs` is defined but unused (`backend/src/documents/manager.rs:232` falls back to whole-string replace). Wire it up:
- Replace naive `update_content` overwrite with CRDT op-based `apply(op)`.
- Persist CRDT snapshot to Postgres periodically (every 100 ops or 5 sec).

**Acceptance**
- `cargo test -p doc-service crdt::property` -- property test 5000 random interleavings converges.
- WS `/ws/doc/{id}` exchanges ops; replicates correctly.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-52.md` recording the work for issue #52.
