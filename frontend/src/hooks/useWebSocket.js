import { useState, useEffect, useRef, useCallback } from 'react';

/**
 * Manages one WebSocket connection with automatic reconnect.
 *
 * @param {string} roomId
 * @param {(data: object) => void} onMessage  Called with parsed JSON payload.
 * @returns {{ connected: boolean, send: (data: object) => void }}
 */
export function useWebSocket(roomId, onMessage) {
  const [connected, setConnected] = useState(false);
  const wsRef = useRef(null);
  const timerRef = useRef(null);
  // Keep a stable ref so reconnect closures always call the latest handler.
  const onMessageRef = useRef(onMessage);
  onMessageRef.current = onMessage;

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${proto}//${window.location.host}/ws/${roomId}`;
    const ws = new WebSocket(url);

    ws.onopen = () => {
      setConnected(true);
      clearTimeout(timerRef.current);
    };

    ws.onmessage = (evt) => {
      try {
        onMessageRef.current(JSON.parse(evt.data));
      } catch (e) {
        console.error('[WS] parse error', e);
      }
    };

    ws.onclose = () => {
      setConnected(false);
      timerRef.current = setTimeout(connect, 3000);
    };

    ws.onerror = (e) => console.error('[WS] error', e);

    wsRef.current = ws;
  }, [roomId]);

  useEffect(() => {
    connect();
    return () => {
      clearTimeout(timerRef.current);
      wsRef.current?.close();
    };
  }, [connect]);

  const send = useCallback((data) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(data));
    }
  }, []);

  return { connected, send };
}
