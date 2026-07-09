# Issue #33 -- [infra] Wire `.env.example` and pydantic-settings for the gateway

**Milestone:** M4-doc  
**Status:** Done (PR auto-merged).

## What was done

Create `infra/.env.example` (committed) listing every var the stack consumes:
- `DATABASE_URL=postgres://ed:ed@postgres:5432/ed`
- `MONGO_URL=mongodb://mongo:27017/ed`
- `REDIS_URL=redis://redis:6379`
- `RABBITMQ_URL=amqp://guest:guest@rabbit:5672/ed`
- `JWT_ISSUER=ed-gateway`, `JWT_AUDIENCE=ed-services`, `JWKS_URL=http://gateway/.well-known/jwks.json`
- `INTERNAL_SERVICE_TOKEN_SECRET=changeme` (overridable in prod)
- `OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317`

Mirror these in `gateway/app/config.py` via `pydantic-settings.Settings` so the gateway fails fast on missing required vars at startup.

**Acceptance**
- `docker compose --env-file infra/.env.example config` validates.
- Starting the gateway without `JWT_ISSUER` panics with a clear error.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-33.md` recording the work for issue #33.
