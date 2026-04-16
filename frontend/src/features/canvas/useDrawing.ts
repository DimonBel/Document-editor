import { RefObject } from 'react';
import { useRef, useCallback } from 'react';
import { useCanvasStore } from '../../store/canvasStore';
import { getTool } from './tools/registry';
import { DraftResult, Point, DrawElement } from '../../types';
import Konva from 'konva';

interface UseDrawingProps {
  addElement: (type: string, data: Record<string, unknown>) => void;
  sendCursor: (pos: Point) => void;
}

export function useDrawing(
  stageRef: RefObject<Konva.Stage | null>,
  { addElement, sendCursor }: UseDrawingProps
) {
  const draftRef = useRef<DraftResult | null>(null);

  const { activeTool, strokeColor, fillColor, strokeWidth, setLiveElement } =
    useCanvasStore();

  const getPointerPos = (): Point =>
    stageRef.current?.getPointerPosition() ?? { x: 0, y: 0 };

  const onPointerDown = useCallback(() => {
    const pos = getPointerPos();
    const tool = getTool(activeTool);

    const draft = tool.onStart({ pos, strokeColor, fillColor, strokeWidth, addElement });
    draftRef.current = draft;
    if (draft) setLiveElement({ ...draft, id: '__live__' } as DrawElement);
  }, [activeTool, strokeColor, fillColor, strokeWidth, addElement, setLiveElement]);

  const onPointerMove = useCallback(() => {
    const pos = getPointerPos();
    sendCursor(pos);

    if (!draftRef.current) return;
    const tool = getTool(activeTool);
    const updated = tool.onMove({ draft: draftRef.current, pos });
    draftRef.current = updated;
    setLiveElement(updated ? { ...updated, id: '__live__' } as DrawElement : null);
  }, [activeTool, sendCursor, setLiveElement]);

  const onPointerUp = useCallback(() => {
    if (!draftRef.current) return;

    const tool = getTool(activeTool);
    const result = tool.onCommit({ draft: draftRef.current, addElement });

    if (result) {
      const { type, ...data } = result;
      addElement(type, data);
    }

    draftRef.current = null;
    setLiveElement(null);
  }, [activeTool, addElement, setLiveElement]);

  return { onPointerDown, onPointerMove, onPointerUp };
}