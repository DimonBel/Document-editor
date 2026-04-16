import { useRef } from 'react';
import { Stage, Layer } from 'react-konva';
import { useCanvasStore } from '../../store/canvasStore';
import { useCollabStore } from '../../store/collabStore';
import { getTool } from './tools/registry';
import { renderElement } from './renderElement';
import { useDrawing } from './useDrawing';
import { LiveCursors } from '../collaboration/LiveCursors';
import { useWindowSize } from '../../shared/hooks/useWindowSize';
import { Point } from '../../types';
import Konva from 'konva';
import './WhiteboardCanvas.css';

const HEADER_HEIGHT = 56;

interface WhiteboardCanvasProps {
  addElement: (type: string, data: Record<string, unknown>) => void;
  sendCursor: (pos: Point) => void;
}

export function WhiteboardCanvas({ addElement, sendCursor }: WhiteboardCanvasProps) {
  const stageRef = useRef<Konva.Stage | null>(null);
  const { width, height } = useWindowSize();

  const { elements, liveElement, activeTool } = useCanvasStore();
  const { cursors } = useCollabStore();

  const { onPointerDown, onPointerMove, onPointerUp } = useDrawing(stageRef, {
    addElement,
    sendCursor,
  });

  const cursor = getTool(activeTool).cursor;

  return (
    <div className="canvas-root" style={{ cursor }}>
      <Stage
        ref={stageRef}
        width={width}
        height={height - HEADER_HEIGHT}
        onMouseDown={onPointerDown}
        onMouseMove={onPointerMove}
        onMouseUp={onPointerUp}
        onTouchStart={onPointerDown}
        onTouchMove={onPointerMove}
        onTouchEnd={onPointerUp}
      >
        <Layer>
          {elements.map((el) => renderElement(el))}
          {liveElement && renderElement(liveElement)}
        </Layer>

        <LiveCursors cursors={cursors} />
      </Stage>
    </div>
  );
}