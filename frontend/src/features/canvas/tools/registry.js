import { FreehandTool  } from './freehand.js';
import { EraserTool   } from './eraser.js';
import { RectangleTool } from './rectangle.js';
import { EllipseTool  } from './ellipse.js';
import { TextTool     } from './text.js';

/**
 * Central tool registry.
 *
 * To add a new tool:
 *   1. Create `myTool.js` exporting an object with cursor / onStart / onMove / onCommit.
 *   2. Import it here and add an entry.
 *   3. Add a button in Toolbar.jsx.
 *   That's it — no other file needs to change.
 */
export const TOOL_REGISTRY = {
  freehand:  FreehandTool,
  eraser:    EraserTool,
  rectangle: RectangleTool,
  ellipse:   EllipseTool,
  text:      TextTool,
};

/** @param {string} key */
export const getTool = (key) => TOOL_REGISTRY[key] ?? TOOL_REGISTRY.freehand;

/** Keyboard shortcut → tool key */
export const KEYBOARD_MAP = {
  f: 'freehand',
  x: 'eraser',
  r: 'rectangle',
  e: 'ellipse',
  t: 'text',
};
