# Issue #20 -- [packages] Bootstrap `ed.contracts` — wire types & topic catalog

**Milestone:** M2-gateway  
**Status:** Done (PR auto-merged).

## What was done

Create `packages/contracts/`:

- `EventMessage<T>` envelope (Serializable/Deserializable): `id: Uuid`, `occurred_at: DateTime<Utc>`, `service_name`, `topic`, `correlation_id`, `schema_version`, `data: T`. `#[serde(rename_all = "snake_case")]` for all enums.
- `Topics` static class with `pub mod` partials per bounded context:
  - `topics::room` -- `room.created`, `room.updated`, `room.deleted`, `room.user_joined`, `room.user_left`, `room.snapshot_requested`
  - `topics::document` -- `document.created`, `document.updated`, `document.deleted`, `document.commit_recorded`
  - `topics::latex` -- `latex.compile_requested`, `latex.compile_succeeded`, `latex.compile_failed`, `latex.docx_generated`, `latex.docx_failed`
  - `topics::audit` -- `*.audit.recorded` (e.g. `room.audit.recorded`)
- Event payload records under `events::{room,document,latex}`:
  - `RoomCreatedEvent`, `RoomUpdatedEvent`, `RoomDeletedEvent`, `RoomUserJoinedEvent`, `RoomUserLeftEvent`, `RoomSnapshotRequestedEvent`
  - `DocumentCreatedEvent`, `DocumentUpdatedEvent`, `DocumentDeletedEvent`, `DocumentCommitRecordedEvent`
  - `LatexCompileRequestedEvent`, `LatexCompileSucceededEvent`, `LatexCompileFailedEvent`, `LatexDocxGeneratedEvent`, `LatexDocxFailedEvent`
  - `AuditRecordedEvent` (carries a `serde_json::Value` of the audited entity)
- A JSON Schema for the envelope at `packages/contracts/schema/event_message.schema.json` (mirrors what the Python gateway will deserialize).

NO dependencies beyond `serde`, `serde_json`, `uuid`, `chrono`.

**Acceptance**
- `cargo test -p ed_contracts` passes (envelope round-trip test + topic constant tests).
- Python `pydantic` models generated from the JSON Schema match exactly (validate in CI later).
- Both Rust backend services AND Python gateway consume the same `topics::*` constants (verified post-M2).

## Where the code lives

The bulk of the implementation was authored in the initial scaffolding commit (see commit `449281a` on `master`).
This tracking PR adds `docs/refactor/done/issue-20.md` recording the work for issue #20.
