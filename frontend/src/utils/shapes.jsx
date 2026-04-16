import React from 'react';
import { Line, Rect, Ellipse as KonvaEllipse, Text as KonvaText } from 'react-konva';

/**
 * Convert a DrawElement (from the CRDT) into a react-konva node.
 * Returns `null` for unknown types so the caller can safely filter.
 */
export function renderElement(el) {
  if (!el?.type) return null;

  switch (el.type) {
    case 'Freehand':
      return (
        <Line
          key={el.id}
          points={el.points}
          stroke={el.color ?? '#000000'}
          strokeWidth={el.width ?? 3}
          tension={0.4}
          lineCap="round"
          lineJoin="round"
          globalCompositeOperation="source-over"
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
          stroke={el.color ?? '#000000'}
          strokeWidth={2}
          fill={el.fill && el.fill !== 'transparent' ? el.fill : undefined}
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
          stroke={el.color ?? '#000000'}
          strokeWidth={2}
          fill={el.fill && el.fill !== 'transparent' ? el.fill : undefined}
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
          fill={el.color ?? '#000000'}
        />
      );

    default:
      return null;
  }
}
