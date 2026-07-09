# Issue #42 -- [gateway] Subscribe `room.*` from RabbitMQ -> SSE fanout

**Milestone:** M5-latex  
**Status:** Done (PR auto-merged).

## What was done

Implement `app/routers/realtime.py` and `app/adapters/rabbit.py`:
- `adapters/rabbit.py` -- `aio_pika.connect_robust(url)`, expose `consume_topic(topic, handler)`.
- On startup: subscribe to `room.*`, deserialize `EventMessage<JsonValue>`, republish via per-user-keyed Redis pub-sub channels.
- `GET /api/realtime/sse?topics=room.*&since=<last_event_id>` -- opens an SSE stream; merges events for the requesting user's rooms.

**Acceptance**
- Publishing to RabbitMQ while a client is connected surfaces events within 200ms.
- `pytest` with `aio-pika` testcontainer: publish three messages, SSE yields three events.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-42.md` recording the work for issue #42.
