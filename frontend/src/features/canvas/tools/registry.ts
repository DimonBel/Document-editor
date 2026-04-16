import { ToolType, Tool } from '../../../types';
import { FreehandTool } from './freehand';
import { EraserTool } from './eraser';
import { RectangleTool } from './rectangle';
import { EllipseTool } from './ellipse';
import { TextTool } from './text';

export const TOOL_REGISTRY: Record<ToolType, Tool> = {
  freehand: FreehandTool,
  eraser: EraserTool,
  rectangle: RectangleTool,
  ellipse: EllipseTool,
  text: TextTool,
};

export const getTool = (key: ToolType): Tool => TOOL_REGISTRY[key] ?? TOOL_REGISTRY.freehand;

export const KEYBOARD_MAP: Record<string, ToolType> = {
  f: 'freehand',
  x: 'eraser',
  r: 'rectangle',
  e: 'ellipse',
  t: 'text',
};