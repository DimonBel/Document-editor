# M6: E2E parity
Bring up the stack with `docker compose -f infra/docker-compose.yml up`.

Verify the following parity scenarios (all run by the Python E2E suite under `tests/e2e/`):
- POST /api/rooms -> 201 + room
- GET /api/rooms/{id} -> 200 + room
- POST /api/documents -> 201 + document
- POST /api/latex/compile -> 200 + PDF
- WS /ws/room/{id} -- two clients converge
- WS /ws/doc/{id}  -- two clients converge
