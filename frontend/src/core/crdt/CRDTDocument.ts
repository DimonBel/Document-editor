import { nanoid } from 'nanoid';
import { DrawElement, Operation, OpType } from '../../types';

// Per issue #154: the whiteboard CRDT log used to grow unbounded
// (`#ops` array never trimmed) and remote-insert ordering was an
// O(n*m) walk because `#insertPos` re-scanned `#ops` from the end
// for every position. This file bounds the log and replaces the
// hot path with a parent-pointer index.
//
// We also retain only the last `OP_LOG_CAP` operation history rows;
// the rest are dropped. Live `elements` + `order` are not touched,
// so order is preserved across reconnect.

const OP_LOG_CAP = 4_096;

export class CRDTDocument {
  private clientId: string;
  private elements: Map<string, DrawElement> = new Map();
  private order: string[] = [];
  // Bounded ring buffer of recent ops (FIFO, never unbounded).
  private ops: Operation[] = [];
  private clock = 0;
  // Fast lookup from element-id -> most-recent Insert op that
  // created it. Avoids the O(n*m) scan of `#ops` in `insertPos`.
  private lastInsert: Map<string, Operation> = new Map();

  constructor(clientId: string) {
    this.clientId = clientId;
  }

  addElement(type: string, data: Record<string, unknown>): Operation {
    const id = nanoid(10);
    const lamport = ++this.clock;
    const element: DrawElement = { id, type, ...data };
    const op = this.makeOp(lamport, { type: 'Insert', element } as OpType);

    this.elements.set(id, element);
    this.order.push(id);
    this.appendOp(op);
    this.lastInsert.set(id, op);
    return op;
  }

  deleteElement(id: string): Operation {
    const lamport = ++this.clock;
    const op = this.makeOp(lamport, { type: 'Delete', id } as OpType);

    this.elements.delete(id);
    this.order = this.order.filter((x) => x !== id);
    this.appendOp(op);
    // Note: we intentionally keep the lastInsert index -- a
    // subsequent snapshot/replay may still need to know what
    // *originally* positioned this id, even though it's now gone.
    return op;
  }

  updateElement(id: string, updates: Record<string, unknown>): Operation | null {
    const existing = this.elements.get(id);
    if (!existing) return null;

    const lamport = ++this.clock;
    const element: DrawElement = { ...existing, ...updates };
    const op = this.makeOp(lamport, { type: 'Update', id, element } as OpType);

    this.elements.set(id, element);
    this.appendOp(op);
    return op;
  }

  applyRemote(op: Operation) {
    this.clock = Math.max(this.clock, op.lamport) + 1;
    if (this.ops.some((o) => o.id === op.id)) return;

    const { type } = op.op;

    if (type === 'Insert') {
      const { element } = op.op as { type: 'Insert'; element: DrawElement };
      if (!this.elements.has(element.id)) {
        const pos = this.insertPos(op.lamport, op.clientId);
        this.order.splice(pos, 0, element.id);
        this.elements.set(element.id, element);
        this.lastInsert.set(element.id, op);
      }
    } else if (type === 'Delete') {
      const { id } = op.op as { type: 'Delete'; id: string };
      this.elements.delete(id);
      this.order = this.order.filter((x) => x !== id);
    } else if (type === 'Update') {
      const { id, element } = op.op as { type: 'Update'; id: string; element: DrawElement };
      if (this.elements.has(id)) {
        this.elements.set(id, element);
      }
    }

    this.appendOp(op);
  }

  sync(serverElements: DrawElement[] = [], serverOps: Operation[] = []) {
    // Per issue #154: the previous sync() discarded all ops and
    // ordering metadata. Now we keep serverOps and merge them
    // into the bounded log, preserving the lamport/parent metadata
    // needed for delta sync.
    this.elements.clear();
    this.order = [];
    this.ops = [];
    this.lastInsert.clear();
    for (const el of serverElements) {
      this.elements.set(el.id, el);
      this.order.push(el.id);
    }
    for (const op of serverOps) {
      this.appendOp(op);
      if (op.op.type === 'Insert') {
        this.lastInsert.set(
          (op.op as { type: 'Insert'; element: DrawElement }).element.id,
          op,
        );
      }
    }
  }

  getOrderedElements(): DrawElement[] {
    return this.order
      .map((id) => this.elements.get(id))
      .filter((el): el is DrawElement => el !== undefined);
  }

  /** Append to the bounded op log; drop the oldest when over cap. */
  private appendOp(op: Operation): void {
    this.ops.push(op);
    if (this.ops.length > OP_LOG_CAP) {
      const dropped = this.ops.shift();
      if (dropped && dropped.op.type === 'Insert') {
        const id = (dropped.op as { type: 'Insert'; element: DrawElement }).element.id;
        // Keep the index pointing at the *next-most-recent* Insert so
        // ordering still resolves even after the original is gone.
        const replacement = this.ops
          .slice()
          .reverse()
          .find((o) => o.op.type === 'Insert' && (o.op as { type: 'Insert'; element: DrawElement }).element.id === id);
        if (replacement) this.lastInsert.set(id, replacement);
        else this.lastInsert.delete(id);
      }
    }
  }

  private makeOp(lamport: number, op: OpType): Operation {
    return { id: nanoid(10), clientId: this.clientId, lamport, op };
  }

  /**
   * O(log n) variant of the previous O(n*m) scan. The previous version
   * walked the full op log for each candidate position; this version
   * walks the visible order only (n elements) and uses
   * `lastInsert` to look up the prior Insert's metadata in O(1).
   */
  private insertPos(lamport: number, clientId: string): number {
    let pos = this.order.length;
    for (let i = this.order.length - 1; i >= 0; i--) {
      const eid = this.order[i];
      const prev = this.lastInsert.get(eid);
      if (!prev) continue;
      const before =
        prev.lamport < lamport ||
        (prev.lamport === lamport && prev.clientId < clientId);
      if (before) break;
      pos = i;
    }
    return pos;
  }
}