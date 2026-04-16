export const RectangleTool = {
  cursor: 'crosshair',

  onStart({ pos, strokeColor, fillColor }) {
    return {
      type:    'Rectangle',
      originX: pos.x,
      originY: pos.y,
      x: pos.x, y: pos.y,
      w: 0,     h: 0,
      color: strokeColor,
      fill:  fillColor,
    };
  },

  onMove({ draft, pos }) {
    return {
      ...draft,
      x: Math.min(pos.x, draft.originX),
      y: Math.min(pos.y, draft.originY),
      w: Math.abs(pos.x - draft.originX),
      h: Math.abs(pos.y - draft.originY),
    };
  },

  onCommit({ draft }) {
    if (draft.w < 3 && draft.h < 3) return null;
    return {
      type: 'Rectangle',
      data: { x: draft.x, y: draft.y, w: draft.w, h: draft.h, color: draft.color, fill: draft.fill },
    };
  },
};
