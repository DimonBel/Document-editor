import { useEffect, useRef, useCallback } from 'react';
import { WSClient      } from '../../core/ws/WSClient.js';
import { CRDTDocument  } from '../../core/crdt/CRDTDocument.js';
import { useRoomStore  } from '../../store/roomStore.js';
import { useCanvasStore } from '../../store/canvasStore.js';
import { useCollabStore } from '../../store/collabStore.js';

/**
 * useCollaboration — the single integration point between the WS/CRDT layer
 * and the Zustand stores.
 *
 * Returns `addElement` and `sendCursor` — stable references the canvas feature
 * calls without knowing anything about networking.
 */
export function useCollaboration() {
  const { roomId, clientId, clientName, setConnected } = useRoomStore();
  const { setElements } = useCanvasStore();
  const { setUsers, addUser, removeUser, updateCursor, reset } = useCollabStore();

  const wsRef   = useRef(null);
  const crdtRef = useRef(null);

  // ── Bootstrap on room change ───────────────────────────────────────────
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
            crdt.sync(data.elements ?? []);
            setElements(crdt.getOrderedElements());
            setUsers(data.clients ?? []);
            break;
          }
          case 'operation': {
            if (data.senderId !== clientId) {
              crdt.applyRemote(data.operation);
              setElements(crdt.getOrderedElements());
            }
            break;
          }
          case 'cursor': {
            if (data.clientId !== clientId) {
              updateCursor(data.clientId, { ...data.position, name: data.name });
            }
            break;
          }
          case 'user_joined': {
            addUser(data.client);
            break;
          }
          case 'user_left': {
            removeUser(data.clientId);
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
  }, [roomId]); // eslint-disable-line react-hooks/exhaustive-deps

  // ── Stable outbound actions ────────────────────────────────────────────

  const addElement = useCallback((type, data) => {
    const crdt = crdtRef.current;
    const ws   = wsRef.current;
    if (!crdt || !ws) return;

    const op = crdt.addElement(type, data);
    setElements(crdt.getOrderedElements());
    ws.send({ type: 'operation', operation: op });
  }, [setElements]);

  const sendCursor = useCallback((position) => {
    wsRef.current?.send({
      type: 'cursor',
      clientId,
      position,
      name: clientName,
    });
  }, [clientId, clientName]);

  return { addElement, sendCursor };
}
