import React, { useState, useRef, useCallback, useEffect } from 'react';
import { Stage, Layer } from 'react-konva';
import { Button, Tooltip, Badge } from 'antd';
import { LogoutOutlined, WifiOutlined, DisconnectOutlined } from '@ant-design/icons';
import Toolbar from './Toolbar.jsx';
import LiveCursors from './LiveCursors.jsx';
import UsersList from './UsersList.jsx';
import { useWhiteboard } from '../hooks/useWhiteboard.js';
import { renderElement } from '../utils/shapes.jsx';
import './Whiteboard.css';

/**
 * Main whiteboard canvas.  Handles pointer events, translates them into
 * CRDT operations, and delegates network/state to useWhiteboard.
 */
export default function Whiteboard({ roomId, roomName, clientId, clientName, onLeave }) {
  const [tool, setTool] = useState('freehand');
  const [color, setColor] = useState('#000000');
  const [strokeWidth, setStrokeWidth] = useState(4);
  const [fillColor, setFillColor] = useState('transparent');
  const [liveEl, setLiveEl] = useState(null); // optimistic preview before mouse-up

  const stageRef = useRef(null);
  const drawingRef = useRef(null);  // mutable in-progress data (not state)
  const startPtRef = useRef(null);

  const { elements, cursors, users, connected, addElement, sendCursor } =
    useWhiteboard(roomId, clientId, clientName);

  // -------------------------------------------------------------------------
  // Keyboard shortcuts
  // -------------------------------------------------------------------------
  useEffect(() => {
    const keys = { f: 'freehand', r: 'rectangle', e: 'ellipse', t: 'text', x: 'eraser' };
    const handler = (ev) => {
      if (ev.target.tagName === 'INPUT' || ev.target.tagName === 'TEXTAREA') return;
      const next = keys[ev.key.toLowerCase()];
      if (next) setTool(next);
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  // -------------------------------------------------------------------------
  // Pointer helpers
  // -------------------------------------------------------------------------
  const getPointer = () => stageRef.current?.getPointerPosition() ?? { x: 0, y: 0 };

  const handleMouseDown = useCallback(
    (e) => {
      const pos = getPointer();

      if (tool === 'text') {
        const text = window.prompt('Enter text:');
        if (text?.trim()) {
          addElement('Text', { x: pos.x, y: pos.y, content: text.trim(), fontSize: 18, color });
        }
        return;
      }

      if (tool === 'freehand' || tool === 'eraser') {
        drawingRef.current = { tool, points: [pos.x, pos.y], color, width: strokeWidth };
      } else {
        startPtRef.current = pos;
        drawingRef.current = { tool, x: pos.x, y: pos.y, w: 0, h: 0, color, fill: fillColor };
      }

      setLiveEl({ ...drawingRef.current, id: '__live__' });
    },
    [tool, color, strokeWidth, fillColor, addElement],
  );

  const handleMouseMove = useCallback(
    (e) => {
      const pos = getPointer();
      sendCursor(pos);

      if (!drawingRef.current) return;

      if (tool === 'freehand' || tool === 'eraser') {
        drawingRef.current.points = [...drawingRef.current.points, pos.x, pos.y];
      } else {
        const s = startPtRef.current;
        drawingRef.current = {
          ...drawingRef.current,
          x: Math.min(pos.x, s.x),
          y: Math.min(pos.y, s.y),
          w: Math.abs(pos.x - s.x),
          h: Math.abs(pos.y - s.y),
        };
      }

      setLiveEl({ ...drawingRef.current, id: '__live__' });
    },
    [tool, sendCursor],
  );

  const handleMouseUp = useCallback(() => {
    const d = drawingRef.current;
    if (!d) return;

    if (d.tool === 'freehand' && d.points.length >= 4) {
      addElement('Freehand', { points: d.points, color: d.color, width: d.width });
    } else if (d.tool === 'eraser' && d.points.length >= 4) {
      addElement('Eraser', { points: d.points, width: d.width * 2 });
    } else if (d.tool === 'rectangle' && (d.w > 2 || d.h > 2)) {
      addElement('Rectangle', { x: d.x, y: d.y, w: d.w, h: d.h, color: d.color, fill: d.fill });
    } else if (d.tool === 'ellipse' && (d.w > 2 || d.h > 2)) {
      addElement('Ellipse', {
        x: d.x + d.w / 2,
        y: d.y + d.h / 2,
        rx: d.w / 2,
        ry: d.h / 2,
        color: d.color,
        fill: d.fill,
      });
    }

    drawingRef.current = null;
    startPtRef.current = null;
    setLiveEl(null);
  }, [addElement]);

  // -------------------------------------------------------------------------
  // Cursor style
  // -------------------------------------------------------------------------
  const cursorStyle = tool === 'eraser' ? 'cell' : tool === 'text' ? 'text' : 'crosshair';

  return (
    <div className="wb-wrapper">
      {/* Header bar */}
      <header className="wb-header">
        <span className="wb-room-name">{roomName}</span>
        <span className={`wb-status ${connected ? 'ok' : 'warn'}`}>
          {connected ? <WifiOutlined /> : <DisconnectOutlined />}
          {connected ? ' Connected' : ' Reconnecting…'}
        </span>
        <Tooltip title="Leave room">
          <Button icon={<LogoutOutlined />} onClick={onLeave} />
        </Tooltip>
      </header>

      {/* Drawing tools */}
      <Toolbar
        tool={tool} setTool={setTool}
        color={color} setColor={setColor}
        strokeWidth={strokeWidth} setStrokeWidth={setStrokeWidth}
        fillColor={fillColor} setFillColor={setFillColor}
      />

      {/* Online users */}
      <UsersList users={users} currentUserId={clientId} />

      {/* Canvas */}
      <Stage
        ref={stageRef}
        width={window.innerWidth}
        height={window.innerHeight - 56}
        className="wb-stage"
        style={{ cursor: cursorStyle, marginTop: 56 }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onTouchStart={handleMouseDown}
        onTouchMove={handleMouseMove}
        onTouchEnd={handleMouseUp}
      >
        {/* Committed elements */}
        <Layer>
          {elements.map((el) => renderElement(el))}
          {/* Optimistic preview of the in-progress stroke */}
          {liveEl && renderElement(liveEl)}
        </Layer>

        {/* Peer cursors — non-interactive overlay */}
        <LiveCursors cursors={cursors} />
      </Stage>
    </div>
  );
}
