import { create } from 'zustand';

/**
 * Collaboration store — online users list and live cursor positions.
 * Updated exclusively by the collaboration hook in response to WS events.
 */
export const useCollabStore = create((set) => ({
  users:   [],
  cursors: {},

  // ── User roster ───────────────────────────────────────────────────────
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

  // ── Cursors ───────────────────────────────────────────────────────────
  updateCursor: (clientId, cursor) =>
    set((s) => ({ cursors: { ...s.cursors, [clientId]: cursor } })),

  // ── Reset (on room leave) ─────────────────────────────────────────────
  reset: () => set({ users: [], cursors: {} }),
}));
