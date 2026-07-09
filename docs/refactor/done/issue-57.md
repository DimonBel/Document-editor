# Issue #57 -- [latex-service] Move `backend/src/latex` into the service crate

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

Move:
- `backend/src/latex/parser.rs` -> `backend/latex-service/src/parser.rs`
- `backend/src/latex/omml.rs` -> `backend/latex-service/src/omml.rs`
- `backend/src/latex/docx_writer.rs` -> `backend/latex-service/src/docx_writer.rs`
- `backend/src/latex/http.rs` -> `backend/latex-service/src/http.rs` (re-shape to axum handlers).
- Keep `MAX_SOURCE_BYTES = 1 MiB`; enforce `-no-shell-escape` when shelling out to `pdflatex`.

**Acceptance**
- `cargo test -p latex-service --all-features` (existing parser unit tests ported).
- `POST /api/latex/compile` returns valid PDF for a sample `.tex`.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-57.md` recording the work for issue #57.
