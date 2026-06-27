interface WSHandlers {
  onOpen?: () => void;
  onClose?: () => void;
  onMessage?: (data: Record<string, unknown>) => void;
  onError?: (e: Event) => void;
}

const INITIAL_RETRY_MS = 1000;
const MAX_RETRY_MS = 30_000;
// Equal-jitter exponential backoff: schedule at base * 2^n with a
// random factor in [base, 2*base] added. Avoids thundering-herd when
// many clients reconnect at the same instant after a server restart.
function nextDelay(prev: number): number {
  const backoff = Math.min(prev * 1.5, MAX_RETRY_MS);
  const jitter = Math.random() * backoff;
  return Math.round(backoff + jitter);
}

export class WSClient {
  #ws: WebSocket | null = null;
  #reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  #retryDelay = INITIAL_RETRY_MS;
  #destroyed = false;
  #handlers: WSHandlers;
  #roomId: string;

  constructor(roomId: string, handlers: WSHandlers = {}) {
    this.#roomId = roomId;
    this.#handlers = handlers;
  }

  connect() {
    if (this.#destroyed) return;
    if (this.#ws?.readyState === WebSocket.OPEN) return;

    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${protocol}//${location.host}/ws/${this.#roomId}`);

    ws.onopen = () => {
      this.#retryDelay = INITIAL_RETRY_MS;
      this.#handlers.onOpen?.();
    };

    ws.onclose = () => {
      if (this.#destroyed) return;
      this.#handlers.onClose?.();
      const delay = this.#retryDelay;
      this.#retryDelay = nextDelay(this.#retryDelay);
      this.#reconnectTimer = setTimeout(() => this.connect(), delay);
    };

    ws.onerror = (e) => this.#handlers.onError?.(e);

    ws.onmessage = (evt) => {
      try {
        this.#handlers.onMessage?.(JSON.parse(evt.data));
      } catch (e) {
        console.warn('[WSClient] failed to parse message', e);
      }
    };

    this.#ws = ws;
  }

  send(data: Record<string, unknown>) {
    if (this.#ws?.readyState === WebSocket.OPEN) {
      this.#ws.send(JSON.stringify(data));
    }
  }

  disconnect() {
    this.#destroyed = true;
    if (this.#reconnectTimer) clearTimeout(this.#reconnectTimer);
    this.#ws?.close();
    this.#ws = null;
  }

  get isConnected(): boolean {
    return this.#ws?.readyState === WebSocket.OPEN;
  }
}