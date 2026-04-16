import { Tool } from '../../../types';

export const FreehandTool: Tool = {
  cursor: 'crosshair',

  onStart({ pos, strokeColor, strokeWidth }) {
    return {
      type: 'Freehand',
      points: [pos.x, pos.y],
      color: strokeColor,
      width: strokeWidth,
    };
  },

  onMove({ draft, pos }) {
    return { ...draft, points: [...(draft.points as number[]), pos.x, pos.y] };
  },

  onCommit({ draft }) {
    const points = draft.points as number[];
    if (points.length < 4) return null;
    return { type: 'Freehand', points, color: draft.color as string, width: draft.width as number };
  },
};