# Issue #59 -- [latex-service] Publish `latex.compile.{succeeded,failed}` events

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

For both the HTTP path and the async RabbitMQ path, on completion append an `EventMessage<LatexCompile*Event>` with the artefact storage URL and the correlation_id.

**Acceptance**
- Test: HTTP compile happy path -> `latex.compile.succeeded` lands on `ed.events` within 5s.
- Test: HTTP compile fail path (malformed LaTeX) -> `latex.compile.failed` carries the error message.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-59.md` recording the work for issue #59.
