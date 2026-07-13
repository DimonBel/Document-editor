// Lightweight WebSocket client with:
//  - exponential-backoff reconnect (issue #151: never lose a write)
//  - bounded offline write queue (issue #151: edits during reconnect
//    are buffered and replayed on reconnect)
//  - safe JSON parsing + runtime shape validation (issue #152:
//    reject malformed frames before they reach the React state)
//  - drop-on-close of the queue if the client is destroyed so we
//    don't leak memory when the user navigates away
//
// All ops take <100 lines; dependencies stay zero (this file used
// to import nothing and that contract is preserved for the rest of
// the app to depend on).

interface WSHandlers {
  onOpen?: () => void;
  onClose?: () => void;
  onMessage?: (data: CollabMessage) => void;
  onError?: (e: Event) => void;
  /** Called when a previously-queued write was discarded (queue overflow). */
  onDropQueued?: (dropped: CollabMessage) => void;
}

// Runtime-validated collaboration message envelope. Only the fields
// our handlers actually need are decoded; unknown fields are
// preserved but not type-asserted.
export interface CollabMessage {
  type: string;
  [k: string]: unknown;
}

// Discriminated union of the subset of messages the app currently
// emits. New cases are added in lockstep with the server.
const KNOWN_TYPES = new Set<string>([
  'op', 'cursor', 'presence', 'room_sync', 'doc_sync',
  'latex_sync', 'doc_snapshot', 'room_snapshot',
]);

function safeParseMessage(raw: string): CollabMessage | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return null;
  }
  if (parsed === null || typeof parsed !== 'object') return null;
  const obj = parsed as Record<string, unknown>;
  if (typeof obj.type !== 'string') return null;
  if (!KNOWN_TYPES.has(obj.type)) return null;  // unknown shape -- drop
  return obj as CollabMessage;
}

const INITIAL_RETRY_MS = 1000;
const MAX_RETRY_MS = 30_000;
const MAX_QUEUED_WRITES = 256;

function nextDelay(prev: number): number {
  const backoff = Math.min(prev * 1.5, MAX_RETRY_MS);
  const jitter = Math.random() * backoff;
  return Math.round(backoff + jitter);
}

export class WSClient {
  private ws: WebSocket | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private retryDelay = INITIAL_RETRY_MS;
  private destroyed = false;
  private handlers: WSHandlers;
  private roomId: string;
  // Queue of writes made while disconnected. Drained on every OPEN.
  // Entries are FIFO so operation order is preserved across reconnects.
  // Bounded to MAX_QUEUED_WRITES -- on overflow, the oldest is
  // dropped and `onDropQueued` fires so the UI can mark the local
  // pending state as failed.
  private queue: CollabMessage[] = [];
  // Cap on drain rate so a slow server is not flooded with the
  // backlog the moment we reconnect.
  private drainInFlight = false;

  constructor(roomId: string, handlers: WSHandlers = {}) {
    this.roomId = roomId;
    this.handlers = handlers;
  }

  connect() {
    if (this.destroyed) return;
    if (this.ws?.readyState === WebSocket.OPEN) return;

    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${protocol}//${location.host}/ws/${this.roomId}`);

    ws.onopen = () => {
      this.retryDelay = INITIAL_RETRY_MS;
      this.handlers.onOpen?.();
      // Drain any writes that piled up while disconnected.
      void this.drain();
    };

    ws.onclose = () => {
      if (this.destroyed) return;
      this.handlers.onClose?.();
      const delay = this.retryDelay;
      this.retryDelay = nextDelay(this.retryDelay);
      this.reconnectTimer = setTimeout(() => this.connect(), delay);
    };

    ws.onerror = (e) => this.handlers.onError?.(e);

    ws.onmessage = (evt) => {
      const msg = safeParseMessage(typeof evt.data === 'string' ? evt.data : '');
      if (!msg) return;  // bad frame -- silently drop, do not crash
      this.handlers.onMessage?.(msg);
    };

    this.ws = ws;
  }

  send(data: CollabMessage) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      // Validate before sending.
      const validated = safeParseMessage(JSON.stringify(data));
      if (!validated) return;
      this.ws.send(JSON.stringify(validated));
      return;
    }
    // Offline: enqueue.
    if (this.queue.length >= MAX_QUEUED_WRITES) {
      const dropped = this.queue.shift();
      if (dropped) this.handlers.onDropQueued?.(dropped);
    }
    this.queue.push(data);
  }

  disconnect() {
    this.destroyed = true;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.queue.length = 0;
    this.ws?.close();
    this.ws = null;
  }

  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  get queueLength(): number {
    return this.queue.length;
  }

  private async drain(): Promise<void> {
    if (this.drainInFlight) return;
    if (!this.queue.length) return;
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
    this.drainInFlight = true;
    try {
      while (this.queue.length && this.ws?.readyState === WebSocket.OPEN) {
        const batch = this.queue.splice(0, Math.min(50, this.queue.length));
        for (const msg of batch) {
          if (this.ws?.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(msg));
          } else {
            // Lost the connection mid-drain; re-queue and bail.
            this.queue.unshift(...batch.splice(batch.indexOf(msg)));
            break;
          }
        }
        // Yield to the event loop so heartbeats / data frames get a turn.
        await new Promise<void>((r) => setTimeout(r, 20));
      }
    } finally {
      this.drainInFlight = false;
    }
  }
}
