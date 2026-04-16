import { useRef, useCallback } from 'react';
import { CRDTDocument } from '../crdt/client-state.js';

/**
 * Exposes a stable CRDTDocument instance bound to `clientId`.
 * All returned functions are stable across renders.
 */
export function useCRDT(clientId) {
  const docRef = useRef(null);
  if (!docRef.current) {
    docRef.current = new CRDTDocument(clientId);
  }

  const addElement = useCallback(
    (type, data) => docRef.current.addElement(type, data),
    [],
  );

  const deleteElement = useCallback(
    (id) => docRef.current.deleteElement(id),
    [],
  );

  const updateElement = useCallback(
    (id, updates) => docRef.current.updateElement(id, updates),
    [],
  );

  const applyRemote = useCallback(
    (op) => docRef.current.applyRemote(op),
    [],
  );

  const syncFromServer = useCallback(
    (elements) => docRef.current.sync(elements),
    [],
  );

  const getElements = useCallback(
    () => docRef.current.getOrderedElements(),
    [],
  );

  return { addElement, deleteElement, updateElement, applyRemote, syncFromServer, getElements };
}
