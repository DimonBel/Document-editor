# Issue #36 -- [gateway] Auth: RS256 JWT issuer + JWKS endpoint

**Milestone:** M4-doc  
**Status:** Done (PR auto-merged).

## What was done

Implement:
- `app/security/jwt.py` -- generate RSA keypair on startup (or load from disk), expose `sign(claims, ttl) -> str` and `verify(token) -> Claims`.
- `app/security/jwks.py` -- `GET /.well-known/jwks.json` returns the public JWK set.
- `app/routers/auth.py`:
  - `POST /auth/login` -- exchanges username+password for RS256 JWT (short-lived) + refresh token.
  - `POST /auth/refresh` -- exchanges refresh token for new JWT (rotates refresh token).
  - `POST /auth/internal` -- exchanges service credentials for an internal-token (used by gateway -> upstream services).
- `app/middleware/auth.py` -- verifies `Authorization: Bearer <jwt>` on protected routes, attaches `CurrentUser` to request scope.

**Acceptance**
- `pytest tests/test_auth.py` -- login, refresh, expired token, wrong issuer, missing scope.
- `curl http://127.0.0.1:8080/.well-known/jwks.json` returns valid JWK set.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-36.md` recording the work for issue #36.
