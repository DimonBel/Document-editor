# Issue #32 -- [infra] Healthchecks + startup ordering (compose)

**Milestone:** M3-room  
**Status:** Done (PR auto-merged).

## What was done

Wire healthchecks for every stateful service in compose (postgres, mongo, redis, rabbit) and per-service `/healthz` endpoints (added in M2/M3+).

- Postgres: `pg_isready -U ed -d ed`.
- Mongo: `mongosh --quiet --eval 'db.runCommand({ping:1}).ok'`.
- Redis: `redis-cli ping`.
- Rabbit: `rabbitmq-diagnostics check_running && rabbitmq-diagnostics check_local_node_health`.

**Acceptance**
- `docker compose -f infra/docker-compose.yml config` validates; restart-on-failure respects `depends_on: service_healthy`.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-32.md` recording the work for issue #32.
