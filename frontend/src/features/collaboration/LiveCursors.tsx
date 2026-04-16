import { Layer, Group, Circle, Text } from 'react-konva';
import { CursorPosition } from '../../types';

interface LiveCursorsProps {
  cursors: Record<string, CursorPosition>;
}

const CURSOR_COLORS = [
  '#3b82f6', '#ef4444', '#10b981', '#f59e0b',
  '#8b5cf6', '#14b8a6', '#f97316', '#ec4899',
];

function hashCode(str: string): number {
  let h = 0;
  for (let i = 0; i < str.length; i++) h = (Math.imul(31, h) + str.charCodeAt(i)) | 0;
  return h;
}

export function LiveCursors({ cursors }: LiveCursorsProps) {
  return (
    <Layer listening={false}>
      {Object.entries(cursors).map(([clientId, cursor]) => {
        const color = CURSOR_COLORS[Math.abs(hashCode(clientId)) % CURSOR_COLORS.length];
        return (
          <Group key={clientId} x={cursor.x ?? 0} y={cursor.y ?? 0}>
            <Circle radius={6} fill={color} opacity={0.9} />
            <Text
              text={cursor.name ?? 'Anonymous'}
              x={10}
              y={-9}
              fontSize={12}
              fill={color}
              fontStyle="bold"
            />
          </Group>
        );
      })}
    </Layer>
  );
}