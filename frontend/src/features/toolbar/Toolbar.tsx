import React, { useEffect } from 'react';
import { Tooltip, Slider, ColorPicker, Divider, Button } from 'antd';
import {
  EditOutlined,
  BorderOutlined,
  RadiusUprightOutlined,
  FontSizeOutlined,
  ClearOutlined,
} from '@ant-design/icons';
import { useCanvasStore } from '../../store/canvasStore';
import { KEYBOARD_MAP } from '../canvas/tools/registry';
import { ToolType } from '../../types';
import './Toolbar.css';

const TOOLS: { key: ToolType; label: string; shortcut: string; icon: React.ReactNode }[] = [
  { key: 'freehand', label: 'Freehand', shortcut: 'F', icon: <EditOutlined /> },
  { key: 'rectangle', label: 'Rectangle', shortcut: 'R', icon: <BorderOutlined /> },
  { key: 'ellipse', label: 'Ellipse', shortcut: 'E', icon: <RadiusUprightOutlined /> },
  { key: 'text', label: 'Text', shortcut: 'T', icon: <FontSizeOutlined /> },
  { key: 'eraser', label: 'Eraser', shortcut: 'X', icon: <ClearOutlined /> },
];

export function Toolbar() {
  const {
    activeTool, setActiveTool,
    strokeColor, setStrokeColor,
    fillColor, setFillColor,
    strokeWidth, setStrokeWidth,
  } = useCanvasStore();

  const showFill = activeTool === 'rectangle' || activeTool === 'ellipse';

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.target instanceof Element && e.target.closest('input, textarea, [contenteditable]')) return;
      const tool = KEYBOARD_MAP[e.key.toLowerCase()];
      if (tool) setActiveTool(tool);
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [setActiveTool]);

  return (
    <div className="toolbar" role="toolbar" aria-label="Drawing tools">
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