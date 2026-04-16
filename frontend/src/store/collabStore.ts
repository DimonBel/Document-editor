import { create } from 'zustand';
import { ClientInfo, CursorPosition, DrawElement } from '../types';

interface CollabState {
  users: ClientInfo[];
  cursors: Record<string, CursorPosition>;
  remotePreviews: Record<string, DrawElement>;
  setUsers: (users: ClientInfo[]) => void;
  addUser: (user: ClientInfo) => void;
  removeUser: (clientId: string) => void;
  updateCursor: (clientId: string, cursor: CursorPosition) => void;
  setRemotePreview: (clientId: string, element: DrawElement | null) => void;
  reset: () => void;
}

export const useCollabStore = create<CollabState>((set) => ({
  users: [],
  cursors: {},
  remotePreviews: {},
  setUsers: (users) => set({ users }),
  addUser: (user) =>
    set((s) => ({
      users: s.users.some((u) => u.id === user.id) ? s.users : [...s.users, user],
    })),
  removeUser: (clientId) =>
    set((s) => {
      const cursors = { ...s.cursors };
      const remotePreviews = { ...s.remotePreviews };
      delete cursors[clientId];
      delete remotePreviews[clientId];
      return { users: s.users.filter((u) => u.id !== clientId), cursors, remotePreviews };
    }),
  updateCursor: (clientId, cursor) =>
    set((s) => ({ cursors: { ...s.cursors, [clientId]: cursor } })),
  setRemotePreview: (clientId, element) =>
    set((s) => {
      if (element === null) {
        const remotePreviews = { ...s.remotePreviews };
        delete remotePreviews[clientId];
        return { remotePreviews };
      }
      return { remotePreviews: { ...s.remotePreviews, [clientId]: element } };
    }),
  reset: () => set({ users: [], cursors: {}, remotePreviews: {} }),
}));