/**
 * Text tool — fires a browser prompt on pointer-down then immediately commits.
 * Because it resolves synchronously (blocking prompt), onMove / onCommit are no-ops.
 */
export const TextTool = {
  cursor: 'text',

  onStart({ pos, strokeColor, addElement }) {
    const content = window.prompt('Enter text:');
    if (content?.trim()) {
      addElement('Text', { x: pos.x, y: pos.y, content: content.trim(), fontSize: 18, color: strokeColor });
    }
    // Return null — no live preview needed
    return null;
  },

  onMove({ draft }) {
    return draft; // no-op
  },

  onCommit() {
    return null; // committed eagerly in onStart
  },
};
