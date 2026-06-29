# Document Editor

Three editors in one app, all with live collaboration over WebSocket:

- **Whiteboard** — Konva canvas. Freehand, rectangle, ellipse, text, eraser. CRDT-merged across users.
- **Documents** — Simple rich-text editor. Last-write-wins. No OT, no CRDT.
- **LaTeX** — CodeMirror source editor with KaTeX preview, live cursors, and export to PDF and DOCX.

The frontend talks to a single Rust binary (`whiteboard-server`) that handles HTTP, WebSocket, and a pure-Rust LaTeX → DOCX pipeline (no external tools required).

## Run it

With Docker:

```bash
docker compose up --build
```

Frontend on `http://localhost`, backend on `http://localhost:8080`.

For local dev:

```bash
# backend
cd backend && cargo run

# frontend
cd frontend && npm install && npm run dev
```

The frontend dev server proxies `/api` and `/ws` to `127.0.0.1:8080`.

## Layout

```
backend/    Rust + Actix. One binary, JSON-on-disk persistence.
frontend/   React 18 + TS + Vite. antd UI, Konva canvas, CodeMirror.
docker-compose.yml
```

## LaTeX export

- **DOCX** — Backend route `POST /api/latex/to-docx`. Produces a real `.docx` with OMML math. Falls back to a TypeScript implementation in the browser if the backend is unreachable.
- **PDF** — Renders the KaTeX DOM via `html2canvas` and paginates with `jspdf`. Stays in the browser; the source never leaves the page.

## Known limitations

- Document editor uses last-write-wins on `innerHTML`. Two fast typists will overwrite each other.
- Whiteboard CRDT log is in-memory; restarting the backend resets the canvas (room metadata and LaTeX source are preserved).
- LaTeX parser is hand-rolled and narrow. See `backend/src/latex/parser.rs` for what it understands.

## License

Personal project. No license declared yet.