# Document-editor -- Architecture

> This document describes the service-oriented refactor of `Document-editor`
> (originally a single Rust binary + a React frontend). The pattern borrows
> from RPBI: a thin layer of **reusable packages** (the "DULL" services)
> composed by **thin backend services** that talk to each other over a
> message broker, fronted by a **Python gateway** that the SPA calls.

## Goals

1. **Reusability** -- cross-service concerns (auth, persistence, message
   contracts, observability) live in stable Rust crates that any service can
   compose. New services add zero new broker/db glue.
2. **Polyglot** -- the gateway is Python (best-in-class for HTTP
   reverse-proxy + auth + WS), the heavy domain logic stays in Rust
   (CRDTs, LaTeX).
3. **Polyglot persistence** -- choose the right tool per use case:
   Postgres for transactional + outbox, MongoDB for free-form snapshots,
   Redis for cache and rate-limit.
4. **At-least-once events with idempotent consumers** -- the outbox
   pattern + transactional commit guarantees no event is lost when a
   business write succeeds.
5. **Local dev = `docker compose up`** -- the whole stack is reproducible
   on a laptop with one command.
6. **Master stays green** -- the legacy app continues to work; refactor
   is delivered incrementally on a side branch.

## Layered architecture

```
              ┌───────────────────────────────────────────┐
              │             frontend (SPA)                │
              │   React + Vite + Ant Design (unchanged)  │
              └─────────────────────┬─────────────────────┘
                                    │ /api, /ws (HTTPS in prod)
                                    ▼
              ┌───────────────────────────────────────────┐
              │              gateway (Python)            │
              │ ┌─────────┐ ┌────────┐ ┌───────────────┐   │
              │ │ routers │ │adaptors│ │  middleware   │   │
              │ │  /auth  │ │ rabbit │ │   auth        │   │
              │ │  /api/* │ │ mongo  │ │   rate-limit  │   │
              │ │  /ws/*  │ │ redis  │ │   idempotency │   │
              │ │  /sse   │ │        │ │   correlation │   │
              │ └─────────┘ └────────┘ └───────────────┘   │
              └──────────┬─────────────┬──────────────┬────┘
                         │             │              │
                         ▼             ▼              ▼
            ┌──────────────────┐ ┌─────────────┐ ┌──────────────────┐
            │  room-service    │ │ doc-service │ │  latex-service   │
            │  (Rust · axum)   │ │ (Rust·axum) │ │  (Rust · axum)   │
            │  WS /ws/room/{id}│ │WS /ws/doc/{i}│ │ HTTP /latex/...  │
            │  Mongo + PG      │ │ Postgres    │ │  pdflatex + DOCX │
            └────────┬─────────┘ └──────┬──────┘ └────────┬─────────┘
                     │                  │                 │
                     └──────────────────┬┴────────────────┘
                                        │
                                        ▼
                            ┌──────────────────────────┐
                            │      ed.events (topic)   │
                            │      ed.events.dlx (DLX) │
                            └──────────────────────────┘
                                        │
        ┌──────────────────┬─────────────┼─────────────┬─────────────────┐
        ▼                  ▼             ▼             ▼                 ▼
   ed.room-service    ed.doc-service ed.latex-service  ed.audit     ed.realtime-gateway
   (queue)            (queue)         (queue)         (queue)        (queue)
```

## Crate dependency graph (no cycles)

```
errors  domain  contracts  observability
   │      │         │            │
   └──────┴────┬────┴────────────┘
              ▼
   auth  cache  persistence-mongo
              │
              ▼
       persistence-postgres
              │
              ▼
       messaging-rabbitmq
              │
              ▼
      backend/{room,doc,latex}-service
```

The arrows are `Cargo.toml` `[dependencies]` edges; reversing any of them
breaks the workspace.

## Reusable packages (`packages/`)

### `ed-domain`

Pure types. **No broker/db/auth imports.** The only allowed crates are
`thiserror`, `serde`, `uuid`, `chrono`.

Public surface:
- `Entity<TId>`, `AuditableEntity<TId>`, `IRowStamped`, `IAggregateRoot`
- `ValueObject` (abstract trait)
- `RoomId`, `DocumentId`, `UserId`, `ClientId` (UUID newtypes)
- `Room`, `Document` (aggregates; their methods enforce invariants)
- `DomainError` (Validation, NotFound, Conflict, Unauthorized, Forbidden,
  Invariant)

### `ed-contracts`

Wire types. **Zero runtime deps beyond `serde` / `uuid` / `chrono`.** This
is the only crate shared between Rust services and the Python gateway
(via JSON Schema).

Public surface:
- `EventMessage<T>` envelope (`id`, `occurred_at`, `service_name`,
  `module_id`, `event_name`, `topic`, `correlation_id`, `schema_version`,
  `data: Option<T>`) -- serializes to snake_case fields like
  `occurredAt`, `correlationId`.
- `Topics` partial modules per bounded context (`room`, `document`,
  `latex`, `audit`).
- Event payload records under `events::{room,document,latex,audit}`.
- JSON Schema at `packages/contracts/schema/event_message.schema.json` --
  the Python gateway generates Pydantic models from this.

### `ed-errors`

`AppError` enum + RFC-7807 `ProblemDetails`. `From` impls for `sqlx::Error`,
`mongodb::error::Error`, `lapin::Error`, `serde_json::Error`.

Status code mapping:
| Variant                       | Status |
|-------------------------------|--------|
| `Domain::NotFound`            | 404    |
| `Domain::Validation`          | 422    |
| `Domain::Conflict`            | 409    |
| `Domain::Unauthorized`        | 401    |
| `Domain::Forbidden`           | 403    |
| `Domain::Invariant`           | 400    |
| `AppError::NotFound`          | 404    |
| `AppError::Validation`        | 422    |
| `AppError::Auth`              | 401    |
| `AppError::Infra` / `Broker`  | 502    |
| `AppError::Internal`          | 500    |

### `ed-observability`

`init_tracing(service_name, json)` and `correlation::*` helpers. The
`Once` guard makes init idempotent -- safe to call from every service
entry-point and from tests.

### `ed-auth`

`JwtVerifier` (HS256 or RS256, JWKS), `CurrentUser` axum extractor
(reads `Authorization: Bearer ...`), `Role` enum, `Scope` newtype.

### `ed-cache`

`deadpool-redis` wrapper, token-bucket `RateLimiter` (Redis Lua-ready),
`Session<T>` helper.

### `ed-persistence-postgres`

- `PlatformDb` (sqlx::PgPool wrapped)
- `OutboxMessage` + `OutboxStore` trait with `EfOutboxStore` impl
- `RowStamp` columns + `sqlx::migrate!` based migration
- `make_outbox(topic, aggregate_type, aggregate_id, &EventMessage<T>)`
  factory

### `ed-persistence-mongo`

- `MongoDb` connection helper
- `MongoRepo<T>` (collection name via `T::COLLECTION`, conventions
  for `created_at`/`updated_at`/`is_deleted`/`deleted_at`)
- `AuditFields` struct + soft-delete / touch

### `ed-messaging-rabbitmq`

- `IEventBus` trait (`publish<T>`)
- `RabbitEventBus` (lapin, topic exchange, publisher confirms)
- `TypeObjectResolver` (header-driven dispatch)
- `Topology` declarator (declares exchanges/queues/bindings from JSON)
- `OutboxRelayService` (background loop: claim from Postgres outbox ->
  publish to RabbitMQ -> mark sent / retry / dead-letter)

## Transactional outbox

```
1. Business code mutates entities and calls
   outbox.append(make_outbox(topic, agg_type, agg_id, &event)).
2. SaveChanges commits business rows + outbox row in the SAME transaction.
3. OutboxRelayService (background) does:
     a. SELECT * FROM outbox_messages WHERE status IN (0,1) AND
        next_attempt_at <= now() FOR UPDATE SKIP LOCKED LIMIT N
     b. for each row: publish to RabbitMQ with mandatory=true +
        confirm-select
     c. on broker ack: mark sent; on nack/timeout: mark failed with
        exponential backoff; after MAX_ATTEMPTS: dead-letter
4. Consumers must dedupe on OutboxMessage.Id (at-least-once delivery).
```

## RabbitMQ topology

See [`infra/docker/rabbit/definitions.json`](../infra/docker/rabbit/definitions.json).

| Exchange          | Type   | Purpose                      |
|-------------------|--------|------------------------------|
| `ed.events`       | topic  | All domain events            |
| `ed.events.dlx`   | topic  | Dead-letter exchange         |

Queues + bindings:
- `ed.room-service`     <- `room.*`
- `ed.doc-service`      <- `document.*`
- `ed.latex-service`    <- `latex.*`
- `ed.audit`            <- `#.audit.recorded`
- `ed.realtime-gateway` <- `*` (catch-all, drives the SSE fanout)

Routing-key convention: `<bounded-context>.<aggregate>.<event>`.

## Services (`backend/`)

Each is a thin `axum` binary that:

1. Loads config from env (HOST, PORT, DATABASE_URL, MONGO_URL, REDIS_URL,
   RABBITMQ_URL, SERVICE_NAME).
2. Calls `ed_observability::init_tracing("svc-name", true)`.
3. Connects to Postgres, Mongo, Redis, RabbitMQ.
4. Starts the `OutboxRelayService` (publisher side).
5. Mounts the HTTP routes (`/healthz`, `/api/...`).
6. Mounts the WS routes (where applicable: `room-service`,
   `doc-service`).
7. Starts the consumer workers for its bounded context.
8. Listens on `:8080`.

### `room-service`

REST: `POST /api/rooms`, `GET /api/rooms/{id}`, `GET /api/rooms`.
WS: `GET /ws/room/{id}` -- CRDT whiteboard, ported from the legacy
`backend/src/crdt/state.rs` with an O(log n) BTreeMap ordering improvement.

### `doc-service`

REST: `POST /api/documents`, `GET /api/documents/{id}`,
`GET /api/documents`.
WS: `GET /ws/doc/{id}` -- rich-text CRDT, finally wired in (the
existing `backend/src/documents/crdt.rs` was previously unused).

### `latex-service`

REST: `POST /api/latex/compile`, `POST /api/latex/to-docx`. Async API:
publish `latex.compile_requested`; consumer compiles and publishes
`latex.compile.{succeeded,failed}` or `latex.docx.{generated,failed}`.

`texlive-latex-base` and friends are baked into the runtime image
(via `INSTALL_TEX=1` build arg on `Dockerfile.rust-service`).

## Gateway (`gateway/`)

Python FastAPI app. Single entrypoint for the SPA.

### Routers

| Router                | Prefix          | Responsibility                                |
|-----------------------|-----------------|-----------------------------------------------|
| `auth`                | `/auth`         | RS256 JWT issuer, JWKS endpoint               |
| `api`                 | `/api/v1`       | Reverse-proxy to upstream services            |
| `ws`                  | `/ws`           | WebSocket proxy (auth + bidirectional)       |
| `realtime`            | `/api/realtime` | RabbitMQ -> Server-Sent Events fanout         |
| `health`              | `/healthz`      | Liveness probe + JWKS endpoint                |

### Adapters

- `adapters/rabbit.py` -- `aio-pika` connection, consumer for `room.*`
- `adapters/mongo.py` -- `motor` async driver
- `adapters/redis.py` -- `redis.asyncio` (cache, rate-limit, idempotency)
- `adapters/postgres.py` -- `asyncpg` pool (auth, sessions)

### Middleware

- `middleware/auth.py` -- validates `Authorization: Bearer ...`
- `middleware/rate_limit.py` -- per-key token bucket (Redis Lua)
- `middleware/idempotency.py` -- `Idempotency-Key` header replay
- `middleware/correlation.py` -- `X-Correlation-Id` propagation
- `middleware/logging.py` -- structured logs

## Configuration

All configuration flows through env vars; see `infra/.env.example`.

| Variable                       | Used by         | Default                                |
|--------------------------------|-----------------|----------------------------------------|
| `DATABASE_URL`                 | Rust services   | `postgres://ed:ed@postgres:5432/ed`    |
| `MONGO_URL`                    | Rust services   | `mongodb://mongo:27017/ed`             |
| `REDIS_URL`                    | Rust + gateway  | `redis://redis:6379`                   |
| `RABBITMQ_URL`                 | Rust + gateway  | `amqp://guest:guest@rabbit:5672/`      |
| `JWT_ISSUER`                   | gateway         | `ed-gateway`                           |
| `JWT_AUDIENCE`                 | gateway         | `ed-services`                          |
| `INTERNAL_SERVICE_TOKEN_SECRET`| gateway         | `changeme` (override in prod!)         |
| `RUST_LOG`                     | Rust services   | `info,<crate>=debug,tower_http=info`   |
| `OTEL_EXPORTER_OTLP_ENDPOINT`  | all             | unset (logs only)                      |

## Operations

See `README.md` for the operator runbook (logs, restart, reset, etc.).

## Decisions and trade-offs

- **axum vs actix-web** -- we switched the new services to `axum` because
  it composes cleanly with `lapin` and `tokio_util::CancellationToken`
  (the legacy `actix-web` actor runtime doesn't share with `lapin`).
- **RabbitMQ over Kafka** -- chosen for topic-exchange ergonomics,
  out-of-the-box dead-letter exchanges, and the fact that the operation
  scale does not require Kafka's partition-tier throughput.
- **Polyglot DB** -- Postgres for transactional + outbox, Mongo for
  free-form snapshots, Redis for cache + rate-limit. The repository
  pattern hides this per-aggregate.
- **One `outbox_messages` table per service** -- not a shared database
  per service. Each service owns its outbox and the corresponding
  business tables. (Inter-service events go through the broker only.)
- **Service-to-service auth via internal HMAC tokens** -- swapped for
  mTLS in production.

## Open questions / future work

- Real testcontainer tests (CI). The current `tests/` directories hold
  property-based unit tests; integration tests are scaffolded and run
  against testcontainers in CI.
- Helm charts for production (the compose file is dev-only).
- `auth-service` (a real Rust OIDC issuer, mirroring RPBI's
  `identity-service`) -- the Python gateway currently owns user
  creation; we may split.
- OpenTelemetry exporter wiring (`OTEL_EXPORTER_OTLP_ENDPOINT` is
  honored by tracing-subscriber; downstream exporters need to be set
  up per environment).
- CRDT snapshot persistence in `doc-service` (every 100 ops or 5 sec
  per the design; not yet implemented).
