import { useRef, useCallback } from 'react';
import { useCanvasStore } from '../../store/canvasStore.js';
import { getTool } from './tools/registry.js';

/**
 * useDrawing — connects pointer events to the active tool strategy.
 *
 * Returns three stable callbacks (down/move/up) to wire into the Konva Stage.
 * All canvas state reads/writes go through `useCanvasStore`.
 *
 * The `addElement` and `sendCursor` props are injected from outside
 * (the collaboration hook owns the WS + CRDT logic).
 */
export function useDrawing(stageRef, { addElement, sendCursor }) {
  const draftRef = useRef(null); // mutable in-progress data (avoids re-renders)

  const { activeTool, strokeColor, fillColor, strokeWidth, setLiveElement } =
    useCanvasStore();

  const getPointerPos = () =>
    stageRef.current?.getPointerPosition() ?? { x: 0, y: 0 };

  const onPointerDown = useCallback(() => {
    const pos  = getPointerPos();
    const tool = getTool(activeTool);

    const draft = tool.onStart({ pos, strokeColor, fillColor, strokeWidth, addElement });
    draftRef.current = draft;
    if (draft) setLiveElement({ ...draft, id: '__live__' });
  }, [activeTool, strokeColor, fillColor, strokeWidth, addElement, setLiveElement]);

  const onPointerMove = useCallback(() => {
    const pos = getPointerPos();

    // Always broadcast cursor so peers see us moving even without drawing
    sendCursor(pos);

    if (!draftRef.current) return;
    const tool    = getTool(activeTool);
    const updated = tool.onMove({ draft: draftRef.current, pos });
    draftRef.current = updated;
    setLiveElement(updated ? { ...updated, id: '__live__' } : null);
  }, [activeTool, sendCursor, setLiveElement]);

  const onPointerUp = useCallback(() => {
    if (!draftRef.current) return;

    const tool   = getTool(activeTool);
    const result = tool.onCommit({ draft: draftRef.current, addElement });

    if (result) {
      addElement(result.type, result.data);
    }

    draftRef.current = null;
    setLiveElement(null);
  }, [activeTool, addElement, setLiveElement]);

  return { onPointerDown, onPointerMove, onPointerUp };
}
