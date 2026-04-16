export const EraserTool = {
  cursor: 'cell',

  onStart({ pos, strokeWidth }) {
    return {
      type:   'Eraser',
      points: [pos.x, pos.y],
      width:  strokeWidth * 3,
    };
  },

  onMove({ draft, pos }) {
    return { ...draft, points: [...draft.points, pos.x, pos.y] };
  },

  onCommit({ draft }) {
    if (draft.points.length < 4) return null;
    return { type: 'Eraser', data: { points: draft.points, width: draft.width } };
  },
};
