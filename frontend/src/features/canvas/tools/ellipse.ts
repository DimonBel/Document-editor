import { Tool } from '../../../types';

export const EllipseTool: Tool = {
  cursor: 'crosshair',

  onStart({ pos, strokeColor, fillColor }) {
    return {
      type: 'Ellipse',
      originX: pos.x,
      originY: pos.y,
      x: pos.x,
      y: pos.y,
      w: 0,
      h: 0,
      color: strokeColor,
      fill: fillColor,
    };
  },

  onMove({ draft, pos }) {
    const w = Math.abs(pos.x - (draft.originX as number));
    const h = Math.abs(pos.y - (draft.originY as number));
    return {
      ...draft,
      x: Math.min(pos.x, draft.originX as number),
      y: Math.min(pos.y, draft.originY as number),
      w,
      h,
    };
  },

  onCommit({ draft }) {
    if ((draft.w as number) < 3 && (draft.h as number) < 3) return null;
    return {
      type: 'Ellipse',
      x: (draft.x as number) + (draft.w as number) / 2,
      y: (draft.y as number) + (draft.h as number) / 2,
      rx: (draft.w as number) / 2,
      ry: (draft.h as number) / 2,
      color: draft.color,
      fill: draft.fill,
    };
  },
};