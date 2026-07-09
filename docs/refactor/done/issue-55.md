# Issue #55 -- [doc-service] Testcontainers-rs + property-based CRDT tests

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

Same pattern as M3#32, plus proptest-based CRDT convergence:
- `proptest!` generates random op sequences; two replicas independently apply them -> final states equal.
- Persisted snapshot recovery test: kill service, restart, snapshot is the same.

**Acceptance**
- `cargo test -p doc-service --all-features` passes; runtime < 5 min.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-55.md` recording the work for issue #55.
