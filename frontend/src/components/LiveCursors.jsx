import React from 'react';
import { Layer, Group, Circle, Text } from 'react-konva';

/**
 * Renders every remote peer's cursor as a coloured dot + name label.
 * Placed on a non-interactive Layer so it never captures pointer events.
 */
export default function LiveCursors({ cursors }) {
  return (
    <Layer listening={false}>
      {Object.entries(cursors).map(([clientId, cursor]) => (
        <Group key={clientId} x={cursor.x ?? 0} y={cursor.y ?? 0}>
          <Circle radius={6} fill="#3b82f6" opacity={0.85} />
          <Text
            text={cursor.name ?? 'Anonymous'}
            x={10}
            y={-9}
            fontSize={12}
            fill="#3b82f6"
            fontStyle="bold"
          />
        </Group>
      ))}
    </Layer>
  );
}
