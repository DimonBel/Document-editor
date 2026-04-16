import { create } from 'zustand';
import { nanoid } from 'nanoid';

const CLIENT_ID = nanoid(10);
const CLIENT_NAME = `User-${CLIENT_ID.slice(0, 4).toUpperCase()}`;

interface DocState {
  clientId: string;
  clientName: string;
  docId: string | null;
  docTitle: string | null;
  content: string;
  version: number;
  connected: boolean;
  joinDoc: (docId: string, docTitle: string) => void;
  leaveDoc: () => void;
  setContent: (content: string) => void;
  setVersion: (version: number) => void;
  setConnected: (connected: boolean) => void;
}

export const useDocStore = create<DocState>((set) => ({
  clientId: CLIENT_ID,
  clientName: CLIENT_NAME,
  docId: null,
  docTitle: null,
  content: '',
  version: 0,
  connected: false,
  joinDoc: (docId, docTitle) => set({ docId, docTitle }),
  leaveDoc: () => set({ docId: null, docTitle: null, content: '', version: 0, connected: false }),
  setContent: (content) => set({ content }),
  setVersion: (version) => set({ version }),
  setConnected: (connected) => set({ connected }),
}));