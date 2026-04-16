import { Tool } from '../../../types';

export const EraserTool: Tool = {
  cursor: 'cell',

  onStart({ pos, strokeWidth }) {
    return {
      type: 'Eraser',
      points: [pos.x, pos.y],
      width: strokeWidth * 3,
    };
  },

  onMove({ draft, pos }) {
    return { ...draft, points: [...(draft.points as number[]), pos.x, pos.y] };
  },

  onCommit({ draft }) {
    const points = draft.points as number[];
    if (points.length < 4) return null;
    return { type: 'Eraser', points, width: draft.width as number };
  },
};