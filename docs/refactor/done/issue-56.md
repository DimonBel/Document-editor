# Issue #56 -- [latex-service] Bootstrap axum service + tracing

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

Mirror M3#27 for `backend/latex-service/`. Different `service_name` = `latex-service`. Stronger tracing spans around `pdflatex` invocations.

**Acceptance**
- `cargo run -p latex-service` boots; `/healthz` 200.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-56.md` recording the work for issue #56.
