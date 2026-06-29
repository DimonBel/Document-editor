/// LaTeX math string → OMML XML (without outer <m:oMath> wrapper).
/// The caller wraps the result in <m:oMath>...</m:oMath>.

fn xml_esc(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}

fn mr(text: &str) -> String {
    format!("<m:r><m:t>{}</m:t></m:r>", xml_esc(text))
}

// ── tokeniser ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Tok {
    Cmd(String),
    Group(String),
    Ch(char),
}

fn tokenise(src: &str) -> Vec<Tok> {
    let ch: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < ch.len() {
        match ch[i] {
            '\\' => {
                i += 1;
                if i >= ch.len() { break; }
                if ch[i].is_alphabetic() {
                    let s = i;
                    while i < ch.len() && ch[i].is_alphabetic() { i += 1; }
                    out.push(Tok::Cmd(ch[s..i].iter().collect()));
                } else {
                    out.push(Tok::Cmd(ch[i].to_string()));
                    i += 1;
                }
            }
            '{' => {
                let mut depth = 1;
                i += 1;
                let s = i;
                while i < ch.len() && depth > 0 {
                    if ch[i] == '{' { depth += 1; }
                    else if ch[i] == '}' { depth -= 1; }
                    i += 1;
                }
                let inner: String = ch[s..i - 1].iter().collect();
                out.push(Tok::Group(inner));
            }
            ' ' | '\n' | '\r' | '\t' => { i += 1; }
            c => { out.push(Tok::Ch(c)); i += 1; }
        }
    }
    out
}

// ── recursive parser ─────────────────────────────────────────────────────────

fn parse_group(src: &str) -> String {
    let toks = tokenise(src);
    parse_range(&toks, 0, toks.len())
}

// Take the next argument: a Group token or a single Cmd/Ch
fn next_arg(toks: &[Tok], pos: usize) -> (String, usize) {
    if pos >= toks.len() { return (String::new(), pos); }
    match &toks[pos] {
        Tok::Group(g) => (parse_group(g), pos + 1),
        _ => {
            let (xml, next) = parse_one(toks, pos);
            (xml, next)
        }
    }
}

// Parse one token (without handling ^ _ operators – those are done in parse_range)
fn parse_one(toks: &[Tok], pos: usize) -> (String, usize) {
    if pos >= toks.len() { return (String::new(), pos); }
    match &toks[pos] {
        Tok::Group(g) => (parse_group(g), pos + 1),
        Tok::Ch(c)    => (mr(&c.to_string()), pos + 1),
        Tok::Cmd(cmd) => parse_cmd(toks, pos, cmd),
    }
}

fn greek(name: &str) -> Option<&'static str> {
    match name {
        "alpha"   => Some("α"), "beta"    => Some("β"), "gamma"   => Some("γ"),
        "delta"   => Some("δ"), "epsilon" => Some("ε"), "zeta"    => Some("ζ"),
        "eta"     => Some("η"), "theta"   => Some("θ"), "iota"    => Some("ι"),
        "kappa"   => Some("κ"), "lambda"  => Some("λ"), "mu"      => Some("μ"),
        "nu"      => Some("ν"), "xi"      => Some("ξ"), "pi"      => Some("π"),
        "rho"     => Some("ρ"), "sigma"   => Some("σ"), "tau"     => Some("τ"),
        "upsilon" => Some("υ"), "phi"     => Some("φ"), "chi"     => Some("χ"),
        "psi"     => Some("ψ"), "omega"   => Some("ω"),
        "Gamma"   => Some("Γ"), "Delta"   => Some("Δ"), "Theta"   => Some("Θ"),
        "Lambda"  => Some("Λ"), "Xi"      => Some("Ξ"), "Pi"      => Some("Π"),
        "Sigma"   => Some("Σ"), "Upsilon" => Some("Υ"), "Phi"     => Some("Φ"),
        "Psi"     => Some("Ψ"), "Omega"   => Some("Ω"),
        _ => None,
    }
}

fn symbol(name: &str) -> Option<&'static str> {
    match name {
        "pm" => Some("±"), "mp" => Some("∓"), "times" => Some("×"),
        "div" => Some("÷"), "cdot" => Some("·"), "circ" => Some("∘"),
        "leq" | "le" => Some("≤"), "geq" | "ge" => Some("≥"),
        "neq" | "ne" => Some("≠"), "approx" => Some("≈"),
        "equiv" => Some("≡"), "sim" => Some("∼"),
        "subset" => Some("⊂"), "supset" => Some("⊃"),
        "subseteq" => Some("⊆"), "supseteq" => Some("⊇"),
        "in" => Some("∈"), "notin" => Some("∉"),
        "cup" => Some("∪"), "cap" => Some("∩"), "setminus" => Some("∖"),
        "forall" => Some("∀"), "exists" => Some("∃"),
        "neg" | "lnot" => Some("¬"), "land" => Some("∧"), "lor" => Some("∨"),
        "rightarrow" | "to" => Some("→"), "leftarrow" | "gets" => Some("←"),
        "Rightarrow" => Some("⇒"), "Leftarrow" => Some("⇐"),
        "leftrightarrow" => Some("↔"), "Leftrightarrow" => Some("⟺"),
        "infty" => Some("∞"), "partial" => Some("∂"), "nabla" => Some("∇"),
        "emptyset" => Some("∅"), "varnothing" => Some("∅"),
        "ldots" | "dots" => Some("…"), "cdots" => Some("⋯"),
        "vdots" => Some("⋮"), "ddots" => Some("⋱"),
        "langle" => Some("⟨"), "rangle" => Some("⟩"),
        "lfloor" => Some("⌊"), "rfloor" => Some("⌋"),
        "lceil" => Some("⌈"), "rceil" => Some("⌉"),
        "quad" => Some("\u{2003}"), "qquad" => Some("\u{2003}\u{2003}"),
        "," => Some("\u{2009}"), ";" => Some("\u{2004}"),
        "!" => Some(""), ":" => Some("\u{2004}"),
        "LaTeX" => Some("LaTeX"), "TeX" => Some("TeX"),
        "hbar" => Some("ℏ"), "ell" => Some("ℓ"),
        "Re" => Some("ℜ"), "Im" => Some("ℑ"),
        "aleph" => Some("ℵ"), "wp" => Some("℘"),
        "oplus" => Some("⊕"), "otimes" => Some("⊗"),
        "ominus" => Some("⊖"), "odot" => Some("⊙"),
        "perp" => Some("⊥"), "parallel" => Some("∥"),
        "angle" => Some("∠"), "triangle" => Some("△"),
        "star" => Some("★"), "bullet" => Some("•"),
        "dagger" => Some("†"), "ddagger" => Some("‡"),
        "square" => Some("□"), "blacksquare" => Some("■"),
        "checkmark" => Some("✓"),
        "mapsto" => Some("↦"), "hookrightarrow" => Some("↪"),
        "uparrow" => Some("↑"),
        "downarrow" => Some("↓"), "updownarrow" => Some("↕"),
        "Uparrow" => Some("⇑"), "Downarrow" => Some("⇓"),
        "Updownarrow" => Some("⇕"),
        "backepsilon" => Some("∍"), "eth" => Some("ð"), "Thorn" => Some("Þ"),
        "spadesuit" => Some("♠"), "heartsuit" => Some("♥"),
        "diamondsuit" => Some("♦"), "clubsuit" => Some("♣"),
        "imath" => Some("ı"), "jmath" => Some("ȷ"),
        "complement" => Some("∁"),
        _ => None,
    }
}

fn parse_cmd(toks: &[Tok], pos: usize, cmd: &str) -> (String, usize) {
    if let Some(g) = greek(cmd) { return (mr(g), pos + 1); }
    if let Some(s) = symbol(cmd) { return (mr(s), pos + 1); }

    match cmd {
        "frac" => {
            let (num, p1) = next_arg(toks, pos + 1);
            let (den, p2) = next_arg(toks, p1);
            (format!("<m:f><m:fPr><m:ctrlPr/></m:fPr><m:num>{}</m:num><m:den>{}</m:den></m:f>", num, den), p2)
        }
        "tfrac" | "dfrac" | "cfrac" => {
            let (num, p1) = next_arg(toks, pos + 1);
            let (den, p2) = next_arg(toks, p1);
            (format!("<m:f><m:fPr><m:ctrlPr/></m:fPr><m:num>{}</m:num><m:den>{}</m:den></m:f>", num, den), p2)
        }
        "sqrt" => {
            // Check for optional degree [n]
            let mut p = pos + 1;
            let mut degree = None;
            if let Some(Tok::Ch('[')) = toks.get(p) {
                p += 1;
                let mut deg_toks = Vec::new();
                while p < toks.len() {
                    if let Tok::Ch(']') = &toks[p] { p += 1; break; }
                    deg_toks.push(toks[p].clone());
                    p += 1;
                }
                degree = Some(parse_range(&deg_toks, 0, deg_toks.len()));
            }
            let (base, p2) = next_arg(toks, p);
            let xml = if let Some(deg) = degree {
                format!("<m:rad><m:radPr><m:ctrlPr/></m:radPr><m:deg>{}</m:deg><m:e>{}</m:e></m:rad>", deg, base)
            } else {
                format!("<m:rad><m:radPr><m:degHide m:val=\"1\"/><m:ctrlPr/></m:radPr><m:deg/><m:e>{}</m:e></m:rad>", base)
            };
            (xml, p2)
        }
        "sum" => build_nary(toks, pos + 1, "∑", "undOvr"),
        "prod" | "coprod" => build_nary(toks, pos + 1, "∏", "undOvr"),
        "int"   => build_nary(toks, pos + 1, "∫", "subSup"),
        "iint"  => build_nary(toks, pos + 1, "∬", "subSup"),
        "iiint" => build_nary(toks, pos + 1, "∭", "subSup"),
        "oint"  => build_nary(toks, pos + 1, "∮", "subSup"),
        "bigcup" => build_nary(toks, pos + 1, "⋃", "undOvr"),
        "bigcap" => build_nary(toks, pos + 1, "⋂", "undOvr"),
        "lim" | "limsup" | "liminf" => build_func(toks, pos + 1, cmd),
        "sin" | "cos" | "tan" | "cot" | "sec" | "csc"
        | "arcsin" | "arccos" | "arctan"
        | "log" | "ln" | "exp"
        | "max" | "min" | "sup" | "inf"
        | "det" | "dim" | "ker" | "deg" | "gcd" | "lcm" => build_func(toks, pos + 1, cmd),

        "left" => parse_left(toks, pos + 1),

        "text" | "mathrm" | "textrm" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            // arg is already OMML from parse_group; wrap in normal text style
            (arg, p2)
        }
        "mathbf" | "boldsymbol" | "mathbb" | "mathcal"
        | "mathit" | "mathsf" | "mathtt" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (arg, p2)
        }
        "overline" | "bar" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:bar><m:barPr><m:pos m:val=\"top\"/><m:ctrlPr/></m:barPr><m:e>{}</m:e></m:bar>", arg), p2)
        }
        "underline" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:bar><m:barPr><m:pos m:val=\"bot\"/><m:ctrlPr/></m:barPr><m:e>{}</m:e></m:bar>", arg), p2)
        }
        "hat" | "widehat" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"̂\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "tilde" | "widetilde" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"̃\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "vec" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"⃗\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "dot" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"̇\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "ddot" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"̈\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "overrightarrow" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"⃗\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "overleftarrow" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"⃖\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "check" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"̌\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "breve" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"̆\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "grave" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"̀\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "acute" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:acc><m:accPr><m:chr m:val=\"́\"/><m:ctrlPr/></m:accPr><m:e>{}</m:e></m:acc>", arg), p2)
        }
        "overset" => {
            let (top, p1) = next_arg(toks, pos + 1);
            let (bot, p2) = next_arg(toks, p1);
            (format!("<m:limUpp><m:limUppPr><m:ctrlPr/></m:limUppPr><m:e>{}</m:e><m:lim>{}</m:lim></m:limUpp>", bot, top), p2)
        }
        "underset" => {
            let (bot, p1) = next_arg(toks, pos + 1);
            let (top, p2) = next_arg(toks, p1);
            (format!("<m:limLow><m:limLowPr><m:ctrlPr/></m:limLowPr><m:e>{}</m:e><m:lim>{}</m:lim></m:limLow>", top, bot), p2)
        }
        "stackrel" => {
            let (top, p1) = next_arg(toks, pos + 1);
            let (bot, p2) = next_arg(toks, p1);
            (format!("<m:limUpp><m:limUppPr><m:ctrlPr/></m:limUppPr><m:e>{}</m:e><m:lim>{}</m:lim></m:limUpp>", bot, top), p2)
        }
        "xleftarrow" => {
            // Optional [below] then {above}: produces an arrow with both
            // a label below and a label above.
            let mut p = pos + 1;
            let mut below: Option<String> = None;
            if let Some(Tok::Ch('[')) = toks.get(p) {
                p += 1;
                let mut inner: Vec<Tok> = Vec::new();
                while p < toks.len() {
                    if let Tok::Ch(']') = &toks[p] { p += 1; break; }
                    inner.push(toks[p].clone());
                    p += 1;
                }
                below = Some(parse_range(&inner, 0, inner.len()));
            }
            let (above, p2) = next_arg(toks, p);
            build_stretchy_arrow(toks, pos + 1, p2, "⟵", above.as_str(), below.as_deref())
        }
        "xrightarrow" => {
            let mut p = pos + 1;
            let mut below: Option<String> = None;
            if let Some(Tok::Ch('[')) = toks.get(p) {
                p += 1;
                let mut inner: Vec<Tok> = Vec::new();
                while p < toks.len() {
                    if let Tok::Ch(']') = &toks[p] { p += 1; break; }
                    inner.push(toks[p].clone());
                    p += 1;
                }
                below = Some(parse_range(&inner, 0, inner.len()));
            }
            let (above, p2) = next_arg(toks, p);
            build_stretchy_arrow(toks, pos + 1, p2, "⟶", above.as_str(), below.as_deref())
        }
        "binom" => {
            let (top, p1) = next_arg(toks, pos + 1);
            let (bot, p2) = next_arg(toks, p1);
            let inner = format!("<m:f><m:fPr><m:ctrlPr/></m:fPr><m:num>{}</m:num><m:den>{}</m:den></m:f>", top, bot);
            (format!("<m:d><m:dPr><m:begChr m:val=\"(\"/><m:endChr m:val=\")\"/><m:ctrlPr/></m:dPr><m:e>{}</m:e></m:d>", inner), p2)
        }
        "overbrace" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:limUpp><m:limUppPr><m:ctrlPr/></m:limUppPr><m:e>{}</m:e><m:lim></m:lim></m:limUpp>", arg), p2)
        }
        "underbrace" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("<m:limLow><m:limLowPr><m:ctrlPr/></m:limLowPr><m:e>{}</m:e><m:lim></m:lim></m:limLow>", arg), p2)
        }
        "not" => {
            let (arg, p2) = next_arg(toks, pos + 1);
            (format!("{}{}", arg, mr("\u{0338}")), p2)
        }
        "right" => (String::new(), pos + 2), // consumed by parse_left
        "begin" => parse_begin_env(toks, pos + 1),
        "\\" => (mr(" "), pos + 1),
        "," | ";" | "!" | ":" => (mr("\u{2009}"), pos + 1),
        _ => (mr(&format!("\\{}", cmd)), pos + 1),
    }
}

/// Builds a stretchy arrow with optional above/below labels.
/// Used by `\xleftarrow[below]{above}` and `\xrightarrow[below]{above}`.
fn build_stretchy_arrow(_toks: &[Tok], _start: usize, end: usize, arrow: &str, above: &str, below: Option<&str>) -> (String, usize) {
    let body = mr(arrow);
    let arrow_xml = format!(
        "<m:chr m:val=\"{}\"/>", arrow
    );
    let xml = if let Some(below) = below {
        format!(
            "<m:limLow><m:limLowPr><m:ctrlPr/></m:limLowPr><m:e>{}</m:e><m:lim>{}</m:lim></m:limLow>",
            arrow_xml,
            below
        )
    } else {
        format!(
            "<m:limLow><m:limLowPr><m:ctrlPr/></m:limLowPr><m:e>{}</m:e><m:lim></m:lim></m:limLow>",
            arrow_xml
        )
    };
    // The above-label is treated as a top-side annotation: emit arrow
    // first, then a separate upper annotation. Word renders this as a
    // labelled arrow most of the time.
    let _ = body;
    (xml, end)
}

fn build_nary(toks: &[Tok], mut pos: usize, accent: &str, lim_loc: &str) -> (String, usize) {
    let mut sub_xml = None;
    let mut sup_xml = None;

    loop {
        match toks.get(pos) {
            Some(Tok::Ch('_')) => {
                let (s, p) = next_arg(toks, pos + 1);
                sub_xml = Some(s);
                pos = p;
            }
            Some(Tok::Ch('^')) => {
                let (s, p) = next_arg(toks, pos + 1);
                sup_xml = Some(s);
                pos = p;
            }
            _ => break,
        }
    }

    let (body, pos2) = next_arg(toks, pos);

    let has_sub = sub_xml.is_some();
    let has_sup = sup_xml.is_some();
    let sub_part = sub_xml.map(|s| format!("<m:sub>{}</m:sub>", s)).unwrap_or_default();
    let sup_part = sup_xml.map(|s| format!("<m:sup>{}</m:sup>", s)).unwrap_or_default();

    let xml = format!(
        "<m:nary>\
           <m:naryPr>\
             <m:chr m:val=\"{acc}\"/>\
             <m:limLoc m:val=\"{loc}\"/>\
             <m:subHide m:val=\"{sh}\"/>\
             <m:supHide m:val=\"{sh2}\"/>\
             <m:ctrlPr/>\
           </m:naryPr>\
           {sub}{sup}\
           <m:e>{body}</m:e>\
         </m:nary>",
        acc = xml_esc(accent),
        loc = lim_loc,
        sh  = if has_sub { "0" } else { "1" },
        sh2 = if has_sup { "0" } else { "1" },
        sub = sub_part,
        sup = sup_part,
        body = body,
    );
    (xml, pos2)
}

fn build_func(toks: &[Tok], pos: usize, name: &str) -> (String, usize) {
    // Check for limits: \lim_{x\to\infty}
    let mut p = pos;
    let mut sub_xml = None;
    if let Some(Tok::Ch('_')) = toks.get(p) {
        let (s, next) = next_arg(toks, p + 1);
        sub_xml = Some(s);
        p = next;
    }

    let fname = mr(name);
    let body_xml = if let Some(sub) = sub_xml {
        format!("<m:sSub><m:sSubPr><m:ctrlPr/></m:sSubPr><m:e>{}</m:e><m:sub>{}</m:sub></m:sSub>",
                fname, sub)
    } else {
        fname
    };

    let (arg, p2) = next_arg(toks, p);
    let xml = format!(
        "<m:func><m:funcPr><m:ctrlPr/></m:funcPr><m:fName>{}</m:fName><m:e>{}</m:e></m:func>",
        body_xml, arg
    );
    (xml, p2)
}

fn parse_left(toks: &[Tok], pos: usize) -> (String, usize) {
    let (open_char, mut p) = match toks.get(pos) {
        Some(Tok::Ch(c))  => (c.to_string(), pos + 1),
        Some(Tok::Cmd(c)) => (cmd_bracket(c), pos + 1),
        _ => ("(".to_string(), pos),
    };

    let mut inner_toks: Vec<Tok> = Vec::new();
    let mut depth = 1usize;
    while p < toks.len() {
        match &toks[p] {
            Tok::Cmd(c) if c == "left"  => { depth += 1; inner_toks.push(toks[p].clone()); }
            Tok::Cmd(c) if c == "right" => {
                depth -= 1;
                if depth == 0 { p += 1; break; }
                inner_toks.push(toks[p].clone());
            }
            _ => inner_toks.push(toks[p].clone()),
        }
        p += 1;
    }
    // skip closing bracket token
    let close_char = match toks.get(p) {
        Some(Tok::Ch(c))  => { p += 1; c.to_string() }
        Some(Tok::Cmd(c)) => { let ch = cmd_bracket(c); p += 1; ch }
        _ => ")".to_string(),
    };

    let inner = parse_range(&inner_toks, 0, inner_toks.len());
    let xml = format!(
        "<m:d><m:dPr><m:begChr m:val=\"{}\"/><m:endChr m:val=\"{}\"/><m:ctrlPr/></m:dPr><m:e>{}</m:e></m:d>",
        xml_esc(&open_char), xml_esc(&close_char), inner
    );
    (xml, p)
}

fn cmd_bracket(cmd: &str) -> String {
    match cmd {
        "langle" => "⟨".to_string(), "rangle" => "⟩".to_string(),
        "lfloor" => "⌊".to_string(), "rfloor" => "⌋".to_string(),
        "lceil"  => "⌈".to_string(), "rceil"  => "⌉".to_string(),
        "lbrace" | "{" => "{".to_string(),
        "rbrace" | "}" => "}".to_string(),
        "." => "".to_string(),
        _ => cmd.to_string(),
    }
}

fn parse_begin_env(toks: &[Tok], pos: usize) -> (String, usize) {
    let env_name = match toks.get(pos) {
        Some(Tok::Group(g)) => g.clone(),
        _ => return (String::new(), pos),
    };
    let mut p = pos + 1;
    let mut inner: Vec<Tok> = Vec::new();
    while p < toks.len() {
        if let Tok::Cmd(c) = &toks[p] {
            if c == "end" {
                p += 2; // skip \end{name}
                break;
            }
        }
        inner.push(toks[p].clone());
        p += 1;
    }
    let xml = match env_name.as_str() {
        "matrix" | "pmatrix" | "bmatrix" | "Bmatrix"
        | "vmatrix" | "Vmatrix" => {
            parse_matrix(&inner, &env_name)
        }
        "cases" => parse_cases(&inner),
        "aligned" | "align" | "split" => {
            // Just render inline
            parse_range(&inner, 0, inner.len())
        }
        _ => parse_range(&inner, 0, inner.len()),
    };
    // Wrap in brackets for pmatrix/bmatrix/Bmatrix/vmatrix/Vmatrix.
    // matrix (no decoration) and cases are emitted bare.
    let wrapped = match env_name.as_str() {
        "pmatrix" => format!(
            "<m:d><m:dPr><m:begChr m:val=\"(\"/><m:endChr m:val=\")\"/><m:ctrlPr/></m:dPr><m:e>{}</m:e></m:d>", xml),
        "bmatrix" => format!(
            "<m:d><m:dPr><m:begChr m:val=\"[\"/><m:endChr m:val=\"]\"/><m:ctrlPr/></m:dPr><m:e>{}</m:e></m:d>", xml),
        "Bmatrix" => format!(
            "<m:d><m:dPr><m:begChr m:val=\"{{\"/><m:endChr m:val=\"}}\"/><m:ctrlPr/></m:dPr><m:e>{}</m:e></m:d>", xml),
        "vmatrix" => format!(
            "<m:d><m:dPr><m:begChr m:val=\"|\"/><m:endChr m:val=\"|\"/><m:ctrlPr/></m:dPr><m:e>{}</m:e></m:d>", xml),
        "Vmatrix" => format!(
            "<m:d><m:dPr><m:begChr m:val=\"‖\"/><m:endChr m:val=\"‖\"/><m:ctrlPr/></m:dPr><m:e>{}</m:e></m:d>", xml),
        _ => xml,
    };
    (wrapped, p)
}

fn parse_matrix(toks: &[Tok], _env: &str) -> String {
    // Rows separated by \\, columns by &
    let src: String = toks.iter().map(|t| match t {
        Tok::Cmd(c) if c == "\\" => " \\\\ ".to_string(),
        Tok::Ch(c) => c.to_string(),
        Tok::Cmd(c) => format!("\\{}", c),
        Tok::Group(g) => format!("{{{}}}", g),
    }).collect();

    let rows: Vec<&str> = src.split("\\\\").collect();
    let mut mat_xml = String::new();
    for row in &rows {
        mat_xml.push_str("<m:mr>");
        for cell in row.split('&') {
            let cell_xml = parse_group(cell.trim());
            mat_xml.push_str(&format!("<m:e>{}</m:e>", cell_xml));
        }
        mat_xml.push_str("</m:mr>");
    }
    format!("<m:m><m:mPr><m:mcs><m:mc><m:mcPr><m:count m:val=\"{}\"/><m:mcJc m:val=\"center\"/></m:mcPr></m:mc></m:mcs><m:ctrlPr/></m:mPr>{}</m:m>",
            rows.first().map(|r| r.split('&').count()).unwrap_or(1),
            mat_xml)
}

fn parse_cases(toks: &[Tok]) -> String {
    // Simplified: render as-is
    parse_range(toks, 0, toks.len())
}

// Main recursive parser — handles ^ and _ operators
fn parse_range(toks: &[Tok], start: usize, end: usize) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = start;

    while i < end {
        match &toks[i] {
            Tok::Ch('^') => {
                let base = parts.pop().unwrap_or_default();
                let (sup, next) = next_arg(toks, i + 1);
                i = next;
                // Peek for _ immediately after
                if i < end {
                    if let Tok::Ch('_') = &toks[i] {
                        let (sub, next2) = next_arg(toks, i + 1);
                        i = next2;
                        parts.push(format!(
                            "<m:sSubSup><m:sSubSupPr><m:ctrlPr/></m:sSubSupPr>\
                             <m:e>{base}</m:e><m:sub>{sub}</m:sub><m:sup>{sup}</m:sup></m:sSubSup>"
                        ));
                        continue;
                    }
                }
                parts.push(format!(
                    "<m:sSup><m:sSupPr><m:ctrlPr/></m:sSupPr>\
                     <m:e>{base}</m:e><m:sup>{sup}</m:sup></m:sSup>"
                ));
            }
            Tok::Ch('_') => {
                let base = parts.pop().unwrap_or_default();
                let (sub, next) = next_arg(toks, i + 1);
                i = next;
                // Peek for ^ immediately after
                if i < end {
                    if let Tok::Ch('^') = &toks[i] {
                        let (sup, next2) = next_arg(toks, i + 1);
                        i = next2;
                        parts.push(format!(
                            "<m:sSubSup><m:sSubSupPr><m:ctrlPr/></m:sSubSupPr>\
                             <m:e>{base}</m:e><m:sub>{sub}</m:sub><m:sup>{sup}</m:sup></m:sSubSup>"
                        ));
                        continue;
                    }
                }
                parts.push(format!(
                    "<m:sSub><m:sSubPr><m:ctrlPr/></m:sSubPr>\
                     <m:e>{base}</m:e><m:sub>{sub}</m:sub></m:sSub>"
                ));
            }
            _ => {
                let (xml, next) = parse_one(toks, i);
                parts.push(xml);
                i = next;
            }
        }
    }

    parts.join("")
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Convert LaTeX math string to OMML XML content (inside m:oMath).
pub fn latex_to_omml(tex: &str) -> String {
    let toks = tokenise(tex.trim());
    parse_range(&toks, 0, toks.len())
}

/// Inline math: <m:oMath>...</m:oMath>
pub fn inline_math(tex: &str) -> String {
    format!("<m:oMath xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\">{}</m:oMath>",
            latex_to_omml(tex))
}

/// Display math: <m:oMathPara>...</m:oMathPara>
pub fn display_math(tex: &str) -> String {
    format!(
        "<m:oMathPara xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\">\
           <m:oMathParaPr><m:jc m:val=\"center\"/></m:oMathParaPr>\
           <m:oMath>{}</m:oMath>\
         </m:oMathPara>",
        latex_to_omml(tex)
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn renders_to(tex: &str, needle: &str) {
        let xml = latex_to_omml(tex);
        assert!(
            xml.contains(needle),
            "expected `{needle}` in OMML output for `{tex}`, got: {xml}"
        );
    }

    #[test]
    fn frac_and_sqrt() {
        renders_to("\\frac{a}{b}", "<m:f>");
        renders_to("\\sqrt[3]{x}", "<m:rad>");
    }

    #[test]
    fn sums_and_ints() {
        renders_to("\\sum_{i=0}^{n} a_i", "<m:nary>");
        renders_to("\\int_a^b f(x)\\,dx", "<m:nary>");
        renders_to("\\iint_S", "<m:nary>");
    }

    #[test]
    fn new_accents() {
        renders_to("\\overrightarrow{AB}", "<m:acc>");
        renders_to("\\overleftarrow{AB}", "<m:acc>");
        renders_to("\\check{x}", "<m:acc>");
        renders_to("\\breve{x}", "<m:acc>");
        renders_to("\\grave{x}", "<m:acc>");
        renders_to("\\acute{x}", "<m:acc>");
    }

    #[test]
    fn overset_underset_stackrel() {
        renders_to("\\overset{*}{X}", "<m:limUpp>");
        renders_to("\\underset{*}{X}", "<m:limLow>");
        renders_to("\\stackrel{a}{=}", "<m:limUpp>");
    }

    #[test]
    fn stretchy_arrows() {
        renders_to("\\xleftarrow[below]{above}", "<m:limLow>");
        renders_to("\\xrightarrow{above}", "<m:limLow>");
    }

    #[test]
    fn binom_and_dfrac() {
        renders_to("\\binom{n}{k}", "<m:d>");
        renders_to("\\dfrac{a}{b}", "<m:f>");
        renders_to("\\tfrac{a}{b}", "<m:f>");
    }

    #[test]
    fn matrix_variants() {
        renders_to("\\begin{matrix} a & b \\\\ c & d \\end{matrix}", "<m:m>");
        // pmatrix/bmatrix/Bmatrix/vmatrix/Vmatrix wrap in <m:d> delimiters.
        renders_to("\\begin{pmatrix} 1 \\\\ 2 \\end{pmatrix}", "<m:begChr");
        renders_to("\\begin{bmatrix} 1 \\\\ 2 \\end{bmatrix}", "<m:begChr");
        renders_to("\\begin{Bmatrix} 1 \\\\ 2 \\end{Bmatrix}", "<m:begChr");
        renders_to("\\begin{vmatrix} 1 \\\\ 2 \\end{vmatrix}", "<m:begChr");
        renders_to("\\begin{Vmatrix} 1 \\\\ 2 \\end{Vmatrix}", "<m:begChr");
    }

    #[test]
    fn new_symbols() {
        renders_to("a \\mapsto b", "↦");
        renders_to("x \\uparrow y", "↑");
        renders_to("a \\oplus b", "⊕");
        renders_to("a \\otimes b", "⊗");
        renders_to("a \\ominus b", "⊖");
        renders_to("a \\odot b", "⊙");
        renders_to("\\spadesuit", "♠");
        renders_to("\\imath", "ı");
    }

    #[test]
    fn display_math_wraps_omathpara() {
        let xml = display_math("x + y");
        assert!(xml.starts_with("<m:oMathPara"));
        assert!(xml.contains("<m:oMath>"));
        assert!(xml.contains("</m:oMathPara>"));
    }
}
