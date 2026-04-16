import { Tool } from '../../../types';

export const RectangleTool: Tool = {
  cursor: 'crosshair',

  onStart({ pos, strokeColor, fillColor }) {
    return {
      type: 'Rectangle',
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
    return {
      ...draft,
      x: Math.min(pos.x, draft.originX as number),
      y: Math.min(pos.y, draft.originY as number),
      w: Math.abs(pos.x - (draft.originX as number)),
      h: Math.abs(pos.y - (draft.originY as number)),
    };
  },

  onCommit({ draft }) {
    if ((draft.w as number) < 3 && (draft.h as number) < 3) return null;
    return {
      type: 'Rectangle',
      x: draft.x,
      y: draft.y,
      w: draft.w,
      h: draft.h,
      color: draft.color,
      fill: draft.fill,
    };
  },
};