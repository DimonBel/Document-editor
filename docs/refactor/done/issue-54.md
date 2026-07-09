# Issue #54 -- [doc-service] Subscribe to `latex.compile.succeeded` for cross-links

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

If a document references a LaTeX project, subscribe to `latex.compile.succeeded` and update the document's preview URL field.

**Acceptance**
- Test: publish `LatexCompileSucceededEvent` carrying a document-id; document preview URL updates.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-54.md` recording the work for issue #54.
