import { useState, useEffect, useCallback } from 'react';
import { roomsApi } from '../../core/api/roomsApi.js';

const POLL_INTERVAL = 6000;

/**
 * useRoomList — polls the rooms list and exposes create / join helpers.
 */
export function useRoomList() {
  const [rooms,   setRooms]   = useState([]);
  const [loading, setLoading] = useState(true);
  const [error,   setError]   = useState(null);

  const fetchRooms = useCallback(async () => {
    try {
      setRooms(await roomsApi.list());
      setError(null);
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchRooms();
    const timer = setInterval(fetchRooms, POLL_INTERVAL);
    return () => clearInterval(timer);
  }, [fetchRooms]);

  const createRoom = useCallback(async (name) => {
    const room = await roomsApi.create(name);
    await fetchRooms();
    return room;
  }, [fetchRooms]);

  const getRoom = useCallback((id) => roomsApi.get(id), []);

  return { rooms, loading, error, createRoom, getRoom, refresh: fetchRooms };
}
