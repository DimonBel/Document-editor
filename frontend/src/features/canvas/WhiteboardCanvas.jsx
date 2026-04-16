import React, { useRef } from 'react';
import { Stage, Layer } from 'react-konva';
import { useCanvasStore } from '../../store/canvasStore.js';
import { useCollabStore  } from '../../store/collabStore.js';
import { getTool         } from './tools/registry.js';
import { renderElement   } from './renderElement.jsx';
import { useDrawing      } from './useDrawing.js';
import { LiveCursors     } from '../collaboration/LiveCursors.jsx';
import { useWindowSize   } from '../../shared/hooks/useWindowSize.js';
import './WhiteboardCanvas.css';

const HEADER_HEIGHT = 56; // matches --header-height token

/**
 * WhiteboardCanvas — the Konva stage wired to the drawing system.
 *
 * Responsibilities:
 *  - Render committed elements + live preview stroke
 *  - Delegate pointer events to useDrawing (which delegates to the active tool)
 *  - Render peer cursors on a non-interactive overlay layer
 *
 * Props are injected by WhiteboardPage; this component is unaware of WS/CRDT.
 */
export function WhiteboardCanvas({ addElement, sendCursor }) {
  const stageRef  = useRef(null);
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
        {/* Committed + live elements */}
        <Layer>
          {elements.map((el) => renderElement(el))}
          {liveElement && renderElement(liveElement)}
        </Layer>

        {/* Peer cursors — non-interactive overlay */}
        <LiveCursors cursors={cursors} />
      </Stage>
    </div>
  );
}
