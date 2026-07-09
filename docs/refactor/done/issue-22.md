# Issue #22 -- [packages] Bootstrap `ed.observability` -- tracing + OTLP

**Milestone:** M2-gateway  
**Status:** Done (PR auto-merged).

## What was done

Create `packages/observability/`:

- `init_tracing(service_name: &str, otlp_endpoint: Option<&str>)`:
  - Always `tracing_subscriber::fmt::layer().json()` to stdout (container-friendly).
  - If `otlp_endpoint` is set: also attach `opentelemetry_otlp::SpanExporter` + `tracing_opentelemetry::layer()`.
  - Read `RUST_LOG` env, default `info,ed_*=debug`.
- `correlation::CorrelationId(String)` newtype with `FromRequest`/extractor for axum + propagation helpers.
- `metrics::Metrics` struct that registers a Prometheus `Registry` and exposes helpers `inc_room_created()`, `inc_latex_compile_seconds(secs)`, etc.

**Acceptance**
- `cargo test -p ed_observability` covers `init_tracing` smoke + correlation propagation.
- OTLP export is no-op when endpoint unset (dev/test friendly).

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-22.md` recording the work for issue #22.
