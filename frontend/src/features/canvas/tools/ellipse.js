export const EllipseTool = {
  cursor: 'crosshair',

  onStart({ pos, strokeColor, fillColor }) {
    return {
      type:    'Ellipse',
      originX: pos.x,
      originY: pos.y,
      x: pos.x, y: pos.y,
      w: 0,     h: 0,
      color: strokeColor,
      fill:  fillColor,
    };
  },

  onMove({ draft, pos }) {
    const w = Math.abs(pos.x - draft.originX);
    const h = Math.abs(pos.y - draft.originY);
    return {
      ...draft,
      x: Math.min(pos.x, draft.originX),
      y: Math.min(pos.y, draft.originY),
      w, h,
    };
  },

  onCommit({ draft }) {
    if (draft.w < 3 && draft.h < 3) return null;
    return {
      type: 'Ellipse',
      data: {
        x:  draft.x + draft.w / 2,
        y:  draft.y + draft.h / 2,
        rx: draft.w / 2,
        ry: draft.h / 2,
        color: draft.color,
        fill:  draft.fill,
      },
    };
  },
};
