import { Line, Rect, Ellipse as KonvaEllipse, Text as KonvaText } from 'react-konva';
import { DrawElement } from '../../types';

interface FreehandElement extends DrawElement {
  type: 'Freehand';
  points: number[];
  color: string;
  width: number;
}

interface EraserElement extends DrawElement {
  type: 'Eraser';
  points: number[];
  width: number;
}

interface RectangleElement extends DrawElement {
  type: 'Rectangle';
  x: number;
  y: number;
  w: number;
  h: number;
  color: string;
  fill?: string;
}

interface EllipseElement extends DrawElement {
  type: 'Ellipse';
  x: number;
  y: number;
  rx: number;
  ry: number;
  color: string;
  fill?: string;
}

interface TextElement extends DrawElement {
  type: 'Text';
  x: number;
  y: number;
  content: string;
  fontSize: number;
  color: string;
}

export function renderElement(el: DrawElement | null): React.ReactNode {
  if (!el?.type) return null;

  switch (el.type) {
    case 'Freehand': {
      const e = el as FreehandElement;
      return (
        <Line
          key={e.id}
          points={e.points}
          stroke={e.color ?? '#000'}
          strokeWidth={e.width ?? 4}
          tension={0.4}
          lineCap="round"
          lineJoin="round"
          globalCompositeOperation="source-over"
          listening={false}
        />
      );
    }

    case 'Eraser': {
      const e = el as EraserElement;
      return (
        <Line
          key={e.id}
          points={e.points}
          stroke="white"
          strokeWidth={e.width ?? 20}
          tension={0.4}
          lineCap="round"
          lineJoin="round"
          globalCompositeOperation="destination-out"
          listening={false}
        />
      );
    }

    case 'Rectangle': {
      const e = el as RectangleElement;
      return (
        <Rect
          key={e.id}
          x={e.x}
          y={e.y}
          width={e.w}
          height={e.h}
          stroke={e.color ?? '#000'}
          strokeWidth={2}
          fill={e.fill && e.fill !== 'transparent' ? e.fill : undefined}
          listening={false}
        />
      );
    }

    case 'Ellipse': {
      const e = el as EllipseElement;
      return (
        <KonvaEllipse
          key={e.id}
          x={e.x}
          y={e.y}
          radiusX={e.rx}
          radiusY={e.ry}
          stroke={e.color ?? '#000'}
          strokeWidth={2}
          fill={e.fill && e.fill !== 'transparent' ? e.fill : undefined}
          listening={false}
        />
      );
    }

    case 'Text': {
      const e = el as TextElement;
      return (
        <KonvaText
          key={e.id}
          x={e.x}
          y={e.y}
          text={e.content ?? ''}
          fontSize={e.fontSize ?? 18}
          fill={e.color ?? '#000'}
          listening={false}
        />
      );
    }

    default:
      return null;
  }
}