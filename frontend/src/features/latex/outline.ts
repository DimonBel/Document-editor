import { useMemo } from 'react';

export interface OutlineEntry {
  /** 1-based heading level: chapter=1, section=1, subsection=2, subsubsection=3. */
  level: number;
  title: string;
  /** 1-based line number where the heading starts in the source. */
  line: number;
}

/**
 * Extracts a heading outline from LaTeX source by scanning for
 * `\chapter{...}`, `\section{...}`, `\subsection{...}`,
 * `\subsubsection{...}` and their `*`-starred variants.
 *
 * The match is intentionally lightweight: it walks line-by-line and
 * regex-matches the `\command{title}` pattern. It does not understand
 * `\newcommand`-renamed headings or `\input`-included files, which is
 * fine for the editor surface — the document stays single-file.
 */
export function extractOutline(source: string): OutlineEntry[] {
  const re = /^\s*\\(chapter|section|subsection|subsubsection)\*?\s*\{((?:[^{}]|\{[^{}]*\})*)\}\s*$/;
  const out: OutlineEntry[] = [];
  const lines = source.split('\n');
  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(re);
    if (!m) continue;
    const cmd = m[1];
    const title = m[2];
    const level =
      cmd === 'chapter' || cmd === 'section' ? 1 :
      cmd === 'subsection' ? 2 :
      cmd === 'subsubsection' ? 3 : 1;
    out.push({ level, title: title.trim(), line: i + 1 });
  }
  return out;
}

/**
 * Reactive wrapper around `extractOutline` so the outline re-derives
 * only when the source string changes.
 */
export function useOutline(source: string): OutlineEntry[] {
  return useMemo(() => extractOutline(source), [source]);
}