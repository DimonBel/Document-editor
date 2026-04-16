import React from 'react';
import { Button, Slider, Tooltip, Space, ColorPicker } from 'antd';
import {
  EditOutlined,
  BorderOutlined,
  FieldTimeOutlined,
  FontSizeOutlined,
  ClearOutlined,
} from '@ant-design/icons';
import './Toolbar.css';

const TOOLS = [
  { key: 'freehand',  icon: <EditOutlined />,       label: 'Freehand (F)' },
  { key: 'rectangle', icon: <BorderOutlined />,      label: 'Rectangle (R)' },
  { key: 'ellipse',   icon: <FieldTimeOutlined />,   label: 'Ellipse (E)' },
  { key: 'text',      icon: <FontSizeOutlined />,    label: 'Text (T)' },
  { key: 'eraser',    icon: <ClearOutlined />,       label: 'Eraser (X)' },
];

export default function Toolbar({
  tool, setTool,
  color, setColor,
  strokeWidth, setStrokeWidth,
  fillColor, setFillColor,
}) {
  const showFill = tool === 'rectangle' || tool === 'ellipse';

  return (
    <div className="toolbar">
      <Space size={4} wrap={false}>
        {/* Tool buttons */}
        {TOOLS.map((t) => (
          <Tooltip title={t.label} key={t.key}>
            <Button
              type={tool === t.key ? 'primary' : 'default'}
              icon={t.icon}
              onClick={() => setTool(t.key)}
            />
          </Tooltip>
        ))}

        <div className="tb-divider" />

        {/* Stroke colour */}
        <Tooltip title="Stroke colour">
          <ColorPicker
            value={color}
            onChange={(c) => setColor(c.toHexString())}
            size="middle"
          />
        </Tooltip>

        {/* Fill colour — shapes only */}
        {showFill && (
          <Tooltip title="Fill colour">
            <ColorPicker
              value={fillColor === 'transparent' ? '#ffffff' : fillColor}
              onChange={(c) => setFillColor(c.toHexString())}
              size="middle"
            />
          </Tooltip>
        )}

        <div className="tb-divider" />

        {/* Stroke width */}
        <Tooltip title={`Stroke width: ${strokeWidth}px`}>
          <div className="tb-slider">
            <span className="tb-slider-label">Width</span>
            <Slider
              min={1}
              max={40}
              value={strokeWidth}
              onChange={setStrokeWidth}
              style={{ width: 90 }}
              tooltip={{ formatter: (v) => `${v}px` }}
            />
          </div>
        </Tooltip>
      </Space>
    </div>
  );
}
