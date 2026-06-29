import { Popover, Tabs, Tooltip } from 'antd';
import { SYMBOL_CATEGORIES, symbolInsert } from './symbols';

interface Props {
  /** Called with the LaTeX command to insert (without the leading `\`). */
  onInsert: (cmd: string) => void;
  /** Wrap the trigger button. */
  children: React.ReactNode;
}

/**
 * Clickable symbol palette popover. Renders the categorized symbol
 * grid from `symbols.ts` and emits a LaTeX command via `onInsert` when
 * the user picks one.
 */
export function SymbolPalette({ onInsert, children }: Props) {
  const items = SYMBOL_CATEGORIES.map((cat) => ({
    key: cat.id,
    label: cat.title,
    children: (
      <div className="ltx-symbol-grid">
        {cat.symbols.map((sym) => (
          <Tooltip key={sym.cmd} title={`\\${sym.cmd}`} mouseEnterDelay={0.4}>
            <button
              className="ltx-symbol-btn"
              type="button"
              onClick={() => onInsert(symbolInsert(sym.cmd).slice(1))}
            >
              {sym.glyph}
            </button>
          </Tooltip>
        ))}
      </div>
    ),
  }));

  return (
    <Popover
      trigger="click"
      placement="bottomRight"
      content={<Tabs items={items} size="small" />}
    >
      {children}
    </Popover>
  );
}