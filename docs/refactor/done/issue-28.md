# Issue #28 -- [infra] Compose -- Postgres, Mongo, Redis, RabbitMQ, services, gateway, frontend

**Milestone:** M3-room  
**Status:** Done (PR auto-merged).

## What was done

Author `infra/docker-compose.yml` with:
- `postgres:16-alpine` (POSTGRES_DB=ed, POSTGRES_USER=ed, POSTGRES_PASSWORD=ed), volume pg, healthcheck pg_isready.
- `mongo:7`, volume mongo.
- `redis:7-alpine`, healthcheck redis-cli ping.
- `rabbitmq:3.13-management-alpine`, mounts `./docker/rabbit/{rabbitmq.conf,definitions.json}`, healthcheck `rabbitmq-diagnostics ping`.
- `room-service`, `doc-service`, `latex-service`, `gateway`, `frontend`. Only `gateway` (8080) and `frontend` (5173) expose host ports.
- `depends_on` with `condition: service_healthy` for stateful services.

**Acceptance**
- `docker compose -f infra/docker-compose.yml up` brings the whole stack online with `docker compose ps` showing all containers `healthy`.
- `/healthz` on every service returns 200 within 60s.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-28.md` recording the work for issue #28.
