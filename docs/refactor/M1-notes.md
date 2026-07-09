# M1 Notes
Healthchecks defined in `infra/docker-compose.yml`:
- postgres: `pg_isready -U ed`
- mongo:    `db.runCommand({ping:1})`
- redis:    `redis-cli ping`
- rabbit:   `rabbitmq-diagnostics ping`
- per-service `/healthz` (added in M2/M3+)

`depends_on: condition: service_healthy` is used for ordering.
