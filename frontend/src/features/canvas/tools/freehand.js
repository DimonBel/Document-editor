/**
 * Freehand tool — continuous stroke following the pointer.
 *
 * Each tool exports an object with:
 *   cursor    — CSS cursor string
 *   onStart   — called on pointer-down; returns the initial live-element draft
 *   onMove    — called on pointer-move; returns an updated draft (pure, no mutation)
 *   onCommit  — called on pointer-up; returns the final { type, data } to add,
 *               or null to discard (e.g. too short a stroke)
 */
export const FreehandTool = {
  cursor: 'crosshair',

  onStart({ pos, strokeColor, strokeWidth }) {
    return {
      type:   'Freehand',
      points: [pos.x, pos.y],
      color:  strokeColor,
      width:  strokeWidth,
    };
  },

  onMove({ draft, pos }) {
    return { ...draft, points: [...draft.points, pos.x, pos.y] };
  },

  onCommit({ draft }) {
    if (draft.points.length < 4) return null;
    return { type: 'Freehand', data: { points: draft.points, color: draft.color, width: draft.width } };
  },
};
