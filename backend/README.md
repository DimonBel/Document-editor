# `backend/` -- the new Rust services

This folder hosts the **service workspace** for Document-editor.

```
backend/
├── room-service/    # Whiteboards (Mongo + Postgres + WS)
├── doc-service/     # Documents (Postgres + Redis + WS)
└── latex-service/   # LaTeX compile (pdflatex + bounded exec)
```

Build everything with `cargo build --workspace` at the repo root,
or a single service with `cargo run -p room-service`.

## Legacy code

The legacy Actix-web monolith (`backend/src/broadcast/...`,
`backend/src/handlers/...`, etc.) was removed when the workspace
vertical slice landed (PR #177). It is gone -- not archived --
because:

1. The new services cover every endpoint the legacy app exposed,
2. The gateway-fronted Rust services supersede it for HTTP, and
3. The in-process WS handler in the legacy `handlers/websocket.rs`
   is now `backend/{room,doc,latex}-service/src/ws.rs` (translated
   to axum + the shared `RoomHub` / `DocHub` broadcast registry).

If you need a historical diff for refs, the pre-removal commit is
44aacf40 (the last commit before this folder was emptied) -- it
contains the legacy code under `backend/src/`. To restore it:

```
git checkout 44aacf40 -- backend/src
```

We do **not** recommend that: the legacy code stays around as an
archaeology lesson for the original squad-merge, single-mutex
design choices the refactor was meant to retire.
