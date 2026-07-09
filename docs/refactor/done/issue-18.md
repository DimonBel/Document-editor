# Issue #18 -- [infra] Root umbrella Cargo workspace (Cargo.toml, rust-toolchain.toml)

**Milestone:** M2-gateway  
**Status:** Done (PR auto-merged).

## What was done

Create the umbrella `Cargo.toml` at the repo root with `[workspace] members = ["packages/*", "backend/*"]`, plus a shared resolver (crates.io, and any internal feed). Pin `rust-toolchain.toml` (stable; if `ed.persistence-postgres` needs sqlx-stable, pin to `1.81+`).

**Acceptance**
- `cargo metadata --workspace` exits 0 with both `packages/*` and `backend/*` members discovered.
- `cargo build --workspace` succeeds once all crates are stubbed.
- `rust-toolchain.toml` is committed and identical for every contributor (CI uses the same file).
- The legacy `backend/Cargo.toml` becomes `backend/<svc>/Cargo.toml` later (M3+); this issue lands the workspace shell first.

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-18.md` recording the work for issue #18.
