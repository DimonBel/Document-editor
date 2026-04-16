import { useEffect, useRef, useCallback } from 'react';
import { WSClient } from '../../core/ws/WSClient';
import { CRDTDocument } from '../../core/crdt/CRDTDocument';
import { useRoomStore } from '../../store/roomStore';
import { useCanvasStore } from '../../store/canvasStore';
import { useCollabStore } from '../../store/collabStore';
import { DrawElement, Point } from '../../types';

export function useCollaboration() {
  const { roomId, clientId, clientName, setConnected } = useRoomStore();
  const { setElements } = useCanvasStore();
  const { setUsers, addUser, removeUser, updateCursor, reset } = useCollabStore();

  const wsRef = useRef<WSClient | null>(null);
  const crdtRef = useRef<CRDTDocument | null>(null);

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
    };
  }, [roomId, clientId, clientName, setConnected, setElements, setUsers, addUser, removeUser, updateCursor, reset]);

  const addElement = useCallback((type: string, data: Record<string, unknown>) => {
    const crdt = crdtRef.current;
    const ws = wsRef.current;
    if (!crdt || !ws) return;

    const op = crdt.addElement(type, data);
    setElements(crdt.getOrderedElements());
    ws.send({ type: 'operation', operation: op });
  }, [setElements]);

  const sendCursor = useCallback((position: Point) => {
    wsRef.current?.send({
      type: 'cursor',
      clientId,
      position,
      name: clientName,
    });
  }, [clientId, clientName]);

  return { addElement, sendCursor };
}