import { useState, useCallback, useEffect } from 'react';
import { useWebSocket } from './useWebSocket.js';
import { useCRDT } from './useCRDT.js';

/**
 * Orchestrates the WebSocket connection and CRDT state for one whiteboard room.
 */
export function useWhiteboard(roomId, clientId, clientName) {
  const [elements, setElements] = useState([]);
  const [cursors, setCursors] = useState({});
  const [users, setUsers] = useState([]);

  const crdt = useCRDT(clientId);

  // -------------------------------------------------------------------------
  // Inbound message handler
  // -------------------------------------------------------------------------

  const handleMessage = useCallback(
    (data) => {
      switch (data.type) {
        case 'sync': {
          crdt.syncFromServer(data.elements ?? []);
          setElements(crdt.getElements());
          setUsers(data.clients ?? []);
          break;
        }
        case 'operation': {
          if (data.senderId !== clientId) {
            crdt.applyRemote(data.operation);
            setElements(crdt.getElements());
          }
          break;
        }
        case 'cursor': {
          if (data.clientId !== clientId) {
            setCursors((prev) => ({
              ...prev,
              [data.clientId]: { ...data.position, name: data.name },
            }));
          }
          break;
        }
        case 'user_joined': {
          setUsers((prev) => {
            if (prev.some((u) => u.id === data.client.id)) return prev;
            return [...prev, data.client];
          });
          break;
        }
        case 'user_left': {
          setUsers((prev) => prev.filter((u) => u.id !== data.clientId));
          setCursors((prev) => {
            const next = { ...prev };
            delete next[data.clientId];
            return next;
          });
          break;
        }
        default:
          break;
      }
    },
    [clientId, crdt],
  );

  const { connected, send } = useWebSocket(roomId, handleMessage);

  // Send join message once connected.
  useEffect(() => {
    if (connected) {
      send({ type: 'join', clientId, name: clientName });
    }
  }, [connected]); // eslint-disable-line react-hooks/exhaustive-deps

  // -------------------------------------------------------------------------
  // Outbound helpers
  // -------------------------------------------------------------------------

  const addElement = useCallback(
    (type, data) => {
      const op = crdt.addElement(type, data);
      setElements(crdt.getElements());
      send({ type: 'operation', operation: op });
      return op;
    },
    [crdt, send],
  );

  const deleteElement = useCallback(
    (id) => {
      const op = crdt.deleteElement(id);
      setElements(crdt.getElements());
      send({ type: 'operation', operation: op });
      return op;
    },
    [crdt, send],
  );

  const sendCursor = useCallback(
    (position) => {
      send({ type: 'cursor', clientId, position, name: clientName });
    },
    [clientId, clientName, send],
  );

  return {
    elements,
    cursors,
    users,
    connected,
    addElement,
    deleteElement,
    sendCursor,
  };
}
