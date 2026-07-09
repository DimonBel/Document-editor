# Issue #27 -- [packages] Bootstrap `ed.messaging-rabbitmq` -- lapin wrapper + OutboxRelayService

**Milestone:** M3-room  
**Status:** Done (PR auto-merged).

## What was done

Create `packages/messaging-rabbitmq/`:

- `lapin` connection wrapper with reconnect (lifetime service similar to KafkaBusLifetimeService in RPBI).
- `IEventBus` trait: `publish<T: Serialize>(topic: &str, envelope: EventMessage<T>) -> Result<()>`, `subscribe<T>(topic: &str, handler: Handler<T>) -> Result<()>`. Split publish/subscribe through the same connection.
- `TypeObjectResolver` -- header-driven (header `x-ed-type-name`) so a consumer can dispatch a single typed handler for `EventMessage<JsonValue>` and re-deserialize internally.
- `Topology` declarator that ingests `infra/docker/rabbit/definitions.json` and asserts the broker has those exchanges/queues/bindings declared.
- `OutboxRelayService`: `tokio::spawn`'d background loop that:
  1. every N ms calls `OutboxStore::claim_pending(BATCH)`.
  2. for each row, publishes via `IEventBus::publish` with `mandatory=true`, `confirm-select` ack awaited.
  3. on success: `mark_sent`; on failure: `mark_failed` (exp backoff) or `mark_dead_lettered` after MAX_ATTEMPTS.
- Middleware chain (compensating for KafkaFlow): `retry -> dead-letter-on-fail -> typed-handler`.

**Acceptance**
- `cargo test -p ed_messaging_rabbitmq --features test-rabbit` spins up `rabbitmq:3.13-management-alpine` and runs the full claim->publish->mark_sent lifecycle.
- TypeObjectResolver dispatches to the right handler per `x-ed-type-name` header.
- OutboxRelayService retries with backoff and dead-letters on MAX_ATTEMPTS.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-27.md` recording the work for issue #27.
