# Issue #46 -- [room-service] Publish `room.created`/`updated`/`deleted` via outbox

**Milestone:** M6-e2e  
**Status:** Done (PR auto-merged).

## What was done

- For every mutation that succeeded, an `EventMessage<RoomCreatedEvent>` etc. is appended to `outbox_messages` with `topic = topics::room::CREATED`, `aggregate_id = room.id`.
- `OutboxRelayService` (from `ed.messaging_rabbitmq`) is started at app boot, polls Postgres, publishes to RabbitMQ.
- `room.snapshot_requested` consumer (for `room.snapshot.{id}.{seq}`) and `room.user_joined`/`room.user_left` publishers are wired.

**Acceptance**
- Test: create a room -> within PollInterval a `room.created` message lands on `ed.events` with correct envelope.
- Removing Postgres access forces dead-letter within `MAX_ATTEMPTS`.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-46.md` recording the work for issue #46.
