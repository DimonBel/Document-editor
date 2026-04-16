import { nanoid } from 'nanoid';

/**
 * Client-side CRDT replica (operation-based, YATA-inspired ordering).
 *
 * Rules:
 *  - Every mutation returns an `Operation` that must be broadcast to the server.
 *  - Remote ops arrive via `applyRemote(op)`.
 *  - On room join the server sends a snapshot; call `sync(elements)` to initialise.
 *  - Lamport clock advances monotonically; tiebreaker is `clientId` string order.
 *
 * This class is intentionally framework-agnostic — no React, no Zustand.
 * React integration lives in `features/collaboration/useCollaboration.js`.
 */
export class CRDTDocument {
  #clientId;
  #elements = new Map();   // id → element
  #order    = [];           // ordered element IDs
  #ops      = [];           // full operation log (for dedup + position lookup)
  #clock    = 0;

  constructor(clientId) {
    this.#clientId = clientId;
  }

  // ── Local mutations ────────────────────────────────────────────────────

  addElement(type, data) {
    const id     = nanoid(10);
    const lamport = ++this.#clock;
    const element = { id, type, ...data };
    const op = this.#makeOp(lamport, { type: 'Insert', element });

    this.#elements.set(id, element);
    this.#order.push(id);
    this.#ops.push(op);
    return op;
  }

  deleteElement(id) {
    const lamport = ++this.#clock;
    const op = this.#makeOp(lamport, { type: 'Delete', id });

    this.#elements.delete(id);
    this.#order = this.#order.filter((x) => x !== id);
    this.#ops.push(op);
    return op;
  }

  updateElement(id, updates) {
    const existing = this.#elements.get(id);
    if (!existing) return null;

    const lamport = ++this.#clock;
    const element = { ...existing, ...updates };
    const op = this.#makeOp(lamport, { type: 'Update', id, element });

    this.#elements.set(id, element);
    this.#ops.push(op);
    return op;
  }

  // ── Remote mutations ───────────────────────────────────────────────────

  applyRemote(op) {
    this.#clock = Math.max(this.#clock, op.lamport) + 1;
    if (this.#ops.some((o) => o.id === op.id)) return; // idempotent

    const { type } = op.op;

    if (type === 'Insert') {
      const { element } = op.op;
      if (!this.#elements.has(element.id)) {
        const pos = this.#insertPos(op.lamport, op.clientId);
        this.#order.splice(pos, 0, element.id);
        this.#elements.set(element.id, element);
      }
    } else if (type === 'Delete') {
      this.#elements.delete(op.op.id);
      this.#order = this.#order.filter((x) => x !== op.op.id);
    } else if (type === 'Update') {
      if (this.#elements.has(op.op.id)) {
        this.#elements.set(op.op.id, op.op.element);
      }
    }

    this.#ops.push(op);
  }

  // ── Full-state sync (on join) ──────────────────────────────────────────

  sync(serverElements = []) {
    this.#elements.clear();
    this.#order = [];
    for (const el of serverElements) {
      this.#elements.set(el.id, el);
      this.#order.push(el.id);
    }
  }

  // ── Read ───────────────────────────────────────────────────────────────

  getOrderedElements() {
    return this.#order.map((id) => this.#elements.get(id)).filter(Boolean);
  }

  // ── Private helpers ────────────────────────────────────────────────────

  #makeOp(lamport, op) {
    return { id: nanoid(10), clientId: this.#clientId, lamport, op };
  }

  #insertPos(lamport, clientId) {
    let pos = this.#order.length;
    for (let i = this.#order.length - 1; i >= 0; i--) {
      const eid  = this.#order[i];
      const prev = [...this.#ops]
        .reverse()
        .find((o) => o.op.type === 'Insert' && o.op.element?.id === eid);
      if (prev) {
        const before =
          prev.lamport < lamport ||
          (prev.lamport === lamport && prev.clientId < clientId);
        if (before) break;
        pos = i;
      }
    }
    return pos;
  }
}
