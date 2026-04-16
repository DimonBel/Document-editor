import { useState, useEffect, useCallback } from 'react';
import { roomsApi } from '../../core/api/roomsApi';
import { RoomInfo } from '../../types';

const POLL_INTERVAL = 6000;

export function useRoomList() {
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchRooms = useCallback(async () => {
    try {
      setRooms(await roomsApi.list());
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch rooms');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchRooms();
    const timer = setInterval(fetchRooms, POLL_INTERVAL);
    return () => clearInterval(timer);
  }, [fetchRooms]);

  const createRoom = useCallback(async (name: string) => {
    const room = await roomsApi.create(name);
    await fetchRooms();
    return room;
  }, [fetchRooms]);

  const getRoom = useCallback((id: string) => roomsApi.get(id), []);

  return { rooms, loading, error, createRoom, getRoom, refresh: fetchRooms };
}