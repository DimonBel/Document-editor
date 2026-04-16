import { create } from 'zustand';
import { nanoid } from 'nanoid';

const CLIENT_ID = nanoid(10);
const CLIENT_NAME = `User-${CLIENT_ID.slice(0, 4).toUpperCase()}`;

interface RoomState {
  clientId: string;
  clientName: string;
  roomId: string | null;
  roomName: string | null;
  connected: boolean;
  joinRoom: (room: { roomId: string; roomName: string }) => void;
  leaveRoom: () => void;
  setConnected: (connected: boolean) => void;
}

export const useRoomStore = create<RoomState>((set) => ({
  clientId: CLIENT_ID,
  clientName: CLIENT_NAME,
  roomId: null,
  roomName: null,
  connected: false,
  joinRoom: ({ roomId, roomName }) => set({ roomId, roomName }),
  leaveRoom: () => set({ roomId: null, roomName: null, connected: false }),
  setConnected: (connected) => set({ connected }),
}));