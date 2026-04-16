import React from 'react';
import { Line, Rect, Ellipse as KonvaEllipse, Text as KonvaText } from 'react-konva';

/**
 * Pure renderer — maps a DrawElement to a react-konva node.
 * Keeping this as a plain function (not a component) lets callers
 * embed it inside any Layer without extra component overhead.
 */
export function renderElement(el) {
  if (!el?.type) return null;

  switch (el.type) {
    case 'Freehand':
      return (
        <Line
          key={el.id}
          points={el.points}
          stroke={el.color ?? '#000'}
          strokeWidth={el.width ?? 4}
          tension={0.4}
          lineCap="round"
          lineJoin="round"
          globalCompositeOperation="source-over"
          listening={false}
        />
      );

    case 'Eraser':
      return (
        <Line
          key={el.id}
          points={el.points}
          stroke="white"
          strokeWidth={el.width ?? 20}
          tension={0.4}
          lineCap="round"
          lineJoin="round"
          globalCompositeOperation="destination-out"
          listening={false}
        />
      );

    case 'Rectangle':
      return (
        <Rect
          key={el.id}
          x={el.x}
          y={el.y}
          width={el.w}
          height={el.h}
          stroke={el.color ?? '#000'}
          strokeWidth={2}
          fill={el.fill && el.fill !== 'transparent' ? el.fill : undefined}
          listening={false}
        />
      );

    case 'Ellipse':
      return (
        <KonvaEllipse
          key={el.id}
          x={el.x}
          y={el.y}
          radiusX={el.rx}
          radiusY={el.ry}
          stroke={el.color ?? '#000'}
          strokeWidth={2}
          fill={el.fill && el.fill !== 'transparent' ? el.fill : undefined}
          listening={false}
        />
      );

    case 'Text':
      return (
        <KonvaText
          key={el.id}
          x={el.x}
          y={el.y}
          text={el.content ?? ''}
          fontSize={el.fontSize ?? 18}
          fill={el.color ?? '#000'}
          listening={false}
        />
      );

    default:
      return null;
  }
}
