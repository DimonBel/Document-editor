import { useEffect, useRef, useCallback } from 'react';
import { WSClient } from '../../core/ws/WSClient';
import { CRDTDocument } from '../../core/crdt/CRDTDocument';
import { useRoomStore } from '../../store/roomStore';
import { useCanvasStore } from '../../store/canvasStore';
import { useCollabStore } from '../../store/collabStore';
import { DrawElement, Point, Operation, DraftResult } from '../../types';

const DEBOUNCE_MS = 50;

export function useCollaboration() {
  const { roomId, clientId, clientName, setConnected } = useRoomStore();
  const { setElements, setLiveElement } = useCanvasStore();
  const { setUsers, addUser, removeUser, updateCursor, reset } = useCollabStore();

  const wsRef = useRef<WSClient | null>(null);
  const crdtRef = useRef<CRDTDocument | null>(null);
  const pendingOpsRef = useRef<Operation[]>([]);
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

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
              const preview: DrawElement = { ...element, id: `preview_${data.senderId}` };
              setLiveElement(preview);
            }
            break;
          }
          case 'preview_clear': {
            if (data.senderId !== clientId) {
              setLiveElement(null);
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
      pendingOpsRef.current = [];
    };
  }, [roomId, clientId, clientName, setConnected, setElements, setUsers, addUser, removeUser, updateCursor, reset, flushPending, setLiveElement]);

  const addElement = useCallback((type: string, data: Record<string, unknown>) => {
    const crdt = crdtRef.current;
    if (!crdt) return;

    const op = crdt.addElement(type, data);
    setElements(crdt.getOrderedElements());

    pendingOpsRef.current.push(op);
    scheduleFlush();
  }, [setElements, scheduleFlush]);

  const sendPreview = useCallback((draft: DraftResult | null) => {
    wsRef.current?.send({
      type: draft ? 'preview' : 'preview_clear',
      senderId: clientId,
      element: draft ? { ...draft, id: '' } : null,
    });
  }, [clientId]);

  const sendCursor = useCallback((position: Point) => {
    wsRef.current?.send({
      type: 'cursor',
      clientId,
      position,
      name: clientName,
    });
  }, [clientId, clientName]);

  return { addElement, sendCursor, sendPreview };
}