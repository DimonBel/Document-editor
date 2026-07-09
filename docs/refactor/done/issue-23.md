# Issue #23 -- [packages] Bootstrap `ed.auth` -- RS256 JWT + CurrentUser

**Milestone:** M2-gateway  
**Status:** Done (PR auto-merged).

## What was done

Create `packages/auth/`:

- `JwtVerifier { issuer, audience, jwks_url }`: fetches JWKS, caches via `ed.cache`, validates RS256.
- `CurrentUser { id: UserId, email, roles, scopes, correlation_id }` extractor (axum) parsed from claims.
- `Role` enum (`User`, `Admin`, `Service`); `Scope` newtype with string parsing (`rooms:read`, `documents:write`, etc.).
- `#[derive(Authorize)]`-style macro on axum handlers: `#[require(Role::User, scope = "rooms:write")]`.
- Service-to-service token support: `internal_token(secret) -> String` for gateway->service calls (symmetric HMAC for v1, swap to mTLS later).

**Acceptance**
- `cargo test -p ed_auth`: unit tests for claim parsing, expired token, wrong issuer/audience.
- JWKS fetched exactly once per cache TTL.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-23.md` recording the work for issue #23.
