# Issue #31 -- [infra] RabbitMQ topology (`definitions.json` + `rabbitmq.conf`)

**Milestone:** M3-room  
**Status:** Done (PR auto-merged).

## What was done

Author `infra/docker/rabbit/definitions.json`:
- vhost `ed` (default).
- Exchanges: `ed.events` (topic, durable), `ed.events.dlx` (topic, durable).
- Queues (all durable, `x-dead-letter-exchange = ed.events.dlx`):
  - `ed.room-service`, `ed.doc-service`, `ed.latex-service`, `ed.audit`, `ed.realtime-gateway` (auto-delete).
- Bindings:
  - `ed.events -> ed.room-service` on `room.*`
  - `ed.events -> ed.doc-service` on `document.*`
  - `ed.events -> ed.latex-service` on `latex.*`
  - `ed.events -> ed.audit` on `#.audit.recorded`
  - `ed.events -> ed.realtime-gateway` on `*` (catch-all, SSE fanout)
- `rabbitmq.conf` enables `management`, `management.tcp.port=15672`, `default_user_tags=[management]`, `loopback_users.guest=false` (for local dev keep `loopback_users.guest=true` and document it).

**Acceptance**
- Topology matches `ed.messaging-rabbitmq::Topology` declarator (M0#10).
- `rabbitmqctl list_queues name messages consumers` after bring-up shows all queues with consumers = 0 (services not yet built) -> consumers >= 1 after services come up.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-31.md` recording the work for issue #31.
