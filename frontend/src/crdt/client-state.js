import { nanoid } from 'nanoid';

/**
 * Client-side CRDT replica.
 *
 * Mirrors the server's DocumentState.  Operations are generated locally,
 * applied optimistically, then broadcast to the server which re-broadcasts
 * them to every other peer.  Remote ops arrive via `applyRemote()`.
 *
 * Ordering rule (YATA-style): two concurrent Insert ops are placed in
 * ascending order of (lamport, clientId).
 */
export class CRDTDocument {
  constructor(clientId) {
    this.clientId = clientId;
    /** @type {Map<string, object>} id → element */
    this.elements = new Map();
    /** @type {string[]} ordered element IDs */
    this.order = [];
    /** @type {object[]} full operation log */
    this.operations = [];
    this.lamportClock = 0;
  }

  // -------------------------------------------------------------------------
  // Local mutations
  // -------------------------------------------------------------------------

  addElement(type, data) {
    const id = nanoid(10);
    const lamport = ++this.lamportClock;

    const element = { id, type, ...data };

    const op = {
      id: nanoid(10),
      clientId: this.clientId,
      lamport,
      op: { type: 'Insert', element },
    };

    this.elements.set(id, element);
    this.order.push(id);
    this.operations.push(op);
    return op;
  }

  deleteElement(elementId) {
    const lamport = ++this.lamportClock;

    const op = {
      id: nanoid(10),
      clientId: this.clientId,
      lamport,
      op: { type: 'Delete', id: elementId },
    };

    this.elements.delete(elementId);
    this.order = this.order.filter((x) => x !== elementId);
    this.operations.push(op);
    return op;
  }

  updateElement(elementId, updates) {
    const existing = this.elements.get(elementId);
    if (!existing) return null;

    const lamport = ++this.lamportClock;
    const element = { ...existing, ...updates };

    const op = {
      id: nanoid(10),
      clientId: this.clientId,
      lamport,
      op: { type: 'Update', id: elementId, element },
    };

    this.elements.set(elementId, element);
    this.operations.push(op);
    return op;
  }

  // -------------------------------------------------------------------------
  // Remote mutations
  // -------------------------------------------------------------------------

  applyRemote(op) {
    // Advance Lamport clock.
    this.lamportClock = Math.max(this.lamportClock, op.lamport) + 1;

    // Idempotency.
    if (this.operations.some((o) => o.id === op.id)) return;

    const { type } = op.op;

    if (type === 'Insert') {
      const { element } = op.op;
      if (!this.elements.has(element.id)) {
        const pos = this._insertPos(op.lamport, op.clientId);
        this.order.splice(pos, 0, element.id);
        this.elements.set(element.id, element);
      }
    } else if (type === 'Delete') {
      this.elements.delete(op.op.id);
      this.order = this.order.filter((x) => x !== op.op.id);
    } else if (type === 'Update') {
      if (this.elements.has(op.op.id)) {
        this.elements.set(op.op.id, op.op.element);
      }
    }

    this.operations.push(op);
  }

  // -------------------------------------------------------------------------
  // Full-state sync (called once when joining a room)
  // -------------------------------------------------------------------------

  sync(serverElements) {
    this.elements.clear();
    this.order = [];
    for (const el of serverElements) {
      this.elements.set(el.id, el);
      this.order.push(el.id);
    }
  }

  // -------------------------------------------------------------------------
  // Helpers
  // -------------------------------------------------------------------------

  getOrderedElements() {
    return this.order.map((id) => this.elements.get(id)).filter(Boolean);
  }

  _insertPos(lamport, clientId) {
    let pos = this.order.length;
    for (let i = this.order.length - 1; i >= 0; i--) {
      const eid = this.order[i];
      const prev = [...this.operations]
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
