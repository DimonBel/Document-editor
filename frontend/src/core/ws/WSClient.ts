interface WSHandlers {
  onOpen?: () => void;
  onClose?: () => void;
  onMessage?: (data: Record<string, unknown>) => void;
  onError?: (e: Event) => void;
}

export class WSClient {
  #ws: WebSocket | null = null;
  #reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  #retryDelay = 1000;
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
      this.#retryDelay = 1000;
      this.#handlers.onOpen?.();
    };

    ws.onclose = () => {
      if (this.#destroyed) return;
      this.#handlers.onClose?.();
      this.#reconnectTimer = setTimeout(() => {
        this.#retryDelay = Math.min(this.#retryDelay * 1.5, 30_000);
        this.connect();
      }, this.#retryDelay);
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