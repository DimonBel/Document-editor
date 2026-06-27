import { useEffect, useRef, useCallback } from 'react';
import { WSClient } from '../../core/ws/WSClient';
import { CRDTDocument } from '../../core/crdt/CRDTDocument';
import { useRoomStore } from '../../store/roomStore';
import { useCanvasStore } from '../../store/canvasStore';
import { useCollabStore } from '../../store/collabStore';
import { DrawElement, Point, Operation, DraftResult } from '../../types';

const DEBOUNCE_MS = 50;

// High-frequency streams (cursors, ghost previews) are throttled so a
// 5-client room at 60 fps doesn't blast 300 frames/sec at the server.
// Both are leading-edge: we send the first sample immediately so the
// remote peer sees the pointer move right away, then suppress samples
// until the throttle window expires.
const CURSOR_THROTTLE_MS = 40;
const PREVIEW_THROTTLE_MS = 40;

export function useCollaboration() {
  const { roomId, clientId, clientName, setConnected } = useRoomStore();
  const { setElements } = useCanvasStore();
  const { setUsers, addUser, removeUser, updateCursor, reset, setRemotePreview } = useCollabStore();

  const wsRef = useRef<WSClient | null>(null);
  const crdtRef = useRef<CRDTDocument | null>(null);
  const pendingOpsRef = useRef<Operation[]>([]);
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const lastCursorSentRef = useRef<number>(0);
  const pendingCursorRef = useRef<Point | null>(null);
  const cursorTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const lastPreviewSentRef = useRef<number>(0);
  const pendingPreviewRef = useRef<DraftResult | null | undefined>(undefined);
  const previewTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const flushPending = useCallback(() => {
    const ws = wsRef.current;
    if (!ws || pendingOpsRef.current.length === 0) return;

    const ops = pendingOpsRef.current;
    pendingOpsRef.current = [];

    for (const op of ops) {
      ws.send({ type: 'operation', operation: op });
    }
  }, []);

  const scheduleFlush = useCallback(() => {
    if (flushTimerRef.current) return;
    flushTimerRef.current = setTimeout(() => {
      flushTimerRef.current = null;
      flushPending();
    }, DEBOUNCE_MS);
  }, [flushPending]);

  useEffect(() => {
    if (!roomId) return;

    const crdt = new CRDTDocument(clientId);
    crdtRef.current = crdt;

    const ws = new WSClient(roomId, {
      onOpen() {
        setConnected(true);
        ws.send({ type: 'join', clientId, name: clientName });
      },
      onClose() {
        setConnected(false);
      },
      onMessage(data) {
        switch (data.type) {
          case 'sync': {
            crdt.sync((data.elements ?? []) as DrawElement[]);
            setElements(crdt.getOrderedElements());
            setUsers((data.clients ?? []) as never[]);
            break;
          }
          case 'operation': {
            if (data.senderId !== clientId) {
              crdt.applyRemote(data.operation as never);
              setElements(crdt.getOrderedElements());
            }
            break;
          }
          case 'preview': {
            if (data.senderId !== clientId) {
              const element = data.element as DrawElement;
              setRemotePreview(data.senderId as string, element);
            }
            break;
          }
          case 'preview_clear': {
            if (data.senderId !== clientId) {
              setRemotePreview(data.senderId as string, null);
            }
            break;
          }
          case 'cursor': {
            if (data.clientId !== clientId) {
              updateCursor(data.clientId as string, { ...(data.position as Point), name: data.name as string | undefined });
            }
            break;
          }
          case 'user_joined': {
            addUser(data.client as never);
            break;
          }
          case 'user_left': {
            removeUser(data.clientId as string);
            break;
          }
        }
      },
    });

    ws.connect();
    wsRef.current = ws;

    return () => {
      ws.disconnect();
      reset();
      setConnected(false);
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current);
      if (cursorTimerRef.current) clearTimeout(cursorTimerRef.current);
      if (previewTimerRef.current) clearTimeout(previewTimerRef.current);
      pendingOpsRef.current = [];
      pendingCursorRef.current = null;
      pendingPreviewRef.current = undefined;
    };
  }, [roomId, clientId, clientName, setConnected, setElements, setUsers, addUser, removeUser, updateCursor, reset, flushPending, setRemotePreview]);

  const addElement = useCallback((type: string, data: Record<string, unknown>) => {
    const crdt = crdtRef.current;
    if (!crdt) return;

    const op = crdt.addElement(type, data);
    setElements(crdt.getOrderedElements());

    pendingOpsRef.current.push(op);
    scheduleFlush();
  }, [setElements, scheduleFlush]);

  const sendCursor = useCallback((position: Point) => {
    const ws = wsRef.current;
    if (!ws) return;
    const now = performance.now();
    const elapsed = now - lastCursorSentRef.current;
    if (elapsed >= CURSOR_THROTTLE_MS) {
      ws.send({ type: 'cursor', clientId, position, name: clientName });
      lastCursorSentRef.current = now;
      pendingCursorRef.current = null;
      if (cursorTimerRef.current) {
        clearTimeout(cursorTimerRef.current);
        cursorTimerRef.current = null;
      }
    } else {
      pendingCursorRef.current = position;
      if (!cursorTimerRef.current) {
        const delay = CURSOR_THROTTLE_MS - elapsed;
        cursorTimerRef.current = setTimeout(() => {
          cursorTimerRef.current = null;
          if (pendingCursorRef.current) {
            const pos = pendingCursorRef.current;
            pendingCursorRef.current = null;
            ws.send({ type: 'cursor', clientId, position: pos, name: clientName });
            lastCursorSentRef.current = performance.now();
          }
        }, delay);
      }
    }
  }, [clientId, clientName]);

  const sendPreview = useCallback((draft: DraftResult | null) => {
    const ws = wsRef.current;
    if (!ws) return;
    // null means "clear"; always send immediately so peers stop showing
    // a stale ghost as soon as the local user releases the pointer.
    if (draft === null) {
      pendingPreviewRef.current = null;
      if (previewTimerRef.current) {
        clearTimeout(previewTimerRef.current);
        previewTimerRef.current = null;
      }
      ws.send({ type: 'previewClear', senderId: clientId });
      return;
    }
    const now = performance.now();
    const elapsed = now - lastPreviewSentRef.current;
    if (elapsed >= PREVIEW_THROTTLE_MS) {
      ws.send({ type: 'preview', senderId: clientId, element: { ...draft, id: '' } });
      lastPreviewSentRef.current = now;
      pendingPreviewRef.current = undefined;
      if (previewTimerRef.current) {
        clearTimeout(previewTimerRef.current);
        previewTimerRef.current = null;
      }
    } else {
      pendingPreviewRef.current = draft;
      if (!previewTimerRef.current) {
        const delay = PREVIEW_THROTTLE_MS - elapsed;
        previewTimerRef.current = setTimeout(() => {
          previewTimerRef.current = null;
          const d = pendingPreviewRef.current;
          pendingPreviewRef.current = undefined;
          if (d) {
            ws.send({ type: 'preview', senderId: clientId, element: { ...d, id: '' } });
            lastPreviewSentRef.current = performance.now();
          }
        }, delay);
      }
    }
  }, [clientId]);

  return { addElement, sendCursor, sendPreview };
}