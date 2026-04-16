import { create } from 'zustand';
import { DrawElement, ToolType } from '../types';

interface CanvasState {
  activeTool: ToolType;
  setActiveTool: (tool: ToolType) => void;
  strokeColor: string;
  fillColor: string;
  strokeWidth: number;
  setStrokeColor: (color: string) => void;
  setFillColor: (color: string) => void;
  setStrokeWidth: (width: number) => void;
  elements: DrawElement[];
  setElements: (elements: DrawElement[]) => void;
  liveElement: DrawElement | null;
  setLiveElement: (el: DrawElement | null) => void;
  clearCanvas: () => void;
}

export const useCanvasStore = create<CanvasState>((set) => ({
  activeTool: 'freehand',
  setActiveTool: (activeTool) => set({ activeTool }),
  strokeColor: '#1a1a1a',
  fillColor: 'transparent',
  strokeWidth: 4,
  setStrokeColor: (strokeColor) => set({ strokeColor }),
  setFillColor: (fillColor) => set({ fillColor }),
  setStrokeWidth: (strokeWidth) => set({ strokeWidth }),
  elements: [],
  setElements: (elements) => set({ elements }),
  liveElement: null,
  setLiveElement: (liveElement) => set({ liveElement }),
  clearCanvas: () => set({ elements: [], liveElement: null }),
}));