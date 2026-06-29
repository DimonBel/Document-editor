use std::io::{Cursor, Write};
use zip::write::{ZipWriter, SimpleFileOptions};
use zip::CompressionMethod;

use super::parser::{Block, Document, Inline};
use super::omml;

// ── static DOCX parts ─────────────────────────────────────────────────────────

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml"   ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
  <Override PartName="/word/footnotes.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml"/>
</Types>"#;

const RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const WORD_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles"    Target="styles.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/>
</Relationships>"#;

const NUMBERING: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="0">
    <w:lvl w:ilvl="0">
      <w:start w:val="1"/><w:numFmt w:val="bullet"/>
      <w:lvlText w:val="&#x2022;"/>
      <w:lvlJc w:val="left"/>
      <w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr>
      <w:rPr><w:rFonts w:ascii="Symbol" w:hAnsi="Symbol"/></w:rPr>
    </w:lvl>
  </w:abstractNum>
  <w:abstractNum w:abstractNumId="1">
    <w:lvl w:ilvl="0">
      <w:start w:val="1"/><w:numFmt w:val="decimal"/>
      <w:lvlText w:val="%1."/>
      <w:lvlJc w:val="left"/>
      <w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr>
    </w:lvl>
  </w:abstractNum>
  <w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>
  <w:num w:numId="2"><w:abstractNumId w:val="1"/></w:num>
</w:numbering>"#;

const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:docDefaults>
    <w:rPrDefault><w:rPr>
      <w:rFonts w:ascii="Georgia" w:hAnsi="Georgia" w:cs="Georgia"/>
      <w:sz w:val="24"/><w:szCs w:val="24"/>
      <w:lang w:val="en-US"/>
    </w:rPr></w:rPrDefault>
  </w:docDefaults>
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/>
    <w:pPr><w:spacing w:after="120"/></w:pPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Title">
    <w:name w:val="Title"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:jc w:val="center"/><w:spacing w:before="0" w:after="120"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="48"/><w:szCs w:val="48"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Subtitle">
    <w:name w:val="Subtitle"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:jc w:val="center"/><w:spacing w:before="0" w:after="80"/></w:pPr>
    <w:rPr><w:sz w:val="26"/><w:szCs w:val="26"/><w:color w:val="555555"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading1">
    <w:name w:val="heading 1"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:spacing w:before="360" w:after="120"/><w:keepNext/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="36"/><w:szCs w:val="36"/><w:color w:val="1F3864"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading2">
    <w:name w:val="heading 2"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:spacing w:before="280" w:after="100"/><w:keepNext/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="30"/><w:szCs w:val="30"/><w:color w:val="2E4057"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading3">
    <w:name w:val="heading 3"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:spacing w:before="200" w:after="80"/><w:keepNext/></w:pPr>
    <w:rPr><w:b/><w:i/><w:sz w:val="26"/><w:szCs w:val="26"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Code">
    <w:name w:val="Code"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:spacing w:before="120" w:after="120"/></w:pPr>
    <w:rPr>
      <w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>
      <w:sz w:val="20"/><w:szCs w:val="20"/>
    </w:rPr>
  </w:style>
  <w:style w:type="character" w:styleId="Hyperlink">
    <w:name w:val="Hyperlink"/>
    <w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr>
  </w:style>
  <w:style w:type="character" w:styleId="FootnoteReference">
    <w:name w:val="footnote reference"/>
    <w:rPr><w:vertAlign w:val="superscript"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="FootnoteText">
    <w:name w:val="footnote text"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr><w:spacing w:after="60"/></w:pPr>
    <w:rPr><w:sz w:val="20"/><w:szCs w:val="20"/></w:rPr>
  </w:style>
</w:styles>"#;

// ── XML helpers ───────────────────────────────────────────────────────────────

fn xe(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

fn w_run(text: &str, bold: bool, italic: bool, underline: bool, code: bool) -> String {
    let mut rpr = String::new();
    if bold      { rpr.push_str("<w:b/>"); }
    if italic    { rpr.push_str("<w:i/>"); }
    if underline { rpr.push_str("<w:u w:val=\"single\"/>"); }
    if code {
        rpr.push_str("<w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>");
        rpr.push_str("<w:sz w:val=\"20\"/>");
    }
    let rpr_block = if rpr.is_empty() { String::new() } else { format!("<w:rPr>{}</w:rPr>", rpr) };
    format!("<w:r>{}<w:t xml:space=\"preserve\">{}</w:t></w:r>", rpr_block, xe(text))
}

// ── inline → XML ─────────────────────────────────────────────────────────────

fn inline_xml(node: &Inline, bold: bool, italic: bool, underline: bool) -> String {
    match node {
        Inline::Text(t)      => w_run(t, bold, italic, underline, false),
        Inline::Code(t)      => w_run(t, false, false, false, true),
        Inline::InlineMath(m) => omml::inline_math(m),
        Inline::Bold(children) =>
            children.iter().map(|c| inline_xml(c, true,  italic,    underline)).collect(),
        Inline::Italic(children) =>
            children.iter().map(|c| inline_xml(c, bold,  true,      underline)).collect(),
        Inline::Underline(children) =>
            children.iter().map(|c| inline_xml(c, bold,  italic,    true     )).collect(),
        Inline::Hyperlink { url: _, label } =>
            // Hyperlinks are emitted as inline runs with their URL as
            // the visible text. The relationship for the click target
            // is registered separately in build_document_xml — for
            // now we render the URL text in blue/underline so the
            // visual cue survives.
            format!(
                "<w:r><w:rPr><w:rStyle w:val=\"Hyperlink\"/><w:u w:val=\"single\"/></w:rPr>{}</w:r>",
                inlines_xml(label)
            ),
        Inline::Url(url) =>
            format!(
                "<w:r><w:rPr><w:rStyle w:val=\"Hyperlink\"/><w:u w:val=\"single\"/></w:rPr>{}</w:r>",
                w_run(url, false, false, true, false)
            ),
        Inline::FootnoteRef(id) =>
            format!(
                "<w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\"/></w:rPr>\
                 <w:footnoteReference w:id=\"{}\"/></w:r>",
                id
            ),
    }
}

fn inlines_xml(nodes: &[Inline]) -> String {
    nodes.iter().map(|n| inline_xml(n, false, false, false)).collect()
}

// ── block → XML ───────────────────────────────────────────────────────────────

fn heading_xml(level: u8, title: &str) -> String {
    let style = match level { 1 => "Heading1", 2 => "Heading2", _ => "Heading3" };
    format!(
        "<w:p><w:pPr><w:pStyle w:val=\"{}\"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>",
        style, xe(title)
    )
}

fn para_xml(nodes: &[Inline], style: Option<&str>, center: bool, num_id: Option<u32>) -> String {
    let mut ppr = String::new();
    if let Some(s) = style {
        ppr.push_str(&format!("<w:pStyle w:val=\"{}\"/>", s));
    }
    if center {
        ppr.push_str("<w:jc w:val=\"center\"/>");
    }
    if let Some(id) = num_id {
        ppr.push_str(&format!("<w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"{}\"/></w:numPr>", id));
    }
    let ppr_block = if ppr.is_empty() { String::new() } else { format!("<w:pPr>{}</w:pPr>", ppr) };
    format!("<w:p>{}{}</w:p>", ppr_block, inlines_xml(nodes))
}

fn table_xml(rows: &[Vec<String>]) -> String {
    let border = "<w:top w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"999999\"/>\
                  <w:left w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"999999\"/>\
                  <w:bottom w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"999999\"/>\
                  <w:right w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"999999\"/>";
    let ins_border = "<w:insideH w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"999999\"/>\
                      <w:insideV w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"999999\"/>";
    let mut xml = format!(
        "<w:tbl><w:tblPr>\
           <w:tblW w:w=\"5000\" w:type=\"pct\"/>\
           <w:tblBorders>{}{}</w:tblBorders>\
         </w:tblPr>",
        border, ins_border
    );

    for (ri, row) in rows.iter().enumerate() {
        xml.push_str("<w:tr>");
        for cell in row {
            let header = ri == 0;
            let shading = if header {
                "<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F2F2F2\"/>"
            } else { "" };
            let cell_inlines = {
                let mut footnotes: Vec<(usize, Vec<super::parser::Inline>)> = Vec::new();
                super::parser::parse_inline(cell, &mut footnotes)
            };
            let bold = if header { true } else { false };
            let runs: String = cell_inlines.iter().map(|n| inline_xml(n, bold, false, false)).collect();
            xml.push_str(&format!(
                "<w:tc><w:tcPr><w:tcBorders>{}</w:tcBorders>{}</w:tcPr>\
                 <w:p><w:r>{}</w:r></w:p></w:tc>",
                border, shading, runs
            ));
        }
        xml.push_str("</w:tr>");
    }
    xml.push_str("</w:tbl>");
    xml
}

fn block_xml(block: &Block) -> String {
    match block {
        Block::MakeTitle => String::new(), // handled separately
        Block::HRule => "<w:p><w:pPr><w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"CCCCCC\"/></w:pBdr></w:pPr></w:p>".to_string(),
        Block::Section { level, title } => heading_xml(*level, title),
        Block::Para(nodes) => para_xml(nodes, None, false, None),
        Block::List { ordered, items } => {
            let num_id = if *ordered { 2 } else { 1 };
            items.iter()
                 .map(|item| para_xml(item, None, false, Some(num_id)))
                 .collect()
        }
        Block::Table { rows } => table_xml(rows),
        Block::DisplayMath(tex) => {
            // Each row becomes a centered paragraph containing oMathPara
            let math_xml = omml::display_math(tex);
            format!("<w:p><w:pPr><w:jc w:val=\"center\"/><w:spacing w:before=\"120\" w:after=\"120\"/></w:pPr>{}</w:p>", math_xml)
        }
        Block::Verbatim(text) => {
            let mut xml = String::new();
            for line in text.lines() {
                xml.push_str(&format!(
                    "<w:p><w:pPr><w:pStyle w:val=\"Code\"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/><w:sz w:val=\"20\"/></w:rPr><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
                    xe(line)
                ));
            }
            xml
        }
        Block::Caption(nodes) => {
            // Centered, italic. Word's caption style would be ideal
            // here but a generic italic centered paragraph reads the
            // same and avoids needing extra style entries.
            let runs: String = nodes.iter()
                .map(|n| inline_xml(n, false, true, false))
                .collect();
            format!(
                "<w:p><w:pPr><w:jc w:val=\"center\"/><w:spacing w:before=\"80\" w:after=\"160\"/></w:pPr>{}</w:p>",
                runs
            )
        }
        Block::Footnote { .. } => String::new(), // bodies are emitted into word/footnotes.xml, not the body
    }
}

// ── title block ───────────────────────────────────────────────────────────────

fn title_xml(doc: &Document) -> String {
    let mut xml = String::new();
    if let Some(t) = &doc.title {
        xml.push_str(&format!(
            "<w:p><w:pPr><w:pStyle w:val=\"Title\"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>",
            xe(t)
        ));
    }
    if let Some(a) = &doc.author {
        xml.push_str(&format!(
            "<w:p><w:pPr><w:pStyle w:val=\"Subtitle\"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>",
            xe(a)
        ));
    }
    if let Some(d) = &doc.date {
        xml.push_str(&format!(
            "<w:p><w:pPr><w:pStyle w:val=\"Subtitle\"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>",
            xe(d)
        ));
    }
    if doc.title.is_some() || doc.author.is_some() {
        xml.push_str("<w:p><w:pPr><w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"CCCCCC\"/></w:pBdr><w:spacing w:after=\"240\"/></w:pPr></w:p>");
    }
    xml
}

// ── document.xml ──────────────────────────────────────────────────────────────

fn build_document_xml(doc: &Document) -> String {
    let mut body = String::new();

    // Title section
    let mut has_maketitle = doc.blocks.iter().any(|b| matches!(b, Block::MakeTitle));
    if !has_maketitle && (doc.title.is_some() || doc.author.is_some()) {
        body.push_str(&title_xml(doc));
    }

    for block in &doc.blocks {
        if matches!(block, Block::MakeTitle) {
            body.push_str(&title_xml(doc));
        } else {
            body.push_str(&block_xml(block));
        }
    }

    // Section properties (page margins matching LaTeX defaults)
    body.push_str("<w:sectPr>\
        <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
        <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
                 w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/>\
    </w:sectPr>");

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <w:document \
           xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
           xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" \
           xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
           <w:body>{}</w:body>\
         </w:document>",
        body
    )
}

/// Build `word/footnotes.xml`. The required `separator` (id=-1) and
/// `continuationSeparator` (id=0) footnote bodies must be present for
/// Word to render the rest correctly.
fn build_footnotes_xml(doc: &Document) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:footnotes xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
    );

    // Required boilerplate separators.
    xml.push_str(
        "<w:footnote w:type=\"separator\" w:id=\"-1\">\
            <w:p><w:r><w:separator/></w:r></w:p>\
         </w:footnote>\
         <w:footnote w:type=\"continuationSeparator\" w:id=\"0\">\
            <w:p><w:r><w:continuationSeparator/></w:r></w:p>\
         </w:footnote>",
    );

    for (id, text) in &doc.footnotes {
        let runs: String = text.iter()
            .map(|n| inline_xml(n, false, false, false))
            .collect();
        xml.push_str(&format!(
            "<w:footnote w:id=\"{}\">\
                <w:p>\
                    <w:pPr><w:pStyle w:val=\"FootnoteText\"/></w:pPr>\
                    <w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\"/></w:rPr><w:footnoteRef/></w:r>\
                    <w:r><w:t xml:space=\"preserve\"> </w:t></w:r>\
                    {}\
                </w:p>\
             </w:footnote>",
            id, runs
        ));
    }

    xml.push_str("</w:footnotes>");
    xml
}

// ── public ────────────────────────────────────────────────────────────────────

pub fn build_docx(source: &str) -> Result<Vec<u8>, String> {
    let doc = Document::parse(source);
    let document_xml = build_document_xml(&doc);

    let cursor = Cursor::new(Vec::<u8>::new());
    let mut zip = ZipWriter::new(cursor);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", opts).map_err(|e| e.to_string())?;
    zip.write_all(CONTENT_TYPES.as_bytes()).map_err(|e| e.to_string())?;

    zip.add_directory("_rels/", opts).map_err(|e| e.to_string())?;
    zip.start_file("_rels/.rels", opts).map_err(|e| e.to_string())?;
    zip.write_all(RELS.as_bytes()).map_err(|e| e.to_string())?;

    zip.add_directory("word/", opts).map_err(|e| e.to_string())?;
    zip.add_directory("word/_rels/", opts).map_err(|e| e.to_string())?;
    zip.start_file("word/_rels/document.xml.rels", opts).map_err(|e| e.to_string())?;
    zip.write_all(WORD_RELS.as_bytes()).map_err(|e| e.to_string())?;

    zip.start_file("word/document.xml", opts).map_err(|e| e.to_string())?;
    zip.write_all(document_xml.as_bytes()).map_err(|e| e.to_string())?;

    zip.start_file("word/styles.xml", opts).map_err(|e| e.to_string())?;
    zip.write_all(STYLES.as_bytes()).map_err(|e| e.to_string())?;

    zip.start_file("word/numbering.xml", opts).map_err(|e| e.to_string())?;
    zip.write_all(NUMBERING.as_bytes()).map_err(|e| e.to_string())?;

    let footnotes_xml = build_footnotes_xml(&doc);
    zip.start_file("word/footnotes.xml", opts).map_err(|e| e.to_string())?;
    zip.write_all(footnotes_xml.as_bytes()).map_err(|e| e.to_string())?;

    let cursor = zip.finish().map_err(|e| e.to_string())?;
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn read_zip_part(bytes: &[u8], name: &str) -> String {
        let cursor = std::io::Cursor::new(bytes);
        let mut zip = zip::ZipArchive::new(cursor).expect("zip");
        let mut file = zip.by_name(name).expect("part not found");
        let mut s = String::new();
        file.read_to_string(&mut s).expect("read");
        s
    }

    #[test]
    fn docx_contains_footnotes_part_when_used() {
        let src = r#"
            \documentclass{article}
            \begin{document}
            Hello\footnote{This is a footnote.}
            \end{document}
        "#;
        let bytes = build_docx(src).expect("build");
        let footnotes_xml = read_zip_part(&bytes, "word/footnotes.xml");
        assert!(footnotes_xml.contains("<w:footnote "));
        assert!(footnotes_xml.contains("This is a footnote"));
    }

    #[test]
    fn docx_contains_separators_when_no_footnotes() {
        let src = r#"
            \documentclass{article}
            \begin{document}
            Just a paragraph.
            \end{document}
        "#;
        let bytes = build_docx(src).expect("build");
        let footnotes_xml = read_zip_part(&bytes, "word/footnotes.xml");
        // The required separator and continuationSeparator must always
        // be present, even with zero user footnotes, so Word doesn't
        // complain when opening the docx.
        assert!(footnotes_xml.contains("w:type=\"separator\""));
        assert!(footnotes_xml.contains("w:type=\"continuationSeparator\""));
    }

    #[test]
    fn docx_contains_hyperlink_when_used() {
        let src = r#"
            \documentclass{article}
            \begin{document}
            See \href{https://example.org}{the docs}.
            \end{document}
        "#;
        let bytes = build_docx(src).expect("build");
        let document_xml = read_zip_part(&bytes, "word/document.xml");
        // The hyperlink inline emits a Hyperlink-style run wrapping the
        // visible label. The URL itself lives on the Inline AST but is
        // not yet emitted as a relationship in document.xml.rels (a
        // future branch will add that).
        assert!(document_xml.contains("w:rStyle w:val=\"Hyperlink\""));
        assert!(document_xml.contains("the docs"));
    }

    #[test]
    fn docx_contains_caption_when_used() {
        let src = r#"
            \documentclass{article}
            \begin{document}
            \caption{A figure caption}
            \end{document}
        "#;
        let bytes = build_docx(src).expect("build");
        let document_xml = read_zip_part(&bytes, "word/document.xml");
        assert!(document_xml.contains("A figure caption"));
    }
}
