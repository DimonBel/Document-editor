import { nanoid } from 'nanoid';
import { DrawElement, Operation, OpType } from '../../types';

export class CRDTDocument {
  #clientId: string;
  #elements: Map<string, DrawElement> = new Map();
  #order: string[] = [];
  #ops: Operation[] = [];
  #clock = 0;

  constructor(clientId: string) {
    this.#clientId = clientId;
  }

  addElement(type: string, data: Record<string, unknown>): Operation {
    const id = nanoid(10);
    const lamport = ++this.#clock;
    const element: DrawElement = { id, type, ...data };
    const op = this.#makeOp(lamport, { type: 'Insert', element } as OpType);

    this.#elements.set(id, element);
    this.#order.push(id);
    this.#ops.push(op);
    return op;
  }

  deleteElement(id: string): Operation {
    const lamport = ++this.#clock;
    const op = this.#makeOp(lamport, { type: 'Delete', id } as OpType);

    this.#elements.delete(id);
    this.#order = this.#order.filter((x) => x !== id);
    this.#ops.push(op);
    return op;
  }

  updateElement(id: string, updates: Record<string, unknown>): Operation | null {
    const existing = this.#elements.get(id);
    if (!existing) return null;

    const lamport = ++this.#clock;
    const element: DrawElement = { ...existing, ...updates };
    const op = this.#makeOp(lamport, { type: 'Update', id, element } as OpType);

    this.#elements.set(id, element);
    this.#ops.push(op);
    return op;
  }

  applyRemote(op: Operation) {
    this.#clock = Math.max(this.#clock, op.lamport) + 1;
    if (this.#ops.some((o) => o.id === op.id)) return;

    const { type } = op.op;

    if (type === 'Insert') {
      const { element } = op.op as { type: 'Insert'; element: DrawElement };
      if (!this.#elements.has(element.id)) {
        const pos = this.#insertPos(op.lamport, op.clientId);
        this.#order.splice(pos, 0, element.id);
        this.#elements.set(element.id, element);
      }
    } else if (type === 'Delete') {
      const { id } = op.op as { type: 'Delete'; id: string };
      this.#elements.delete(id);
      this.#order = this.#order.filter((x) => x !== id);
    } else if (type === 'Update') {
      const { id, element } = op.op as { type: 'Update'; id: string; element: DrawElement };
      if (this.#elements.has(id)) {
        this.#elements.set(id, element);
      }
    }

    this.#ops.push(op);
  }

  sync(serverElements: DrawElement[] = []) {
    this.#elements.clear();
    this.#order = [];
    for (const el of serverElements) {
      this.#elements.set(el.id, el);
      this.#order.push(el.id);
    }
  }

  getOrderedElements(): DrawElement[] {
    return this.#order.map((id) => this.#elements.get(id)).filter((el): el is DrawElement => el !== undefined);
  }

  #makeOp(lamport: number, op: OpType): Operation {
    return { id: nanoid(10), clientId: this.#clientId, lamport, op };
  }

  #insertPos(lamport: number, clientId: string): number {
    let pos = this.#order.length;
    for (let i = this.#order.length - 1; i >= 0; i--) {
      const eid = this.#order[i];
      const prev = [...this.#ops]
        .reverse()
        .find((o) => o.op.type === 'Insert' && (o.op as { type: 'Insert'; element: DrawElement }).element?.id === eid);
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