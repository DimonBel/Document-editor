import { create } from 'zustand';
import { ClientInfo, CursorPosition } from '../types';

interface CollabState {
  users: ClientInfo[];
  cursors: Record<string, CursorPosition>;
  setUsers: (users: ClientInfo[]) => void;
  addUser: (user: ClientInfo) => void;
  removeUser: (clientId: string) => void;
  updateCursor: (clientId: string, cursor: CursorPosition) => void;
  reset: () => void;
}

export const useCollabStore = create<CollabState>((set) => ({
  users: [],
  cursors: {},
  setUsers: (users) => set({ users }),
  addUser: (user) =>
    set((s) => ({
      users: s.users.some((u) => u.id === user.id) ? s.users : [...s.users, user],
    })),
  removeUser: (clientId) =>
    set((s) => {
      const cursors = { ...s.cursors };
      delete cursors[clientId];
      return { users: s.users.filter((u) => u.id !== clientId), cursors };
    }),
  updateCursor: (clientId, cursor) =>
    set((s) => ({ cursors: { ...s.cursors, [clientId]: cursor } })),
  reset: () => set({ users: [], cursors: {} }),
}));