use chrono::Local;

#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Underline(Vec<Inline>),
    Code(String),
    InlineMath(String),
    /// `\href{url}{text}` — clickable link in the output.
    Hyperlink { url: String, label: Vec<Inline> },
    /// `\url{url}` — bare URL rendered as the URL itself but clickable.
    Url(String),
    /// Reference into `Document::footnotes` by id.
    FootnoteRef(usize),
}

#[derive(Debug)]
pub enum Block {
    MakeTitle,
    HRule,
    Section { level: u8, title: String },
    Para(Vec<Inline>),
    List { ordered: bool, items: Vec<Vec<Inline>> },
    Table { rows: Vec<Vec<String>> },
    DisplayMath(String),
    Verbatim(String),
    /// `\caption{...}` — rendered as centered italic text below a
    /// figure/table placeholder. The parser emits a placeholder block
    /// before the caption so authors can position them correctly.
    Caption(Vec<Inline>),
    /// A footnote body, keyed by the same id referenced via
    /// `Inline::FootnoteRef`. Collected during parsing and emitted
    /// into `word/footnotes.xml` by the DOCX writer.
    Footnote { id: usize, text: Vec<Inline> },
}

pub struct Document {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub blocks: Vec<Block>,
    /// Footnote bodies, indexed by `Inline::FootnoteRef`. Insertion
    /// order preserves the order of `\footnote{...}` commands in the
    /// source.
    pub footnotes: Vec<(usize, Vec<Inline>)>,
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let mut esc = false;
        let mut cut = line.len();
        for (i, c) in line.char_indices() {
            if esc { esc = false; continue; }
            if c == '\\' { esc = true; continue; }
            if c == '%' { cut = i; break; }
        }
        out.push_str(&line[..cut]);
        out.push('\n');
    }
    out
}

/// Read the content of a `{...}` block starting at position `pos` in `s`.
/// Returns (content, byte_position_after_closing_brace).
fn take_braced(s: &str, pos: usize) -> Option<(String, usize)> {
    let bytes = s.as_bytes();
    if pos >= bytes.len() || bytes[pos] != b'{' { return None; }
    let mut depth = 0usize;
    let mut i = pos;
    let ch: Vec<char> = s[pos..].chars().collect();
    let mut char_pos = pos;
    for c in &ch {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let inner = s[pos + 1..char_pos].to_string();
                    return Some((inner, char_pos + c.len_utf8()));
                }
            }
            _ => {}
        }
        char_pos += c.len_utf8();
    }
    None
}

fn scan_cmd_arg(src: &str, cmd: &str) -> Option<String> {
    let needle = format!("\\{}", cmd);
    let mut search = src;
    let mut offset = 0;
    while let Some(rel) = search.find(&needle) {
        let abs = offset + rel;
        let after_cmd = abs + needle.len();
        let rest = src[after_cmd..].trim_start();
        let trimmed_offset = after_cmd + (src[after_cmd..].len() - rest.len());
        if let Some((content, _)) = take_braced(src, trimmed_offset) {
            return Some(content);
        }
        offset += rel + 1;
        search = &src[offset..];
    }
    None
}

fn today() -> String {
    Local::now().format("%B %-d, %Y").to_string()
}

// ── inline parser ─────────────────────────────────────────────────────────────

pub fn parse_inline(src: &str, footnotes: &mut Vec<(usize, Vec<Inline>)>) -> Vec<Inline> {
    let mut result: Vec<Inline> = Vec::new();
    let mut buf = String::new();
    let ch: Vec<char> = src.chars().collect();
    let mut i = 0;

    macro_rules! flush {
        () => {
            if !buf.is_empty() {
                result.push(Inline::Text(std::mem::take(&mut buf)));
            }
        };
    }

    while i < ch.len() {
        // em/en dash
        if ch[i] == '-' {
            if i + 2 < ch.len() && ch[i+1] == '-' && ch[i+2] == '-' {
                buf.push('\u{2014}'); i += 3; continue;
            }
            if i + 1 < ch.len() && ch[i+1] == '-' {
                buf.push('\u{2013}'); i += 2; continue;
            }
        }
        // smart quotes
        if ch[i] == '`' && i + 1 < ch.len() && ch[i+1] == '`' {
            buf.push('\u{201C}'); i += 2; continue;
        }
        if ch[i] == '\'' && i + 1 < ch.len() && ch[i+1] == '\'' {
            buf.push('\u{201D}'); i += 2; continue;
        }
        // ~
        if ch[i] == '~' { buf.push('\u{00A0}'); i += 1; continue; }

        // inline math $...$
        if ch[i] == '$' {
            // check $$
            if i + 1 < ch.len() && ch[i+1] == '$' {
                // skip display math (handled at block level)
                i += 2;
                while i < ch.len() {
                    if ch[i] == '$' && i + 1 < ch.len() && ch[i+1] == '$' { i += 2; break; }
                    i += 1;
                }
                continue;
            }
            // Honour a backslash-escaped \$ so a literal $ in prose
            // (e.g. "price \$5") does not get swallowed by the
            // math delimiter.
            if i > 0 && ch[i - 1] == '\\' {
                // The escape itself was consumed by the general
                // command handler below; treat this $ as a literal.
                buf.push('$');
                i += 1;
                continue;
            }
            flush!();
            i += 1;
            let start = i;
            // Greedy scan: an inline $ ends at the next un-escaped $.
            // Previously this was limited to ~300 chars which silently
            // broke long equations; there is no LaTeX-level limit.
            while i < ch.len() {
                if ch[i] == '\\' && i + 1 < ch.len() && (ch[i + 1] == '$' || ch[i + 1] == '\\') {
                    i += 2;
                    continue;
                }
                if ch[i] == '$' { break; }
                i += 1;
            }
            let math: String = ch[start..i].iter().collect();
            result.push(Inline::InlineMath(math));
            if i < ch.len() { i += 1; } // skip closing $
            continue;
        }

        // commands
        if ch[i] == '\\' && i + 1 < ch.len() {
            // escaped chars
            match ch[i+1] {
                '%' => { buf.push('%'); i += 2; continue; }
                '&' => { buf.push('&'); i += 2; continue; }
                '$' => { buf.push('$'); i += 2; continue; }
                '#' => { buf.push('#'); i += 2; continue; }
                '_' => { buf.push('_'); i += 2; continue; }
                '{' => { buf.push('{'); i += 2; continue; }
                '}' => { buf.push('}'); i += 2; continue; }
                '\\' => { buf.push('\n'); i += 2; continue; }
                ' ' => { buf.push(' '); i += 2; continue; }
                _ => {}
            }

            // read command name
            let mut j = i + 1;
            if ch[j].is_alphabetic() {
                while j < ch.len() && ch[j].is_alphabetic() { j += 1; }
                while j < ch.len() && ch[j] == ' ' { j += 1; }
                let cmd: String = ch[i+1..j.min(ch.len())].iter().collect();
                // check for alphabetic end
                let cmd_end = j;

                match cmd.as_str() {
                    "textbf" | "mathbf" => {
                        flush!();
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((inner, _)) = take_braced(&rest, 0) {
                            result.push(Inline::Bold(parse_inline(&inner, footnotes)));
                            i = cmd_end + 1 + inner.len() + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "textit" | "textsl" | "emph" | "mathit" => {
                        flush!();
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((inner, _)) = take_braced(&rest, 0) {
                            result.push(Inline::Italic(parse_inline(&inner, footnotes)));
                            i = cmd_end + 1 + inner.len() + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "underline" => {
                        flush!();
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((inner, _)) = take_braced(&rest, 0) {
                            result.push(Inline::Underline(parse_inline(&inner, footnotes)));
                            i = cmd_end + 1 + inner.len() + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "texttt" | "verb" | "textsc" => {
                        flush!();
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((inner, _)) = take_braced(&rest, 0) {
                            result.push(Inline::Code(inner.clone()));
                            i = cmd_end + 1 + inner.len() + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "text" | "textrm" | "textsf" => {
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((inner, _)) = take_braced(&rest, 0) {
                            buf.push_str(&inner);
                            i = cmd_end + 1 + inner.len() + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "LaTeX" => { buf.push_str("LaTeX"); i = cmd_end; continue; }
                    "TeX"   => { buf.push_str("TeX");   i = cmd_end; continue; }
                    "today" => { buf.push_str(&today()); i = cmd_end; continue; }
                    "footnote" => {
                        // Register the footnote body and emit a FootnoteRef
                        // placeholder. The DOCX writer renders a real
                        // <w:footnoteReference> pointing at word/footnotes.xml.
                        flush!();
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((inner, _)) = take_braced(&rest, 0) {
                            let id = footnotes.len() + 1;
                            let text = parse_inline(&inner, footnotes);
                            footnotes.push((id, text));
                            result.push(Inline::FootnoteRef(id));
                            i = cmd_end + 1 + inner.len() + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "href" => {
                        // \href{url}{text} — emits Inline::Hyperlink.
                        flush!();
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((url, after)) = take_braced(&rest, 0) {
                            let after_str: String = ch[cmd_end + 1 + url.len() + 1..].iter().collect();
                            if let Some((label, _)) = take_braced(&after_str, 0) {
                                let url_len = url.len();
                                let label_len = label.len();
                                result.push(Inline::Hyperlink {
                                    url,
                                    label: parse_inline(&label, footnotes),
                                });
                                // Advance past both braced args.
                                let total = 1 + url_len + 1 + 1 + label_len + 1;
                                i = cmd_end + total;
                            } else {
                                let url_len = url.len();
                                i = cmd_end + 1 + url_len + 1 + after;
                            }
                        } else { i = cmd_end; }
                        continue;
                    }
                    "url" => {
                        // \url{url} — bare URL rendered as text.
                        flush!();
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((url, _)) = take_braced(&rest, 0) {
                            let url_len = url.len();
                            result.push(Inline::Url(url));
                            i = cmd_end + 1 + url_len + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "label" | "ref" | "cite" | "vspace" | "hspace"
                    | "noindent" | "par" | "newline" | "linebreak"
                    | "centering" | "raggedright" | "raggedleft" => {
                        // skip with optional braced arg
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((inner, _)) = take_braced(&rest, 0) {
                            i = cmd_end + 1 + inner.len() + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "quad"  => { buf.push('\u{2003}'); i = cmd_end; continue; }
                    "qquad" => { buf.push_str("\u{2003}\u{2003}"); i = cmd_end; continue; }
                    _ => { i = cmd_end; continue; }
                }
            }
        }

        buf.push(ch[i]);
        i += 1;
    }
    flush!();
    result
}

fn inner_len(src: &str, pos: usize) -> usize {
    if let Some((inner, after)) = take_braced(src, pos) {
        1 + inner.len() + 1
    } else { 0 }
}

// ── block parser ──────────────────────────────────────────────────────────────

/// Returns true if the line looks like a tabular column specification,
/// e.g. `{|l|c|r|p{3cm}|}`. We use this to filter stray spec lines
/// out of the body if the spec was placed on its own line inside
/// `\begin{tabular}{...}`.
fn looks_like_column_spec(line: &str) -> bool {
    let t = line.trim();
    // Use strip_prefix/strip_suffix (not trim_matches) so we don't
    // accidentally strip the closing `}` from a `p{3cm}` token.
    let after_open = match t.strip_prefix('{').or_else(|| t.strip_prefix('|')) {
        Some(s) => s,
        None => return false,
    };
    let before_close = match after_open.strip_suffix('}').or_else(|| after_open.strip_suffix('|')) {
        Some(s) => s,
        None => return false,
    };
    // A column spec consists of column tokens separated by `|`. Each
    // token is either a single letter (l/c/r) or `p{...}` / `m{...}`
    // / `b{...}` / `*{...}` with a width argument.
    before_close.split('|').all(|tok| {
        let tok = tok.trim();
        matches!(tok, "l" | "c" | "r" | "")
            || (tok.starts_with("p{") && tok.ends_with('}'))
            || (tok.starts_with("m{") && tok.ends_with('}'))
            || (tok.starts_with("b{") && tok.ends_with('}'))
            || tok.starts_with("*{")
    })
}

fn is_preamble(line: &str) -> bool {
    for cmd in &[
        "\\documentclass", "\\usepackage", "\\geometry",
        "\\pagestyle", "\\thispagestyle", "\\setlength",
        "\\renewcommand", "\\newcommand", "\\newtheorem",
        "\\theoremstyle", "\\DeclareMathOperator", "\\providecommand",
        "\\RequirePackage", "\\setcounter", "\\pagenumbering",
        "\\bibliographystyle", "\\bibliography", "\\printbibliography",
    ] {
        if line.trim_start().starts_with(cmd) { return true; }
    }
    false
}

fn parse_section(line: &str) -> Option<Block> {
    let line = line.trim_start();
    let (level, rest) = if line.starts_with("\\chapter") { (1u8, &line[8..]) }
        else if line.starts_with("\\section") { (1, &line[8..]) }
        else if line.starts_with("\\subsection") { (2, &line[11..]) }
        else if line.starts_with("\\subsubsection") { (3, &line[14..]) }
        else { return None; };
    let rest = rest.trim_start_matches('*').trim_start();
    if let Some((title, _)) = take_braced(rest, 0) {
        Some(Block::Section { level, title })
    } else { None }
}

fn parse_env_begin(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("\\begin{") { return None; }
    // `take_braced` expects the opening `{` to be at `pos`, so we
    // want position 6 (one past `\begin`, where `{` lives) — not 7
    // (which is one past the `{`, inside the env name).
    if let Some((env, _)) = take_braced(line, 6) { Some(env) } else { None }
}

fn collect_env<'a>(env: &str, lines: &[&'a str], start: usize) -> (String, usize) {
    let end_tag = format!("\\end{{{}}}", env);
    let mut body = String::new();
    let mut i = start;
    while i < lines.len() {
        let l = lines[i];
        if l.trim().starts_with(&end_tag) { i += 1; break; }
        body.push_str(l);
        body.push('\n');
        i += 1;
    }
    (body, i)
}

/// Like `collect_env` but also handles the single-line case
/// `\begin{env}body\end{env}`. If the `\begin` line also contains the
/// closing `\end`, the body is extracted from that line and the
/// returned `next` line index points at the line after the `\begin`
/// line (so the caller doesn't re-scan it).
fn collect_env_or_inline(env: &str, begin_line: &str, lines: &[&str], start: usize) -> (String, usize) {
    let end_tag = format!("\\end{{{}}}", env);
    // First, look for a same-line end on the \begin line.
    if let Some(end_pos) = begin_line.find(&end_tag) {
        // Everything between `\begin{env}` and `\end{env}` on the
        // same line. The `\begin{env}` prefix is 7 + env.len() chars
        // (`\` + `begin` + `{` + `env` + `}`).
        let prefix_len = "\\begin{".len() + env.len() + 1; // +1 for closing `}`
        let body = begin_line[prefix_len..end_pos].trim().to_string();
        // Caller advanced past `begin_line` already (i += 1), so
        // return the line after it.
        return (body, start + 1);
    }
    collect_env(env, lines, start)
}

fn parse_itemize(content: &str, footnotes: &mut Vec<(usize, Vec<Inline>)>) -> Vec<Vec<Inline>> {
    content.split("\\item")
           .skip(1) // first split is before first \item
           .map(|item| {
               let text = item.trim();
               // remove optional [label]
               let text = if text.starts_with('[') {
                   if let Some(end) = text.find(']') { &text[end+1..] } else { text }
               } else { text };
               parse_inline(text.trim(), footnotes)
           })
           .filter(|v| !v.is_empty())
           .collect()
}

fn parse_tabular(content: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let content = content
        .lines()
        // Drop \hline and empty lines, plus any stray column-spec
        // tokens that landed in the body (e.g. when the spec was
        // placed on its own line).
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("\\hline") && !t.is_empty() && !looks_like_column_spec(t)
        })
        .collect::<Vec<_>>()
        .join("\n");

    for row_str in content.split("\\\\") {
        let row_str = row_str.trim();
        if row_str.is_empty() { continue; }
        // A stray mid-row \hline would otherwise produce a junk cell.
        let row_str = row_str.replace("\\hline", "");
        let cells: Vec<String> = row_str.split('&').map(|c| c.trim().to_string()).collect();
        if !cells.is_empty() && cells.iter().any(|c| !c.is_empty()) {
            rows.push(cells);
        }
    }
    rows
}

fn flush_para(buf: &mut Vec<String>, blocks: &mut Vec<Block>, footnotes: &mut Vec<(usize, Vec<Inline>)>) {
    let text: String = buf.join(" ");
    let text = text.trim().to_string();
    if !text.is_empty() {
        blocks.push(Block::Para(parse_inline(&text, footnotes)));
    }
    buf.clear();
}

fn parse_body(body: &str, footnotes: &mut Vec<(usize, Vec<Inline>)>) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    let mut para_buf: Vec<String> = Vec::new();

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            flush_para(&mut para_buf, &mut blocks, footnotes);
            i += 1;
            continue;
        }

        if is_preamble(trimmed) { i += 1; continue; }

        if trimmed.starts_with("\\maketitle") {
            flush_para(&mut para_buf, &mut blocks, footnotes);
            blocks.push(Block::MakeTitle);
            i += 1; continue;
        }

        if trimmed.starts_with("\\hrule") || trimmed.starts_with("\\noindent\\hrule") {
            flush_para(&mut para_buf, &mut blocks, footnotes);
            blocks.push(Block::HRule);
            i += 1; continue;
        }

        if trimmed.starts_with("\\newpage") || trimmed.starts_with("\\clearpage") {
            flush_para(&mut para_buf, &mut blocks, footnotes);
            blocks.push(Block::HRule);
            i += 1; continue;
        }

        if let Some(sec) = parse_section(trimmed) {
            flush_para(&mut para_buf, &mut blocks, footnotes);
            blocks.push(sec);
            i += 1; continue;
        }

        // \begin{env}
        if let Some(env) = parse_env_begin(trimmed) {
            flush_para(&mut para_buf, &mut blocks, footnotes);
            // Check if end is on same line
            let rest = &trimmed[trimmed.find('}').map(|p| p+1).unwrap_or(trimmed.len())..];
            let same_line_content = rest.trim();
            i += 1;

            match env.as_str() {
                "document" => { /* skip */ }
                "equation" | "equation*" | "displaymath" => {
                    let (body, next) = collect_env(&env, &lines, i);
                    blocks.push(Block::DisplayMath(body.trim().to_string()));
                    i = next;
                }
                "align" | "align*" | "aligned" | "gather" | "gather*"
                | "multline" | "multline*" | "eqnarray" | "eqnarray*" => {
                    let (body, next) = collect_env(&env, &lines, i);
                    // Split aligned rows
                    for row in body.split("\\\\") {
                        let row = row.trim().trim_end_matches('&').trim();
                        // Remove alignment markers & from align
                        let row = row.replace('&', "");
                        let row = row.trim().to_string();
                        if !row.is_empty() {
                            blocks.push(Block::DisplayMath(row));
                        }
                    }
                    i = next;
                }
                "itemize" | "compactitem" => {
                    let (body, next) = collect_env(&env, &lines, i);
                    let items = parse_itemize(&body, footnotes);
                    blocks.push(Block::List { ordered: false, items });
                    i = next;
                }
                "enumerate" | "compactenum" => {
                    let (body, next) = collect_env(&env, &lines, i);
                    let items = parse_itemize(&body, footnotes);
                    blocks.push(Block::List { ordered: true, items });
                    i = next;
                }
                "tabular" | "tabularx" | "longtable" => {
                    // skip column spec
                    let (body, next) = collect_env(&env, &lines, i);
                    let rows = parse_tabular(&body);
                    blocks.push(Block::Table { rows });
                    i = next;
                }
"verbatim" | "lstlisting" | "minted" | "Verbatim" => {
                    let (body, next) = collect_env(&env, &lines, i);
                    blocks.push(Block::Verbatim(body.trim_end().to_string()));
                    i = next;
                }
                "abstract" => {
                    let (body, next) = collect_env(&env, &lines, i);
                    blocks.push(Block::Para(parse_inline(body.trim(), footnotes)));
                    i = next;
                }
                "theorem" | "lemma" | "definition" | "proposition"
                | "corollary" | "conjecture" | "claim" | "fact" => {
                    // Render the environment as a paragraph whose first
                    // run is a bold, italic "Theorem." / "Lemma." /
                    // etc. label. The exact label capitalisation
                    // matches amsmath's default style.
                    let (body, next) = collect_env_or_inline(&env, trimmed, &lines, i);
                    let label = match env.as_str() {
                        "theorem"     => "Theorem.",
                        "lemma"       => "Lemma.",
                        "definition"  => "Definition.",
                        "proposition" => "Proposition.",
                        "corollary"   => "Corollary.",
                        "conjecture"  => "Conjecture.",
                        "claim"       => "Claim.",
                        "fact"        => "Fact.",
                        _ => unreachable!(),
                    };
                    let body = body.trim();
                    let inlines = if body.is_empty() {
                        vec![Inline::Text(label.to_string())]
                    } else {
                        let mut out = Vec::new();
                        out.push(Inline::Bold(vec![Inline::Italic(vec![
                            Inline::Text(label.to_string()),
                        ])]));
                        out.push(Inline::Text(" ".to_string()));
                        out.extend(parse_inline(body, footnotes));
                        out
                    };
                    blocks.push(Block::Para(inlines));
                    i = next;
                }
                "proof" => {
                    // Render as italic body followed by an "□"
                    // (QED) marker. Matches amsmath's \qedhere when
                    // the proof ends on its own line.
                    let (body, next) = collect_env_or_inline(&env, trimmed, &lines, i);
                    let body = body.trim();
                    let mut inlines = Vec::new();
                    if !body.is_empty() {
                        inlines.push(Inline::Italic(parse_inline(body, footnotes)));
                        inlines.push(Inline::Text(" ".to_string()));
                    }
                    inlines.push(Inline::Text("\u{25A1}".to_string())); // □
                    blocks.push(Block::Para(inlines));
                    i = next;
                }
                "figure" | "table" | "wrapfigure" => {
                    // skip figures
                    let (_, next) = collect_env(&env, &lines, i);
                    i = next;
                }
                _ => {
                    // Unknown environment — treat as paragraph content
                    let (body, next) = collect_env(&env, &lines, i);
                    para_buf.push(body);
                    i = next;
                }
            }
            continue;
        }

        // \caption{...} — block-level caption above/below a figure/table
        if trimmed.starts_with("\\caption{") {
            flush_para(&mut para_buf, &mut blocks, footnotes);
            if let Some((inner, _)) = take_braced(trimmed, "\\caption".len()) {
                blocks.push(Block::Caption(parse_inline(&inner, footnotes)));
            }
            i += 1; continue;
        }

        // \[ display math \]
        if trimmed.starts_with("\\[") {
            flush_para(&mut para_buf, &mut blocks, footnotes);
            let mut math = trimmed[2..].to_string();
            if let Some(end) = math.find("\\]") {
                math.truncate(end);
                blocks.push(Block::DisplayMath(math.trim().to_string()));
                i += 1;
            } else {
                i += 1;
                while i < lines.len() {
                    let l = lines[i].trim();
                    if let Some(end) = l.find("\\]") {
                        math.push(' ');
                        math.push_str(&l[..end]);
                        i += 1;
                        break;
                    }
                    math.push(' ');
                    math.push_str(l);
                    i += 1;
                }
                blocks.push(Block::DisplayMath(math.trim().to_string()));
            }
            continue;
        }

        // $$ display $$
        if trimmed.starts_with("$$") {
            flush_para(&mut para_buf, &mut blocks, footnotes);
            let rest = &trimmed[2..];
            if let Some(end) = rest.find("$$") {
                blocks.push(Block::DisplayMath(rest[..end].trim().to_string()));
                i += 1;
            } else {
                let mut math = rest.to_string();
                i += 1;
                while i < lines.len() {
                    let l = lines[i].trim();
                    if let Some(end) = l.find("$$") {
                        math.push(' ');
                        math.push_str(&l[..end]);
                        i += 1;
                        break;
                    }
                    math.push(' ');
                    math.push_str(l);
                    i += 1;
                }
                blocks.push(Block::DisplayMath(math.trim().to_string()));
            }
            continue;
        }

        para_buf.push(line.to_string());
        i += 1;
    }

    flush_para(&mut para_buf, &mut blocks, footnotes);
    blocks
}

// ── public ────────────────────────────────────────────────────────────────────

impl Document {
    pub fn parse(source: &str) -> Self {
        let src = strip_comments(source);

        let title  = scan_cmd_arg(&src, "title");
        let author = scan_cmd_arg(&src, "author");
        let date = scan_cmd_arg(&src, "date").map(|d| {
            if d.trim() == "\\today" { today() } else { d }
        });

        let body = {
            let tag = "\\begin{document}";
            if let Some(s) = src.find(tag) {
                let after = &src[s + tag.len()..];
                if let Some(e) = after.find("\\end{document}") {
                    &after[..e]
                } else { after }
            } else { &src }
        };

        let mut footnotes: Vec<(usize, Vec<Inline>)> = Vec::new();
        let blocks = parse_body(body, &mut footnotes);
        Document { title, author, date, blocks, footnotes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_only(src: &str) -> (Vec<Block>, Vec<(usize, Vec<Inline>)>) {
        let mut footnotes: Vec<(usize, Vec<Inline>)> = Vec::new();
        let blocks = parse_body(src, &mut footnotes);
        (blocks, footnotes)
    }

    /// Recursively walks an inline list looking for a Text node that
    /// contains the needle.
    fn has_text(block: &Block, needle: &str) -> bool {
        match block {
            Block::Para(inlines) => inlines.iter().any(|n| text_contains(n, needle)),
            _ => false,
        }
    }

    fn text_contains(n: &Inline, needle: &str) -> bool {
        match n {
            Inline::Text(t) => t.contains(needle),
            Inline::Bold(c) | Inline::Italic(c) | Inline::Underline(c) => {
                c.iter().any(|x| text_contains(x, needle))
            }
            _ => false,
        }
    }

    #[test]
    fn caption_is_block() {
        let (blocks, _) = parse_only(r"\caption{Hello world}");
        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0], Block::Caption(_)));
    }

    #[test]
    fn href_yields_hyperlink_inline() {
        let mut footnotes: Vec<(usize, Vec<Inline>)> = Vec::new();
        let inlines = parse_inline(r"\href{https://example.org}{click me}", &mut footnotes);
        assert!(matches!(inlines[0], Inline::Hyperlink { ref url, .. } if url == "https://example.org"));
    }

    #[test]
    fn url_yields_bare_url_inline() {
        let mut footnotes: Vec<(usize, Vec<Inline>)> = Vec::new();
        let inlines = parse_inline(r"\url{https://example.org}", &mut footnotes);
        assert!(matches!(inlines[0], Inline::Url(ref u) if u == "https://example.org"));
    }

    #[test]
    fn footnote_registers_body_and_emits_ref() {
        let (blocks, footnotes) = parse_only(r"This is a test\footnote{with a body}.");
        assert_eq!(footnotes.len(), 1);
        assert_eq!(footnotes[0].0, 1);
        // The body block should contain a FootnoteRef pointing at id 1.
        match &blocks[0] {
            Block::Para(inlines) => {
                let has_ref = inlines.iter().any(|n| matches!(n, Inline::FootnoteRef(1)));
                assert!(has_ref, "expected a FootnoteRef(1) in: {inlines:?}");
            }
            _ => panic!("expected a paragraph"),
        }
    }

    #[test]
    fn theorem_env_emits_bold_italic_label() {
        let (blocks, _) = parse_only(
            r"\begin{theorem}Every prime > 2 is odd.\end{theorem}",
        );
        assert_eq!(blocks.len(), 1);
        let b = &blocks[0];
        match b {
            Block::Para(inlines) => {
                // First run should be a Bold containing an Italic
                // "Theorem." label.
                match &inlines[0] {
                    Inline::Bold(children) => match &children[0] {
                        Inline::Italic(grandchildren) => match &grandchildren[0] {
                            Inline::Text(t) => assert_eq!(t, "Theorem."),
                            _ => panic!("expected Text label"),
                        },
                        _ => panic!("expected Italic inside Bold"),
                    },
                    _ => panic!("expected Bold label, got {:?}", inlines[0]),
                }
                assert!(has_text(b, "Every prime > 2 is odd."));
            }
            _ => panic!("expected a paragraph"),
        }
    }

    #[test]
    fn other_theorem_envs_use_their_labels() {
        let cases = [
            ("lemma",      "Lemma."),
            ("definition", "Definition."),
            ("proposition","Proposition."),
            ("corollary",  "Corollary."),
            ("conjecture", "Conjecture."),
            ("claim",      "Claim."),
            ("fact",       "Fact."),
        ];
        for (env, label) in cases {
            let src = format!(r"\begin{{{env}}}Body.\end{{{env}}}", env = env);
            let mut footnotes: Vec<(usize, Vec<Inline>)> = Vec::new();
            let blocks = parse_body(&src, &mut footnotes);
            assert_eq!(blocks.len(), 1, "{env}: expected one block");
            assert!(has_text(&blocks[0], label), "{env}: missing label {label:?}");
            assert!(has_text(&blocks[0], "Body."), "{env}: missing body");
        }
    }

    #[test]
    fn proof_env_emits_qed_marker() {
        let (blocks, _) = parse_only(
            r"\begin{proof}Trivial.\end{proof}",
        );
        assert_eq!(blocks.len(), 1);
        assert!(has_text(&blocks[0], "\u{25A1}"), "missing QED marker");
        // The body should be wrapped in Italic.
        match &blocks[0] {
            Block::Para(inlines) => {
                let has_italic = inlines.iter().any(|n| matches!(n, Inline::Italic(_)));
                assert!(has_italic, "expected italic body");
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn looks_like_column_spec_recognises_variants() {
        assert!(looks_like_column_spec("|l|c|r|"));
        assert!(looks_like_column_spec("{|l|c|r|p{3cm}|}"));
        assert!(looks_like_column_spec("{|p{3cm}|}"));
        assert!(looks_like_column_spec("{|*{1}|l|}"));
        assert!(!looks_like_column_spec("hello world"));
        assert!(!looks_like_column_spec("|l|c")); // missing closing |
        assert!(!looks_like_column_spec("a | b"));
    }

    #[test]
    fn parse_tabular_drops_spec_line() {
        let rows = parse_tabular(
            r"
                |l|c|r|
                \hline
                a & b & c \\
                d & e & f \\
            ",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["a", "b", "c"]);
        assert_eq!(rows[1], vec!["d", "e", "f"]);
    }
}
