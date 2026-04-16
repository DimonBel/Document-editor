/**
 * WSClient — framework-agnostic WebSocket wrapper.
 *
 * Handles:
 *  - Automatic reconnection with exponential back-off (capped at 30 s)
 *  - JSON serialisation / deserialisation
 *  - Typed handler callbacks so callers never touch raw MessageEvent
 *
 * Usage:
 *   const client = new WSClient(roomId, { onOpen, onClose, onMessage, onError });
 *   client.connect();
 *   client.send({ type: 'join', ... });
 *   client.disconnect();   // call on cleanup
 */
export class WSClient {
  #ws            = null;
  #reconnectTimer = null;
  #retryDelay    = 1000;
  #destroyed     = false;

  /** @type {{ onOpen?:()=>void, onClose?:()=>void, onMessage?:(d:object)=>void, onError?:(e:Event)=>void }} */
  #handlers;
  #roomId;

  constructor(roomId, handlers = {}) {
    this.#roomId   = roomId;
    this.#handlers = handlers;
  }

  connect() {
    if (this.#destroyed) return;
    if (this.#ws?.readyState === WebSocket.OPEN) return;

    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws    = new WebSocket(`${proto}//${location.host}/ws/${this.#roomId}`);

    ws.onopen = () => {
      this.#retryDelay = 1000; // reset back-off on success
      this.#handlers.onOpen?.();
    };

    ws.onclose = () => {
      if (this.#destroyed) return;
      this.#handlers.onClose?.();
      // Exponential back-off, max 30 s
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

  send(data) {
    if (this.#ws?.readyState === WebSocket.OPEN) {
      this.#ws.send(JSON.stringify(data));
    }
  }

  disconnect() {
    this.#destroyed = true;
    clearTimeout(this.#reconnectTimer);
    this.#ws?.close();
    this.#ws = null;
  }

  get isConnected() {
    return this.#ws?.readyState === WebSocket.OPEN;
  }
}
