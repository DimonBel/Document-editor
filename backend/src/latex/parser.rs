use chrono::Local;

#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Underline(Vec<Inline>),
    Code(String),
    InlineMath(String),
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
}

pub struct Document {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub blocks: Vec<Block>,
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

pub fn parse_inline(src: &str) -> Vec<Inline> {
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
                            result.push(Inline::Bold(parse_inline(&inner)));
                            i = cmd_end + 1 + inner.len() + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "textit" | "textsl" | "emph" | "mathit" => {
                        flush!();
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((inner, _)) = take_braced(&rest, 0) {
                            result.push(Inline::Italic(parse_inline(&inner)));
                            i = cmd_end + 1 + inner.len() + 1;
                        } else { i = cmd_end; }
                        continue;
                    }
                    "underline" => {
                        flush!();
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((inner, _)) = take_braced(&rest, 0) {
                            result.push(Inline::Underline(parse_inline(&inner)));
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
                        // skip footnote content
                        let rest: String = ch[cmd_end..].iter().collect();
                        if let Some((_, after)) = take_braced(&rest, 0) {
                            i = cmd_end + after - rest.len() + rest.len() - after + 1 + inner_len(&rest, 0);
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
    if let Some((env, _)) = take_braced(line, 7) { Some(env) } else { None }
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

fn parse_itemize(content: &str) -> Vec<Vec<Inline>> {
    content.split("\\item")
           .skip(1) // first split is before first \item
           .map(|item| {
               let text = item.trim();
               // remove optional [label]
               let text = if text.starts_with('[') {
                   if let Some(end) = text.find(']') { &text[end+1..] } else { text }
               } else { text };
               parse_inline(text.trim())
           })
           .filter(|v| !v.is_empty())
           .collect()
}

fn parse_tabular(content: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let content = content
        .lines()
        .filter(|l| !l.trim().starts_with("\\hline") && !l.trim().is_empty())
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

fn flush_para(buf: &mut Vec<String>, blocks: &mut Vec<Block>) {
    let text: String = buf.join(" ");
    let text = text.trim().to_string();
    if !text.is_empty() {
        blocks.push(Block::Para(parse_inline(&text)));
    }
    buf.clear();
}

fn parse_body(body: &str) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    let mut para_buf: Vec<String> = Vec::new();

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            flush_para(&mut para_buf, &mut blocks);
            i += 1;
            continue;
        }

        if is_preamble(trimmed) { i += 1; continue; }

        if trimmed.starts_with("\\maketitle") {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(Block::MakeTitle);
            i += 1; continue;
        }

        if trimmed.starts_with("\\hrule") || trimmed.starts_with("\\noindent\\hrule") {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(Block::HRule);
            i += 1; continue;
        }

        if trimmed.starts_with("\\newpage") || trimmed.starts_with("\\clearpage") {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(Block::HRule);
            i += 1; continue;
        }

        if let Some(sec) = parse_section(trimmed) {
            flush_para(&mut para_buf, &mut blocks);
            blocks.push(sec);
            i += 1; continue;
        }

        // \begin{env}
        if let Some(env) = parse_env_begin(trimmed) {
            flush_para(&mut para_buf, &mut blocks);
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
                    let items = parse_itemize(&body);
                    blocks.push(Block::List { ordered: false, items });
                    i = next;
                }
                "enumerate" | "compactenum" => {
                    let (body, next) = collect_env(&env, &lines, i);
                    let items = parse_itemize(&body);
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
                    blocks.push(Block::Para(parse_inline(body.trim())));
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

        // \[ display math \]
        if trimmed.starts_with("\\[") {
            flush_para(&mut para_buf, &mut blocks);
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
            flush_para(&mut para_buf, &mut blocks);
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

    flush_para(&mut para_buf, &mut blocks);
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

        Document { title, author, date, blocks: parse_body(body) }
    }
}
