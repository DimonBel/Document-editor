# Issue #48 -- [room-service] Subscribe to `document.commit.recorded` for read-model updates

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

Add a `document_commit_recorded` consumer that updates the room's read-model when a document commits (if the room has a `latex_source` pointer, store the latest commit hash).

**Acceptance**
- Test: publish a `DocumentCommitRecordedEvent`, observe a corresponding update in Mongo (or read-model document).
- DLQ test: handler panics -> after MAX_ATTEMPTS the message lands on `ed.events.dlx`.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-48.md` recording the work for issue #48.
