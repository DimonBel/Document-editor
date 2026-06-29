import { List, Typography } from 'antd';
import { useOutline, OutlineEntry } from './outline';

interface Props {
  source: string;
  onJump?: (line: number) => void;
}

/**
 * Document outline sidebar. Lists every section/subsection/etc. in the
 * current source and lets the user click to jump to the line.
 *
 * `onJump` is optional — when provided, clicks call it with the
 * 1-based line number of the heading. The page wires this to scroll
 * the editor to that line (best-effort; CodeMirror doesn't expose a
 * public scroll-to-line API beyond view.dispatch).
 */
export function OutlinePanel({ source, onJump }: Props) {
  const entries = useOutline(source);

  if (entries.length === 0) {
    return (
      <div className="ltx-outline ltx-outline--empty">
        <Typography.Text type="secondary">
          Add <code>\section{`{...}`}</code> headings to populate the outline.
        </Typography.Text>
      </div>
    );
  }

  return (
    <List
      className="ltx-outline"
      size="small"
      dataSource={entries}
      renderItem={(entry: OutlineEntry) => (
        <List.Item
          className={`ltx-outline__item ltx-outline__item--l${entry.level}`}
          onClick={() => onJump?.(entry.line)}
          style={{ cursor: onJump ? 'pointer' : 'default' }}
        >
          <span className="ltx-outline__title">{entry.title}</span>
          <span className="ltx-outline__line">L{entry.line}</span>
        </List.Item>
      )}
    />
  );
}