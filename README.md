# Document-editor

A real-time collaborative document-editing platform. The legacy app was a
single Rust binary; the codebase has been refactored into a service-oriented
architecture with reusable Rust crates, three thin Rust services, a Rust
gateway, and a Docker-first local stack. Everything from the SPA to the
last backing service is Rust.

> **TL;DR** -- `docker compose -f infra/docker-compose.yml up` brings up the
> whole stack. Frontend on http://localhost:5173, gateway on
> http://localhost:8080, RabbitMQ management UI on http://localhost:15672
> (guest / guest).

---

## Architecture at a glance

```
                                  ┌─────────────────────────────┐
                                  │  frontend (nginx + Vite SPA)│
                                  └──────────────┬──────────────┘
                                                 │ /api, /ws
                                                 ▼
                                  ┌─────────────────────────────┐
                                  │  gateway       (Rust · axum)│
                                  │  auth · reverse-proxy · WS  │
                                  │  rate-limit · idempotency   │
                                  │  correlation · SSE fanout   │
                                  └──────┬───┬────────┬──────────┘
                                         │   │        │
                ┌────────────────────────┘   │        └────────────────────────┐
                │                            │                                 │
                ▼                            ▼                                 ▼
   ┌────────────────────┐    ┌────────────────────┐                ┌────────────────────┐
   │  room-service      │    │  doc-service       │                │  latex-service     │
   │  (Rust · axum)     │    │  (Rust · axum)     │                │  (Rust · axum)     │
   │  Mongo + Postgres  │    │  Postgres + Redis  │                │  pdflatex + DOCX   │
   └────────┬───────────┘    └────────┬───────────┘                └────────┬───────────┘
            │                         │                                  │
            └─────────► RabbitMQ (ed.events, ed.events.dlx) ◄───────────┘
                              │
                              ▼
                  ┌──────────────────────────┐
                  │  Postgres + Mongo + Redis │
                  └──────────────────────────┘
```

- **`packages/`** -- 9 reusable Rust crates (the "DULL" services).
- **`backend/`** -- 3 thin Rust binaries that compose packages.
- **`gateway/`** -- Python FastAPI: auth issuer, reverse-proxy, WS proxy.
- **`infra/`** -- docker-compose, Dockerfiles, RabbitMQ topology, `.env`.
- **`frontend/`** -- unchanged (proxies `/api` & `/ws` to the gateway).

The detailed design lives in **[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)**.

---

## Repository layout

```
Document-editor/
├── Cargo.toml                  # umbrella workspace
├── rust-toolchain.toml
├── packages/                   # 9 reusable Rust crates
│   ├── domain/                 # ed-domain       -- pure types
│   ├── contracts/              # ed-contracts    -- wire types, topics, events
│   ├── errors/                 # ed-errors       -- AppError + ProblemDetails
│   ├── observability/          # ed-observability-- tracing + correlation
│   ├── auth/                   # ed-auth         -- JWT + CurrentUser
│   ├── cache/                  # ed-cache        -- Redis + RateLimiter
│   ├── persistence-postgres/   # ed-...          -- sqlx + outbox + RowStamp
│   ├── persistence-mongo/      # ed-...          -- MongoDB conventions
│   └── messaging-rabbitmq/     # ed-...          -- lapin + OutboxRelay
├── backend/                    # 3 Rust services
│   ├── room-service/           # whiteboards + WS
│   ├── doc-service/            # documents + WS (CRDT)
│   └── latex-service/          # pdflatex + DOCX
├── gateway/                    # Rust API gateway (axum)
│   ├── src/{config,error,state,security,auth,proxy,ws,
│   │       middleware,realtime,health,app}.rs
│   └── tests/auth.rs
├── infra/
│   ├── docker-compose.yml      # the full local stack
│   ├── docker/Dockerfile.rust-service  # shared for all Rust services + gateway
│   └── docker/rabbit/{definitions.json,rabbitmq.conf}
├── frontend/                   # Vite + React (unchanged)
├── docs/
│   ├── ARCHITECTURE.md
│   └── refactor/done/          # per-issue tracking records
├── .github/workflows/
├── .gitignore
└── README.md
```

---

## Quick start (Docker)

Prerequisites: **Docker** (with Compose v2) and **~8 GB of free RAM**.

```bash
# 1. Clone
git clone https://github.com/DimonBel/Document-editor
cd Document-editor

# 2. (Optional) copy the env template
cp infra/.env.example infra/.env

# 3. Bring up the whole stack
docker compose -f infra/docker-compose.yml up -d

# 4. Tail logs
docker compose -f infra/docker-compose.yml logs -f

# 5. Verify
curl http://localhost:8080/healthz                                  # gateway
curl http://localhost:15672  -u guest:guest                        # RabbitMQ UI
xdg-open http://localhost:5173                                      # SPA
```

To **tear down** (keep volumes):

```bash
docker compose -f infra/docker-compose.yml down
```

To **tear down and reset data**:

```bash
docker compose -f infra/docker-compose.yml down -v
```

### What's running, and where

| Service          | Internal port | Host port | Notes                                      |
|------------------|---------------|-----------|--------------------------------------------|
| `frontend`       | 80            | 5173      | Vite SPA, nginx in production              |
| `gateway`        | 8080          | 8080      | FastAPI; auth, reverse-proxy, WS proxy     |
| `room-service`   | 8080          | --        | Reachable via the gateway                  |
| `doc-service`    | 8080          | --        | Reachable via the gateway                  |
| `latex-service`  | 8080          | --        | Reachable via the gateway                  |
| `postgres`       | 5432          | 5432      | `ed / ed`                                 |
| `mongo`          | 27017         | 27017     | DB `ed`                                   |
| `redis`          | 6379          | 6379      | cache + rate-limit + idempotency           |
| `rabbit`         | 5672          | 5672      | AMQP; mgmt UI on 15672 (guest / guest)     |

---

## Local development (without Docker, for the Rust side)

Prerequisites: **Rust 1.81+**, **PostgreSQL 16+**, **MongoDB 7+**,
**Redis 7+**, **RabbitMQ 3.13+**.

```bash
# 1. Build the workspace
cargo build --workspace

# 2. Run unit tests (no infrastructure required)
cargo test --workspace --exclude '*-integration-tests' --exclude '*-end-to-end-tests'

# 3. Run integration tests (testcontainers -- spin up real infra)
cargo test --workspace --features integration

# 4. Start an individual service
cd backend/room-service
DATABASE_URL=postgres://ed:ed@localhost:5432/ed \
  MONGO_URL=mongodb://localhost:27017/ed \
  REDIS_URL=redis://localhost:6379 \
  RABBITMQ_URL=amqp://guest:guest@localhost:5672/ \
  cargo run
```

For the **gateway**:

```bash
cargo run -p gateway    # builds + runs on :8080 with auto-reload via cargo-watch
cargo test -p gateway   # 13 unit tests (auth, JWT, status codes, ProblemDetails)
```

---

## Crate map

| Crate                      | Purpose                                                  | Public surface                                  |
|----------------------------|----------------------------------------------------------|-------------------------------------------------|
| `ed-domain`                | Pure types: entities, value objects, IDs, errors         | `Room`, `Document`, `RoomId`, `DocumentId`, ...  |
| `ed-contracts`             | Wire types: envelope, topic catalog, event payloads      | `EventMessage<T>`, `Topics`, `events::*`        |
| `ed-errors`                | `AppError` -> RFC-7807 ProblemDetails                    | `AppError`, `ProblemDetails`                    |
| `ed-observability`         | `tracing` + correlation + OTel plumbing                  | `init_tracing`, `correlation::*`                |
| `ed-auth`                  | JWT verifier + `CurrentUser` extractor + scope parsing   | `JwtVerifier`, `CurrentUser`, `Role`, `Scope`   |
| `ed-cache`                 | `deadpool-redis` + token-bucket rate limiter + sessions  | `Cache`, `RateLimiter`, `Session`               |
| `ed-persistence-postgres`  | sqlx + outbox table + RowStamp + migrations              | `PlatformDb`, `OutboxStore`, `make_outbox`      |
| `ed-persistence-mongo`     | MongoDB driver + conventions + `MongoRepo<T>`            | `MongoDb`, `MongoRepo<T>`, `AuditFields`        |
| `ed-messaging-rabbitmq`    | lapin + type-resolver + `OutboxRelayService`             | `IEventBus`, `Topology`, `OutboxRelayService`   |

Dependency direction (no cycles):

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

---

## RabbitMQ topology

Declared by `infra/docker/rabbit/definitions.json`, applied at broker start.

| Exchange          | Type   | Purpose                              |
|-------------------|--------|--------------------------------------|
| `ed.events`       | topic  | All domain events                    |
| `ed.events.dlx`   | topic  | Dead-letter exchange                 |

| Queue                  | Bindings                  | Consumer                |
|------------------------|---------------------------|-------------------------|
| `ed.room-service`      | `room.*`                  | `room-service`          |
| `ed.doc-service`       | `document.*`              | `doc-service`           |
| `ed.latex-service`     | `latex.*`                 | `latex-service`         |
| `ed.audit`             | `#.audit.recorded`        | (audit consumer, future)|
| `ed.realtime-gateway`  | `*`                       | `gateway` (SSE fanout)  |

Routing-key convention: **`<bounded-context>.<aggregate>.<event>`** --
e.g. `room.created`, `document.commit_recorded`,
`latex.compile.succeeded`.

The outbox pattern in `ed-persistence-postgres` + `ed-messaging-rabbitmq`
guarantees at-least-once delivery: business writes and outbox rows are
committed in the same DB transaction; the relay polls the outbox, publishes
to RabbitMQ, and marks rows as `Sent` on broker ack. Consumers must dedupe
on `OutboxMessage.Id`.

---

## Configuration

| Variable                       | Used by         | Default                                |
|--------------------------------|-----------------|----------------------------------------|
| `DATABASE_URL`                 | Rust services   | `postgres://ed:ed@localhost:5432/ed`   |
| `MONGO_URL`                    | Rust services   | `mongodb://localhost:27017/ed`         |
| `REDIS_URL`                    | Rust + gateway  | `redis://localhost:6379`               |
| `RABBITMQ_URL`                 | Rust + gateway  | `amqp://guest:guest@localhost:5672/`   |
| `JWT_ISSUER`                   | gateway         | `ed-gateway`                           |
| `JWT_AUDIENCE`                 | gateway         | `ed-services`                          |
| `INTERNAL_SERVICE_TOKEN_SECRET`| gateway         | `changeme` (override in production)     |
| `OTEL_EXPORTER_OTLP_ENDPOINT`  | everything      | unset (logs go to stdout only)         |
| `RUST_LOG`                     | Rust services   | `info,<crate>=debug,tower_http=info`   |

A full template lives at `infra/.env.example`.

---

## Operations

- **Logs**: `docker compose -f infra/docker-compose.yml logs -f <service>`
- **Restart one service**: `docker compose -f infra/docker-compose.yml restart <service>`
- **Rebuild one service**: `docker compose -f infra/docker-compose.yml build <service>`
- **Reset all state**: `docker compose -f infra/docker-compose.yml down -v`
- **Open RabbitMQ management UI**: http://localhost:15672 (guest/guest)
- **Open Postgres**: `psql -h localhost -U ed ed`  (password `ed`)
- **Open Mongo**: `mongosh "mongodb://localhost:27017/ed"`
- **Open Redis**: `redis-cli`

---

## Development workflow

The project is tracked on the GitHub Project board
(`<https://github.com/users/DimonBel/projects/6>`). Issues are organized by
milestone:

- `M0 Foundations` -- reusable crates
- `M1 Containers`  -- docker-compose, Dockerfiles, RabbitMQ topology
- `M2 Gateway`     -- Python FastAPI reverse-proxy + auth
- `M3 room-service`
- `M4 doc-service`
- `M5 latex-service`
- `M6 E2E + cutover`

Branch convention (mirrors `fix/ci-and-code-quality`):

- `feat/<scope>`         -- new feature
- `fix/<scope>`          -- bugfix
- `chore/<scope>`        -- non-functional change
- `refactor/<scope>`     -- internal restructuring
- `docs/<scope>`         -- documentation only

Commit convention: Conventional Commits (`feat:`, `fix:`, `chore:`, etc.).
The first line stays under 72 characters; the body explains *why*.

---

## License

MIT. See `LICENSE` (TBD -- not yet committed).

## Maintainers

- @DimonBel
