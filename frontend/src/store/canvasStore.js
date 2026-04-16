import { create } from 'zustand';

/**
 * Canvas store — owns everything related to the drawing surface:
 * active tool, visual properties, committed elements, and the live preview.
 *
 * Kept separate from room/collab so canvas state can survive a room reconnect
 * without being wiped by unrelated events.
 */
export const useCanvasStore = create((set) => ({
  // ── Tool ──────────────────────────────────────────────────────────────
  activeTool: 'freehand',
  setActiveTool: (activeTool) => set({ activeTool }),

  // ── Visual properties ─────────────────────────────────────────────────
  strokeColor: '#1a1a1a',
  fillColor:   'transparent',
  strokeWidth: 4,
  setStrokeColor: (strokeColor) => set({ strokeColor }),
  setFillColor:   (fillColor)   => set({ fillColor }),
  setStrokeWidth: (strokeWidth) => set({ strokeWidth }),

  // ── Committed elements (output of CRDT doc) ───────────────────────────
  elements: [],
  setElements: (elements) => set({ elements }),

  // ── In-progress stroke (optimistic preview before mouse-up) ──────────
  liveElement: null,
  setLiveElement: (liveElement) => set({ liveElement }),

  // ── Clear board helper ────────────────────────────────────────────────
  clearCanvas: () => set({ elements: [], liveElement: null }),
}));
