export interface DrawElement {
  id: string;
  type: string;
  [key: string]: unknown;
}

export interface OpInsert {
  type: 'Insert';
  element: DrawElement;
}

export interface OpDelete {
  type: 'Delete';
  id: string;
}

export interface OpUpdate {
  type: 'Update';
  id: string;
  element: DrawElement;
}

export type OpType = OpInsert | OpDelete | OpUpdate;

export interface Operation {
  id: string;
  clientId: string;
  lamport: number;
  op: OpType;
}

export interface RoomInfo {
  id: string;
  name: string;
  created_at: string;
  client_count: number;
}

export interface ClientInfo {
  id: string;
  name: string;
}

export interface CursorPosition {
  x: number;
  y: number;
  name?: string;
}

export type ToolType = 'freehand' | 'rectangle' | 'ellipse' | 'text' | 'eraser';

export interface Point {
  x: number;
  y: number;
}

export interface ToolContext {
  pos: Point;
  strokeColor: string;
  fillColor: string;
  strokeWidth: number;
  addElement: (type: string, data: Record<string, unknown>) => void;
}

export interface DraftResult {
  type: string;
  [key: string]: unknown;
}

export interface Tool {
  onStart: (ctx: ToolContext) => DraftResult | null;
  onMove: (ctx: { draft: DraftResult; pos: Point }) => DraftResult | null;
  onCommit: (ctx: { draft: DraftResult; addElement: (type: string, data: Record<string, unknown>) => void }) => DraftResult | null;
  cursor: string;
}