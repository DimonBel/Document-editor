# Issue #50 -- [doc-service] Bootstrap axum service + tracing

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

Mirror M3#27 for `backend/doc-service/` (different `service_name` = `doc-service`).

**Acceptance**
- `cargo run -p doc-service` boots; `/healthz` 200.
- Docker build works.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-50.md` recording the work for issue #50.
