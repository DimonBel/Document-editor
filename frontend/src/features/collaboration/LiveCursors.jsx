import React from 'react';
import { Layer, Group, Circle, Text } from 'react-konva';

/**
 * Renders all remote peer cursors as coloured dots with name labels.
 * Lives on a `listening={false}` Layer so it never captures pointer events.
 */
export function LiveCursors({ cursors }) {
  return (
    <Layer listening={false}>
      {Object.entries(cursors).map(([clientId, cursor], i) => {
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

const CURSOR_COLORS = [
  '#3b82f6','#ef4444','#10b981','#f59e0b',
  '#8b5cf6','#14b8a6','#f97316','#ec4899',
];

function hashCode(str) {
  let h = 0;
  for (let i = 0; i < str.length; i++) h = (Math.imul(31, h) + str.charCodeAt(i)) | 0;
  return h;
}
