import React, { useEffect } from 'react';
import { Tooltip, Slider, ColorPicker, Divider, Button } from 'antd';
import {
  EditOutlined,
  BorderOutlined,
  RadiusUprightOutlined,
  FontSizeOutlined,
  ClearOutlined,
} from '@ant-design/icons';
import { useCanvasStore } from '../../store/canvasStore.js';
import { KEYBOARD_MAP  } from '../canvas/tools/registry.js';
import './Toolbar.css';

const TOOLS = [
  { key: 'freehand',  label: 'Freehand',  shortcut: 'F', icon: <EditOutlined /> },
  { key: 'rectangle', label: 'Rectangle', shortcut: 'R', icon: <BorderOutlined /> },
  { key: 'ellipse',   label: 'Ellipse',   shortcut: 'E', icon: <RadiusUprightOutlined /> },
  { key: 'text',      label: 'Text',      shortcut: 'T', icon: <FontSizeOutlined /> },
  { key: 'eraser',    label: 'Eraser',    shortcut: 'X', icon: <ClearOutlined /> },
];

/**
 * Toolbar reads and writes directly from/to useCanvasStore.
 * No props needed — decoupled from parent layout.
 */
export function Toolbar() {
  const {
    activeTool, setActiveTool,
    strokeColor, setStrokeColor,
    fillColor,   setFillColor,
    strokeWidth, setStrokeWidth,
  } = useCanvasStore();

  const showFill = activeTool === 'rectangle' || activeTool === 'ellipse';

  // Keyboard shortcuts — only when not typing in an input
  useEffect(() => {
    const handler = (e) => {
      if (e.target.closest('input, textarea, [contenteditable]')) return;
      const tool = KEYBOARD_MAP[e.key.toLowerCase()];
      if (tool) setActiveTool(tool);
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [setActiveTool]);

  return (
    <div className="toolbar" role="toolbar" aria-label="Drawing tools">
      {/* Tool buttons */}
      <div className="toolbar__group">
        {TOOLS.map(({ key, label, shortcut, icon }) => (
          <Tooltip key={key} title={`${label} (${shortcut})`} placement="bottom">
            <Button
              className={`toolbar__btn${activeTool === key ? ' toolbar__btn--active' : ''}`}
              icon={icon}
              onClick={() => setActiveTool(key)}
              aria-pressed={activeTool === key}
            />
          </Tooltip>
        ))}
      </div>

      <Divider type="vertical" className="toolbar__divider" />

      {/* Colours */}
      <div className="toolbar__group">
        <Tooltip title="Stroke colour" placement="bottom">
          <div className="toolbar__color-wrap">
            <span className="toolbar__color-label">Stroke</span>
            <ColorPicker
              value={strokeColor}
              onChange={(c) => setStrokeColor(c.toHexString())}
              size="small"
            />
          </div>
        </Tooltip>

        {showFill && (
          <Tooltip title="Fill colour" placement="bottom">
            <div className="toolbar__color-wrap">
              <span className="toolbar__color-label">Fill</span>
              <ColorPicker
                value={fillColor === 'transparent' ? '#ffffff' : fillColor}
                onChange={(c) => setFillColor(c.toHexString())}
                size="small"
              />
            </div>
          </Tooltip>
        )}
      </div>

      <Divider type="vertical" className="toolbar__divider" />

      {/* Stroke width */}
      <div className="toolbar__group toolbar__group--slider">
        <span className="toolbar__slider-label">Width&nbsp;{strokeWidth}px</span>
        <Slider
          min={1}
          max={40}
          value={strokeWidth}
          onChange={setStrokeWidth}
          tooltip={{ formatter: (v) => `${v}px` }}
          style={{ width: 96 }}
        />
      </div>
    </div>
  );
}
