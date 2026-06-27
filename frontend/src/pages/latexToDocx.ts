import {
  Document, Packer, Paragraph, TextRun, HeadingLevel,
  Table, TableRow, TableCell, WidthType, AlignmentType,
  BorderStyle, LevelFormat,
  Math as DocxMath, MathRun, MathFraction, MathRadical,
  MathSuperScript, MathSubScript, MathSubSuperScript,
  MathSum, MathIntegral, MathFunction,
  MathRoundBrackets, MathSquareBrackets, MathCurlyBrackets,
  MathAngledBrackets, MathLimit, MathLimitLower, MathLimitUpper,
} from 'docx';

// ─── LaTeX math → docx OMML ──────────────────────────────────────────────────

type MathChild =
  | MathRun | MathFraction | MathRadical
  | MathSuperScript | MathSubScript | MathSubSuperScript
  | MathSum | MathIntegral | MathFunction
  | MathRoundBrackets | MathSquareBrackets | MathCurlyBrackets | MathAngledBrackets
  | MathLimit | MathLimitLower | MathLimitUpper;

const GREEK: Record<string, string> = {
  alpha:'α', beta:'β', gamma:'γ', delta:'δ', epsilon:'ε', zeta:'ζ',
  eta:'η', theta:'θ', iota:'ι', kappa:'κ', lambda:'λ', mu:'μ',
  nu:'ν', xi:'ξ', pi:'π', rho:'ρ', sigma:'σ', tau:'τ',
  upsilon:'υ', phi:'φ', chi:'χ', psi:'ψ', omega:'ω',
  Gamma:'Γ', Delta:'Δ', Theta:'Θ', Lambda:'Λ', Xi:'Ξ',
  Pi:'Π', Sigma:'Σ', Upsilon:'Υ', Phi:'Φ', Psi:'Ψ', Omega:'Ω',
};

const SYMBOLS: Record<string, string> = {
  pm:'±', mp:'∓', times:'×', div:'÷', cdot:'·', circ:'∘',
  leq:'≤', geq:'≥', neq:'≠', approx:'≈', equiv:'≡', sim:'∼',
  subset:'⊂', supset:'⊃', subseteq:'⊆', supseteq:'⊇',
  in:'∈', notin:'∉', cup:'∪', cap:'∩', setminus:'∖',
  forall:'∀', exists:'∃', neg:'¬', land:'∧', lor:'∨',
  rightarrow:'→', leftarrow:'←', Rightarrow:'⇒', Leftarrow:'⇐',
  leftrightarrow:'↔', Leftrightarrow:'⟺', to:'→', gets:'←',
  infty:'∞', partial:'∂', nabla:'∇', emptyset:'∅',
  ldots:'…', cdots:'⋯', vdots:'⋮', ddots:'⋱',
  langle:'⟨', rangle:'⟩',
  quad:'\u2003', qquad:'\u2003\u2003',
  sqrt: '', // handled separately
};

function mr(text: string): MathRun { return new MathRun(text); }

// Tokeniser: returns array of tokens from a LaTeX math string
type Token = { kind: 'cmd'; val: string } | { kind: 'char'; val: string } | { kind: 'group'; val: string };

function tokenise(src: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  while (i < src.length) {
    if (src[i] === '\\') {
      let j = i + 1;
      if (j < src.length && /[a-zA-Z]/.test(src[j])) {
        while (j < src.length && /[a-zA-Z]/.test(src[j])) j++;
        tokens.push({ kind: 'cmd', val: src.slice(i + 1, j) });
      } else {
        tokens.push({ kind: 'cmd', val: src[j] ?? '' });
        j++;
      }
      i = j;
    } else if (src[i] === '{') {
      // find matching }
      let depth = 1, j = i + 1;
      while (j < src.length && depth > 0) {
        if (src[j] === '{') depth++;
        else if (src[j] === '}') depth--;
        j++;
      }
      tokens.push({ kind: 'group', val: src.slice(i + 1, j - 1) });
      i = j;
    } else if (src[i] === ' ' || src[i] === '\n') {
      i++;
    } else {
      tokens.push({ kind: 'char', val: src[i] });
      i++;
    }
  }
  return tokens;
}

// Convert a group (string) or single char to array of MathChildren
function parseGroup(src: string): MathChild[] {
  return parseMath(tokenise(src));
}

function nextArg(tokens: Token[], idx: number): { children: MathChild[]; next: number } {
  const t = tokens[idx];
  if (!t) return { children: [], next: idx };
  if (t.kind === 'group') return { children: parseGroup(t.val), next: idx + 1 };
  // single char/cmd as argument
  return { children: parseToken(tokens, idx).children, next: parseToken(tokens, idx).next };
}

function parseToken(tokens: Token[], i: number): { children: MathChild[]; next: number } {
  const t = tokens[i];
  if (!t) return { children: [], next: i };

  if (t.kind === 'group') {
    return { children: parseGroup(t.val), next: i + 1 };
  }

  if (t.kind === 'char') {
    return { children: [mr(t.val)], next: i + 1 };
  }

  // cmd
  const cmd = t.val;

  if (cmd in GREEK) return { children: [mr(GREEK[cmd])], next: i + 1 };
  if (cmd in SYMBOLS && cmd !== 'sqrt') return { children: [mr(SYMBOLS[cmd])], next: i + 1 };

  if (cmd === 'frac') {
    const num = nextArg(tokens, i + 1);
    const den = nextArg(tokens, num.next);
    return {
      children: [new MathFraction({ numerator: num.children, denominator: den.children })],
      next: den.next,
    };
  }

  if (cmd === 'sqrt') {
    // optional degree: \sqrt[n]{x}
    let degree: MathChild[] | undefined;
    let ni = i + 1;
    if (tokens[ni]?.kind === 'char' && tokens[ni].val === '[') {
      // collect until ]
      let degStr = '';
      ni++;
      while (ni < tokens.length && !(tokens[ni].kind === 'char' && tokens[ni].val === ']')) {
        degStr += tokens[ni].val;
        ni++;
      }
      ni++; // skip ]
      degree = parseGroup(degStr);
    }
    const base = nextArg(tokens, ni);
    return {
      children: [new MathRadical({ degree, children: base.children })],
      next: base.next,
    };
  }

  if (cmd === 'sum') {
    return buildNary(tokens, i + 1, 'sum');
  }
  if (cmd === 'prod') {
    return buildNary(tokens, i + 1, 'prod');
  }
  if (cmd === 'int' || cmd === 'iint' || cmd === 'iiint' || cmd === 'oint') {
    return buildNary(tokens, i + 1, 'int');
  }
  if (cmd === 'lim' || cmd === 'limsup' || cmd === 'liminf') {
    return buildNary(tokens, i + 1, 'lim', cmd);
  }

  if (cmd === 'text') {
    const str = tokens[i + 1]?.kind === 'group' ? (tokens[i + 1] as {kind:'group';val:string}).val : '';
    return { children: [mr(str)], next: i + 2 };
  }

  if (cmd === 'mathbb' || cmd === 'mathbf' || cmd === 'mathit' || cmd === 'mathrm' || cmd === 'mathcal' || cmd === 'boldsymbol') {
    const arg = nextArg(tokens, i + 1);
    return { children: arg.children, next: arg.next };
  }

  if (cmd === 'left') {
    return parseLeft(tokens, i + 1);
  }
  if (cmd === 'right') {
    // Should be consumed by parseLeft; return empty
    return { children: [], next: i + 2 };
  }

  if (cmd === 'begin') {
    return parseBeginEnd(tokens, i + 1);
  }

  // Misc formatting: just return space or skip
  if (['quad','qquad'].includes(cmd)) return { children: [mr(SYMBOLS[cmd])], next: i + 1 };
  if ([',',';','!',':'].includes(cmd)) return { children: [mr('\u2009')], next: i + 1 };
  if (['cdot'].includes(cmd)) return { children: [mr('·')], next: i + 1 };
  if (cmd === 'infty') return { children: [mr('∞')], next: i + 1 };
  if (cmd === 'partial') return { children: [mr('∂')], next: i + 1 };
  if (cmd === 'nabla') return { children: [mr('∇')], next: i + 1 };
  if (cmd === 'ldots' || cmd === 'cdots' || cmd === 'dots') return { children: [mr('…')], next: i + 1 };

  // Trig / log functions
  const FUNCS = ['sin','cos','tan','cot','sec','csc','arcsin','arccos','arctan',
                 'log','ln','exp','max','min','sup','inf','det','dim','ker','deg','gcd'];
  if (FUNCS.includes(cmd)) {
    const name = [mr(cmd)];
    return {
      children: [new MathFunction({ name, children: [] })],
      next: i + 1,
    };
  }

  if (cmd === 'hat' || cmd === 'bar' || cmd === 'tilde' || cmd === 'vec' || cmd === 'dot' || cmd === 'ddot' || cmd === 'acute' || cmd === 'grave') {
    const base = nextArg(tokens, i + 1);
    // Represent as superscript workaround (not perfect but functional)
    return { children: base.children, next: base.next };
  }

  if (cmd === 'overline' || cmd === 'underline' || cmd === 'widehat' || cmd === 'widetilde') {
    const base = nextArg(tokens, i + 1);
    return { children: base.children, next: base.next };
  }

  if (cmd === 'overbrace' || cmd === 'underbrace') {
    const base = nextArg(tokens, i + 1);
    return { children: base.children, next: base.next };
  }

  if (cmd === 'binom') {
    const top = nextArg(tokens, i + 1);
    const bot = nextArg(tokens, top.next);
    return {
      children: [new MathRoundBrackets({ children: [new MathFraction({ numerator: top.children, denominator: bot.children })] })],
      next: bot.next,
    };
  }

  if (cmd === 'not') {
    const base = nextArg(tokens, i + 1);
    return { children: [...base.children, mr('\u0338')], next: base.next };
  }

  // Unknown command — render as-is
  return { children: [mr('\\' + cmd)], next: i + 1 };
}

function buildNary(tokens: Token[], start: number, type: 'sum'|'prod'|'int'|'lim', name?: string): { children: MathChild[]; next: number } {
  let subScript: MathChild[] | undefined;
  let superScript: MathChild[] | undefined;
  let i = start;

  // peek for _ and ^
  while (i < tokens.length) {
    const t = tokens[i];
    if (t.kind === 'char' && t.val === '_') {
      const sub = nextArg(tokens, i + 1);
      subScript = sub.children;
      i = sub.next;
    } else if (t.kind === 'char' && t.val === '^') {
      const sup = nextArg(tokens, i + 1);
      superScript = sup.children;
      i = sup.next;
    } else {
      break;
    }
  }

  // body — next token group or single token
  const body = nextArg(tokens, i);

  if (type === 'sum') return { children: [new MathSum({ children: body.children, subScript, superScript })], next: body.next };
  if (type === 'int') return { children: [new MathIntegral({ children: body.children, subScript, superScript })], next: body.next };
  if (type === 'lim') {
    const limName = name ?? 'lim';
    const lim = new MathFunction({
      name: [mr(limName)],
      children: body.children,
    });
    return { children: [lim], next: body.next };
  }
  // prod
  return { children: [new MathSum({ children: body.children, subScript, superScript })], next: body.next };
}

function parseLeft(tokens: Token[], start: number): { children: MathChild[]; next: number } {
  // tokens[start] should be opening bracket char
  const openTok = tokens[start];
  const openChar = openTok?.kind === 'char' ? openTok.val
    : openTok?.kind === 'cmd' ? (openTok.val === 'langle' ? '⟨' : openTok.val === '.' ? '' : openTok.val) : '(';
  let i = start + 1;

  // collect inner until \right at depth 0
  const inner: Token[] = [];
  let depth = 1;
  while (i < tokens.length) {
    const t = tokens[i];
    if (t.kind === 'cmd' && t.val === 'left') depth++;
    else if (t.kind === 'cmd' && t.val === 'right') {
      depth--;
      if (depth === 0) break;
    }
    inner.push(t);
    i++;
  }
  if (i >= tokens.length) {
    // Unmatched \left — fall back to round brackets and consume the rest.
    const innerChildren = parseMath(inner);
    return {
      children: [new MathRoundBrackets({ children: innerChildren })],
      next: i,
    };
  }
  // Skip the \right token and exactly one closing-bracket token
  // (a single char like `)` or a group `{...}`).
  i += 1;
  if (i < tokens.length) {
    const next = tokens[i];
    if (next.kind === 'group' || next.kind === 'char') i += 1;
  }

  const innerChildren = parseMath(inner);

  if (openChar === '(' || openChar === ')') return { children: [new MathRoundBrackets({ children: innerChildren })], next: i };
  if (openChar === '[' || openChar === ']') return { children: [new MathSquareBrackets({ children: innerChildren })], next: i };
  if (openChar === '{') return { children: [new MathCurlyBrackets({ children: innerChildren })], next: i };
  if (openChar === '⟨') return { children: [new MathAngledBrackets({ children: innerChildren })], next: i };
  return { children: [new MathRoundBrackets({ children: innerChildren })], next: i };
}

function parseBeginEnd(tokens: Token[], start: number): { children: MathChild[]; next: number } {
  const envTok = tokens[start];
  if (!envTok || envTok.kind !== 'group') return { children: [], next: start };
  let i = start + 1;
  const inner: Token[] = [];
  while (i < tokens.length) {
    const t = tokens[i];
    if (t.kind === 'cmd' && t.val === 'end') { i += 2; break; }
    inner.push(t);
    i++;
  }
  // For aligned/cases environments: collect rows and render as fraction-like
  const children = parseMath(inner);
  return { children, next: i };
}

function parseMath(tokens: Token[]): MathChild[] {
  const result: MathChild[] = [];
  let i = 0;
  while (i < tokens.length) {
    const t = tokens[i];

    // Handle ^ superscript
    if (t.kind === 'char' && t.val === '^') {
      const base = result.pop();
      const sup = nextArg(tokens, i + 1);
      i = sup.next;
      if (base instanceof MathSubScript) {
        // Can't easily combine; just push superscript on base
        result.push(new MathSuperScript({ children: [base], superScript: sup.children }));
      } else {
        result.push(new MathSuperScript({ children: base ? [base] : [], superScript: sup.children }));
      }
      continue;
    }

    // Handle _ subscript
    if (t.kind === 'char' && t.val === '_') {
      const base = result.pop();
      const subResult = nextArg(tokens, i + 1);
      i = subResult.next;
      if (tokens[i]?.kind === 'char' && tokens[i].val === '^') {
        const sup = nextArg(tokens, i + 1);
        i = sup.next;
        result.push(new MathSubSuperScript({ children: base ? [base] : [], subScript: subResult.children, superScript: sup.children }));
      } else {
        result.push(new MathSubScript({ children: base ? [base] : [], subScript: subResult.children }));
      }
      continue;
    }

    const { children, next } = parseToken(tokens, i);
    result.push(...children);
    i = next;
  }
  return result;
}

function latexMathToDocx(tex: string, display: boolean): Paragraph {
  const tokens = tokenise(tex.trim());
  const children = parseMath(tokens);
  return new Paragraph({
    children: [new DocxMath({ children })],
    alignment: display ? AlignmentType.CENTER : undefined,
    spacing: display ? { before: 120, after: 120 } : undefined,
  });
}

// ─── Inline text parser ───────────────────────────────────────────────────────

interface InlineOpts { bold?: boolean; italic?: boolean; underline?: boolean; code?: boolean }

type ParagraphRun = TextRun | DocxMath;

function parseInline(src: string, opts: InlineOpts = {}): ParagraphRun[] {
  const runs: ParagraphRun[] = [];
  const re = /\\textbf\{([^}]*)}|\\textit\{([^}]*)}|\\emph\{([^}]*)}|\\underline\{([^}]*)}|\\texttt\{([^}]*)}|\\text\{([^}]*)}|\\\((.+?)\\\)|\$([^$\n]{1,300}?)\$|\\LaTeX\b|\\TeX\b|\\today\b/gs;
  let last = 0, m: RegExpExecArray | null;
  while ((m = re.exec(src)) !== null) {
    if (m.index > last) runs.push(makeRun(src.slice(last, m.index), opts));
    if (m[1] !== undefined) runs.push(...parseInline(m[1], { ...opts, bold: true }));
    else if (m[2] !== undefined) runs.push(...parseInline(m[2], { ...opts, italic: true }));
    else if (m[3] !== undefined) runs.push(...parseInline(m[3], { ...opts, italic: true }));
    else if (m[4] !== undefined) runs.push(...parseInline(m[4], { ...opts, underline: true }));
    else if (m[5] !== undefined) runs.push(...parseInline(m[5], { ...opts, code: true }));
    else if (m[6] !== undefined) runs.push(makeRun(m[6], opts));
    else if (m[7] !== undefined) runs.push(new DocxMath({ children: parseMath(tokenise(m[7])) })); // \(...\)
    else if (m[8] !== undefined) runs.push(new DocxMath({ children: parseMath(tokenise(m[8])) })); // $...$
    else if (m[0] === '\\LaTeX') runs.push(makeRun('LaTeX', opts));
    else if (m[0] === '\\TeX') runs.push(makeRun('TeX', opts));
    else if (m[0] === '\\today') runs.push(makeRun(new Date().toLocaleDateString('en-US', { year:'numeric', month:'long', day:'numeric' }), opts));
    last = re.lastIndex;
  }
  if (last < src.length) runs.push(makeRun(src.slice(last), opts));
  return runs;
}

function makeRun(text: string, opts: InlineOpts): TextRun {
  const cleaned = text
    .replace(/---/g,'\u2014').replace(/--/g,'\u2013')
    .replace(/``/g,'\u201C').replace(/''/g,'\u201D')
    .replace(/~/g,'\u00A0').replace(/\\%/g,'%')
    .replace(/\\&/g,'&').replace(/\\\$/g,'$')
    .replace(/\\#/g,'#').replace(/\\_/g,'_')
    .replace(/\\\{/g,'{').replace(/\\\}/g,'}')
    .replace(/\\qquad\b/g,'  ').replace(/\\quad\b/g,' ')
    .replace(/\n/g,' ');
  return new TextRun({
    text: cleaned,
    bold: opts.bold,
    italics: opts.italic,
    underline: opts.underline ? { type: 'single' } : undefined,
    font: opts.code ? 'Courier New' : 'Georgia',
    size: opts.code ? 20 : undefined,
  });
}

// ─── Block / document parser ──────────────────────────────────────────────────

type DocBlock = Paragraph | Table;

function stripComments(src: string): string { return src.replace(/%[^\n]*/g, ''); }

function extractMeta(src: string): { title: string; author: string; date: string; body: string } {
  let title = '', author = '', date = '';
  src = src.replace(/\\title\{([^}]*)}/g, (_, t) => { title = t; return ''; });
  src = src.replace(/\\author\{([^}]*)}/g, (_, a) => { author = a; return ''; });
  src = src.replace(/\\date\{([^}]*)}/g, (_, d) => {
    date = d === '\\today' ? new Date().toLocaleDateString('en-US',{year:'numeric',month:'long',day:'numeric'}) : d;
    return '';
  });
  const dm = src.match(/\\begin\{document}([\s\S]*?)\\end\{document}/);
  return { title, author, date, body: dm ? dm[1] : src };
}

function parseTabular(content: string): Table {
  // Strip \hline first (including mid-row occurrences) and collapse
  // trailing whitespace so row splitting on \\ works cleanly.
  const cleaned = content
    .replace(/\\hline\b/g, '')
    .replace(/[ \t]+\n/g, '\n');
  const rows = cleaned.split(/\\\\/).map(r=>r.trim()).filter(Boolean);
  return new Table({
    width: { size: 100, type: WidthType.PERCENTAGE },
    rows: rows.map((row, ri) => {
      // Preserve inline formatting (\textbf, math, ...) in every cell,
      // including the header row, so a header like $x$ renders as math.
      const cells = row.split('&').map(cell => {
        const cleaned = cell.trim();
        return new TableCell({
          children: [new Paragraph({
            children: parseInline(cleaned, { bold: ri === 0 }),
          })],
          borders: {
            top:{style:BorderStyle.SINGLE,size:1},
            bottom:{style:BorderStyle.SINGLE,size:1},
            left:{style:BorderStyle.SINGLE,size:1},
            right:{style:BorderStyle.SINGLE,size:1},
          },
        });
      });
      return new TableRow({ children: cells });
    }),
  });
}

function parseItems(content: string, ordered: boolean): Paragraph[] {
  return content.split(/\\item\b/).filter(s => s.trim()).map(item =>
    new Paragraph({
      children: parseInline(item.trim()),
      numbering: { reference: ordered ? 'ordered-list' : 'bullet-list', level: 0 },
    })
  );
}

function parseBody(body: string): DocBlock[] {
  const blocks: DocBlock[] = [];

  // Strip line-leading preamble AND any inline \maketitle so the DOCX
  // export doesn't leak a literal "\maketitle" string into a paragraph.
  body = body.replace(/\\(maketitle|documentclass|usepackage|geometry|pagestyle|thispagestyle|setlength|renewcommand|newcommand|newtheorem|theoremstyle|DeclareMathOperator|tableofcontents|appendix|centering|raggedright|raggedleft|normalsize|small|large|Large|LARGE|huge|Huge|tiny|footnotesize|normalfont|rmfamily|sffamily|ttfamily|bfseries|itshape)\b[^\n]*/g, '');
  body = body.replace(/\\maketitle\b/g, '');

  // Save display math before line splitting
  const mathBlocks: { tex: string; display: boolean }[] = [];
  function saveMath(tex: string, display: boolean) {
    const id = `__MATH${mathBlocks.length}__`;
    mathBlocks.push({ tex: tex.trim(), display });
    return `\n${id}\n`;
  }
  body = body.replace(/\\begin\{equation\*?}([\s\S]*?)\\end\{equation\*?}/g, (_,c) => saveMath(c, true));
  body = body.replace(/\\begin\{align\*?}([\s\S]*?)\\end\{align\*?}/g, (_,c) => saveMath(c, true));
  body = body.replace(/\\begin\{gather\*?}([\s\S]*?)\\end\{gather\*?}/g, (_,c) => saveMath(c, true));
  body = body.replace(/\\begin\{multline\*?}([\s\S]*?)\\end\{multline\*?}/g, (_,c) => saveMath(c, true));
  body = body.replace(/\\\[([\s\S]*?)\\\]/g, (_,c) => saveMath(c, true));
  body = body.replace(/\$\$([\s\S]*?)\$\$/g, (_,c) => saveMath(c, true));

  const lines = body.split('\n');
  let buffer = '';

  function flushBuffer() {
    const t = buffer.trim();
    buffer = '';
    if (!t) return;
    const mathRef = t.match(/^__MATH(\d+)__$/);
    if (mathRef) {
      const { tex, display } = mathBlocks[+mathRef[1]];
      // For aligned envs, split on \\ and render each line
      const lines = tex.split(/\\\\/).map(l => l.trim()).filter(Boolean);
      for (const line of lines) {
        blocks.push(latexMathToDocx(line, display));
      }
      return;
    }
    blocks.push(new Paragraph({ children: parseInline(t), spacing: { after: 120 } }));
  }

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    const secMatch = line.match(/^\\(chapter|section|subsection|subsubsection)\*?\{([^}]*)}/);
    if (secMatch) {
      flushBuffer();
      const lvlMap: Record<string, typeof HeadingLevel[keyof typeof HeadingLevel]> = {
        chapter: HeadingLevel.HEADING_1, section: HeadingLevel.HEADING_1,
        subsection: HeadingLevel.HEADING_2, subsubsection: HeadingLevel.HEADING_3,
      };
      blocks.push(new Paragraph({ text: secMatch[2], heading: lvlMap[secMatch[1]], spacing: { before: 240, after: 120 } }));
      continue;
    }

    const beginMatch = line.match(/^\\begin\{(\w+)}(.*)$/);
    if (beginMatch) {
      const env = beginMatch[1];
      const endRe = new RegExp(`\\\\end\\{${env}}`);
      const envLines: string[] = [beginMatch[2]];
      i++;
      while (i < lines.length && !endRe.test(lines[i])) { envLines.push(lines[i]); i++; }
      const content = envLines.join('\n');
      flushBuffer();
      if (env === 'itemize') blocks.push(...parseItems(content, false));
      else if (env === 'enumerate') blocks.push(...parseItems(content, true));
      else if (env === 'tabular') blocks.push(parseTabular(content));
      else if (env === 'verbatim') blocks.push(new Paragraph({ children: [new TextRun({ text: content, font: 'Courier New', size: 18 })], spacing: { before: 120, after: 120 } }));
      else if (env === 'abstract') blocks.push(new Paragraph({ children: [new TextRun({ text: 'Abstract. ', bold: true }), ...parseInline(content.trim())], indent: { left: 720, right: 720 }, spacing: { before: 120, after: 240 } }));
      else { const t = content.trim(); if (t) blocks.push(new Paragraph({ children: parseInline(t) })); }
      continue;
    }

    if (line.trim() === '') { flushBuffer(); continue; }
    buffer += (buffer ? ' ' : '') + line;
  }
  flushBuffer();
  return blocks;
}

// ─── Public API ───────────────────────────────────────────────────────────────

export async function latexToDocxBlob(source: string): Promise<Blob> {
  source = stripComments(source);
  const { title, author, date, body } = extractMeta(source);

  const titleBlocks: Paragraph[] = [];
  if (title) titleBlocks.push(new Paragraph({ text: title, heading: HeadingLevel.TITLE, alignment: AlignmentType.CENTER }));
  if (author) titleBlocks.push(new Paragraph({ children: [new TextRun({ text: author, size: 24 })], alignment: AlignmentType.CENTER }));
  if (date) titleBlocks.push(new Paragraph({ children: [new TextRun({ text: date, size: 22, color: '555555' })], alignment: AlignmentType.CENTER }));
  if (titleBlocks.length) titleBlocks.push(new Paragraph({ border: { bottom: { style: BorderStyle.SINGLE, size: 1, color: 'DDDDDD', space: 1 } }, spacing: { after: 240 } }));

  const doc = new Document({
    numbering: {
      config: [
        { reference: 'bullet-list', levels: [{ level: 0, format: LevelFormat.BULLET, text: '\u2022', alignment: AlignmentType.LEFT, style: { paragraph: { indent: { left: 720, hanging: 360 } } } }] },
        { reference: 'ordered-list', levels: [{ level: 0, format: LevelFormat.DECIMAL, text: '%1.', alignment: AlignmentType.LEFT, style: { paragraph: { indent: { left: 720, hanging: 360 } } } }] },
      ],
    },
    styles: {
      default: { document: { run: { font: 'Georgia', size: 24 } } },
    },
    sections: [{ children: [...titleBlocks, ...parseBody(body)] }],
  });

  return Packer.toBlob(doc);
}
