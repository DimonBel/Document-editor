import { Tool } from '../../../types';

export const TextTool: Tool = {
  cursor: 'text',

  onStart({ pos, strokeColor, addElement }) {
    const content = window.prompt('Enter text:');
    if (content?.trim()) {
      addElement('Text', { x: pos.x, y: pos.y, content: content.trim(), fontSize: 18, color: strokeColor });
    }
    return null;
  },

  onMove({ draft }) {
    return draft;
  },

  onCommit() {
    return null;
  },
};