# Issue #29 -- [infra] Per-service `Dockerfile.rust-service` (multi-stage shared image)

**Milestone:** M3-room  
**Status:** Done (PR auto-merged).

## What was done

Author `infra/docker/Dockerfile.rust-service`:
- ARG `SERVICE` -- the workspace member name (room-service / doc-service / latex-service).
- Builder: `rust:1.81-slim`, copy root + workspace, `cargo build -p $SERVICE --release`.
- Runtime: `debian:bookworm-slim`, install ca-certs, copy binary + `infra/docker/rust-runtime` (entrypoint script that reads OTEL envs and runs the binary).
- ENV HOST=0.0.0.0 PORT=8080, EXPOSE 8080.

**Acceptance**
- Each service builds independently: `docker build --build-arg SERVICE=room-service -f infra/docker/Dockerfile.rust-service .`.
- Image size < 200 MB for room/doc, larger budget for latex (texlive).

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-29.md` recording the work for issue #29.
