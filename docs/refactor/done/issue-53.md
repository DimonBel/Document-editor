# Issue #53 -- [doc-service] Publish `document.created`/`updated`/`deleted` events

**Milestone:** misc  
**Status:** Done (PR auto-merged).

## What was done

On every document mutation append an `EventMessage<...Event>` to `outbox_messages`; `OutboxRelayService` publishes to RabbitMQ.

**Acceptance**
- Test: create a document, see `document.created` on `ed.events` within PollInterval.
- Delete a document, see `document.deleted` exactly once (Idempotency-Key against duplicate relay).

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-53.md` recording the work for issue #53.
