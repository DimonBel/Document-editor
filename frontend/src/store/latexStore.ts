import { create } from 'zustand';
import { nanoid } from 'nanoid';
import { DrawElement, ToolType } from '../types';

const CLIENT_ID = nanoid(10);
const CLIENT_NAME = `User-${CLIENT_ID.slice(0, 4).toUpperCase()}`;

export interface CommitEntry {
  id: string;
  author: string;
  timestamp: number;
  latexContent: string;
  annotationCount: number;
}

interface RemoteUser {
  name: string;
  color: string;
}

interface LatexState {
  clientId: string;
  clientName: string;
  roomId: string | null;
  roomName: string | null;
  connected: boolean;
  elements: DrawElement[];
  liveElement: DrawElement | null;
  remoteUsers: Record<string, RemoteUser>;
  commits: CommitEntry[];
  activeTool: ToolType;
  strokeColor: string;
  fillColor: string;
  strokeWidth: number;
  joinLatexRoom: (roomId: string, roomName: string) => void;
  leaveLatexRoom: () => void;
  setConnected: (connected: boolean) => void;
  setElements: (elements: DrawElement[]) => void;
  setLiveElement: (el: DrawElement | null) => void;
  setRemoteUsers: (users: Record<string, RemoteUser>) => void;
  addRemoteUser: (id: string, name: string, color: string) => void;
  removeRemoteUser: (id: string) => void;
  addCommit: (author: string, latexContent: string, annotationCount: number) => void;
  setActiveTool: (tool: ToolType) => void;
  setStrokeColor: (color: string) => void;
  setFillColor: (color: string) => void;
  setStrokeWidth: (width: number) => void;
  clearAnnotations: () => void;
}

export const useLatexStore = create<LatexState>((set) => ({
  clientId: CLIENT_ID,
  clientName: CLIENT_NAME,
  roomId: null,
  roomName: null,
  connected: false,
  elements: [],
  liveElement: null,
  remoteUsers: {},
  commits: [],
  activeTool: 'freehand',
  strokeColor: '#1a1a1a',
  fillColor: 'transparent',
  strokeWidth: 4,

  joinLatexRoom: (roomId, roomName) => set({ roomId, roomName }),
  leaveLatexRoom: () => set({ roomId: null, roomName: null, connected: false, elements: [], liveElement: null, remoteUsers: {} }),
  setConnected: (connected) => set({ connected }),
  setElements: (elements) => set({ elements }),
  setLiveElement: (liveElement) => set({ liveElement }),
  setRemoteUsers: (users) => set({ remoteUsers: users }),
  addRemoteUser: (id, name, color) => set((state) => ({
    remoteUsers: { ...state.remoteUsers, [id]: { name, color } }
  })),
  removeRemoteUser: (id) => set((state) => {
    const next = { ...state.remoteUsers };
    delete next[id];
    return { remoteUsers: next };
  }),
  addCommit: (author, latexContent, annotationCount) => set((state) => ({
    commits: [...state.commits, {
      id: nanoid(8),
      author,
      timestamp: Date.now(),
      latexContent,
      annotationCount,
    }]
  })),
  setActiveTool: (activeTool) => set({ activeTool }),
  setStrokeColor: (strokeColor) => set({ strokeColor }),
  setFillColor: (fillColor) => set({ fillColor }),
  setStrokeWidth: (strokeWidth) => set({ strokeWidth }),
  clearAnnotations: () => set({ elements: [], liveElement: null }),
}));