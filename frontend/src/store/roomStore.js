import { create } from 'zustand';
import { nanoid } from 'nanoid';

// Stable client identity that persists for the browser tab's lifetime.
const CLIENT_ID   = nanoid(10);
const CLIENT_NAME = `User-${CLIENT_ID.slice(0, 4).toUpperCase()}`;

/**
 * Room store — connection identity and room metadata.
 * Intentionally does NOT hold canvas or collab state; those are
 * separate concerns with separate lifecycles.
 */
export const useRoomStore = create((set) => ({
  // Stable identity — never changes after init
  clientId:   CLIENT_ID,
  clientName: CLIENT_NAME,

  // Mutable room session
  roomId:    null,
  roomName:  null,
  connected: false,

  joinRoom: ({ roomId, roomName }) => set({ roomId, roomName }),
  leaveRoom: () => set({ roomId: null, roomName: null, connected: false }),
  setConnected: (connected) => set({ connected }),
}));
