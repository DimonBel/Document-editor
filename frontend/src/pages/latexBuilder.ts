// Returns a one-time init page that loads KaTeX once and re-renders via postMessage.
// Send {type:'ltx-compile', src:'...'} to compile; iframe replies with ltx-ok / ltx-err.
export function buildInitPage(): string {
  const preamble =
'<!DOCTYPE html>\n' +
'<html>\n' +
'<head>\n' +
'<meta charset="UTF-8">\n' +
'<meta name="viewport" content="width=device-width,initial-scale=1">\n' +
'<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css">\n' +
'<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js"><' + '/script>\n' +
'<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/contrib/auto-render.min.js"\n' +
'  onload="onKaTeXReady()"><' + '/script>\n' +
'<style>\n' +
'  *{box-sizing:border-box}\n' +
'  body{font-family:"Computer Modern","Georgia",serif;max-width:780px;margin:48px auto;\n' +
'       padding:0 48px 64px;font-size:16px;line-height:1.7;color:#111;background:#fff}\n' +
'  h1{font-size:1.9em;text-align:center;margin:.3em 0 .1em}\n' +
'  h2{font-size:1.35em;margin:1.6em 0 .4em;border-bottom:1px solid #ddd;padding-bottom:3px}\n' +
'  h3{font-size:1.15em;margin:1.2em 0 .3em}\n' +
'  h4{font-size:1em;margin:1em 0 .2em}\n' +
'  .author,.date{text-align:center;color:#555;margin:3px 0;font-size:.95em}\n' +
'  p{margin:.5em 0}\n' +
'  ul,ol{padding-left:2em;margin:.4em 0}\n' +
'  li{margin:.2em 0}\n' +
'  table{border-collapse:collapse;margin:1em auto}\n' +
'  td,th{border:1px solid #999;padding:5px 12px}\n' +
'  th{background:#f5f5f5;font-weight:600}\n' +
'  code,tt{font-family:"Courier New",monospace;font-size:.88em;background:#f3f3f3;padding:1px 4px;border-radius:3px}\n' +
'  pre{background:#f3f3f3;padding:14px;border-radius:5px;overflow:auto;font-size:.85em}\n' +
'  .katex-display{overflow-x:auto;overflow-y:hidden;padding:4px 0}\n' +
'  .error{color:#c0392b;background:#fee;padding:14px;border-radius:5px;font-family:monospace;\n' +
'         white-space:pre-wrap;font-size:13px;border-left:4px solid #c0392b}\n' +
'</style>\n' +
'</head>\n' +
'<body>\n' +
'<div id="out"></div>\n' +
'<script>\n' +
'var katexReady = false;\n' +
'var pendingSrc = null;\n' +
'\n' +
'function onKaTeXReady() {\n' +
'  katexReady = true;\n' +
'  parent.postMessage({type:\'ltx-ready\'},\'*\');\n' +
'  if (pendingSrc !== null) { render(pendingSrc); pendingSrc = null; }\n' +
'}\n' +
'\n' +
'window.addEventListener(\'message\', function(e) {\n' +
'  if (e.data && e.data.type === \'ltx-compile\') {\n' +
'    if (katexReady) render(e.data.src);\n' +
'    else pendingSrc = e.data.src;\n' +
'  }\n' +
'});\n' +
'\n' +
'function render(src) {\n' +
'  try {\n' +
'    document.getElementById(\'out\').innerHTML = latexToHtml(src);\n' +
'    renderMathInElement(document.body, {\n' +
'      delimiters: [\n' +
'        {left:\'$$\',right:\'$$\',display:true},\n' +
'        {left:\'$\',right:\'$\',display:false}\n' +
'      ],\n' +
'      macros: {\n' +
'        "\\\\R": "\\\\mathbb{R}",\n' +
'        "\\\\N": "\\\\mathbb{N}",\n' +
'        "\\\\Z": "\\\\mathbb{Z}",\n' +
'        "\\\\qed": "\\\\square"\n' +
'      },\n' +
'      throwOnError: false,\n' +
'      errorColor: \'#c0392b\'\n' +
'    });\n' +
'    parent.postMessage({type:\'ltx-ok\'},\'*\');\n' +
'  } catch(e) {\n' +
'    document.getElementById(\'out\').innerHTML =\n' +
'      \'<div class="error">\'+String(e).replace(/&/g,\'&amp;\').replace(/</g,\'&lt;\')+\'</div>\';\n' +
'    parent.postMessage({type:\'ltx-err\',msg:String(e)},\'*\');\n' +
'  }\n' +
'}\n' +
'function latexToHtml(src) {\n' +
'  src = src.replace(/%[^\\n]*/g, \'\');\n' +
'  var title = \'\', author = \'\', date = \'\';\n' +
'  src = src.replace(/\\\\title\\{([^}]*)\\}/g,  function(_,t){title=t;return \'\';});\n' +
'  src = src.replace(/\\\\author\\{([^}]*)}/, function(_,a){author=a;return \'\';});\n' +
'  src = src.replace(/\\\\date\\{([^}]*)}/, function(_,d){\n' +
'    date = d===\'\\\\today\' ? new Date().toLocaleDateString(\'en-US\',{year:\'numeric\',month:\'long\',day:\'numeric\'}) : d;\n' +
'    return \'\';\n' +
'  });\n' +
'  var docMatch = src.match(/\\\\begin\\{document\\}([\\s\\S]*?)\\\\end\\{document\\}/);\n' +
'  var body = docMatch ? docMatch[1] : src;\n' +
'  var math = [];\n' +
'  function saveMath(tex, display) {\n' +
'    var id = \'__M\'+math.length+\'__\';\n' +
'    math.push(display ? \'$$\'+tex+\'$$\' : \'$\'+tex+\'$\');\n' +
'    return id;\n' +
'  }\n' +
'  body = body.replace(/\\\\begin\\{equation\\*?}([\\s\\S]*?)\\\\end\\{equation\\*?}/g,\n' +
'    function(_,c){return saveMath(c,true);});\n' +
'  body = body.replace(/\\\\begin\\{align\\*?}([\\s\\S]*?)\\\\end\\{align\\*?}/g,\n' +
'    function(_,c){return saveMath(\'\\\\begin{aligned}\'+c+\'\\\\end{aligned}\',true);});\n' +
'  body = body.replace(/\\\\begin\\{gather\\*?}([\\s\\S]*?)\\\\end\\{gather\\*?}/g,\n' +
'    function(_,c){return saveMath(c,true);});\n' +
'  body = body.replace(/\\\\begin\\{multline\\*?}([\\s\\S]*?)\\\\end\\{multline\\*?}/g,\n' +
'    function(_,c){return saveMath(c,true);});\n' +
'  body = body.replace(/\\\\\\[([\\s\\S]*?)\\\\\\]/g,\n' +
'    function(_,c){return saveMath(c,true);});\n' +
'  body = body.replace(/\\$\\$([\\s\\S]*?)\\$\\$/g,\n' +
'    function(_,c){return saveMath(c,true);});\n' +
'  body = body.replace(/\\\\\\(([\\s\\S]*?)\\\\\\)/g,\n' +
'    function(_,c){return saveMath(c,false);});\n' +
'  body = body.replace(/\\$([^\\$\\n]{1,300}?)\\$/g,\n' +
'    function(_,c){return saveMath(c,false);});\n' +
'  var verb = [];\n' +
'  body = body.replace(/\\\\begin\\{verbatim}([\\s\\S]*?)\\\\end\\{verbatim}/g, function(_,c){\n' +
'    var id=\'__V\'+verb.length+\'__\';\n' +
'    verb.push(\'<pre>\'+escHtml(c)+\'</pre>\'); return id;\n' +
'  });\n' +
'  body = body.replace(/\\\\maketitle/g, \'__TITLE__\');\n' +
'  body = body.replace(/\\\\(documentclass|usepackage|geometry|pagestyle|thispagestyle|setlength|renewcommand|newcommand|newtheorem|theoremstyle|DeclareMathOperator|providecommand|RequirePackage)\\b[^\\n]*/g,\'\');\n' +
'  body = processEnv(body, \'itemize\', function(c){\n' +
'    return \'<ul>\'+c.split(/\\\\item\\b/).filter(function(s){return s.trim();})\n' +
'      .map(function(s){return \'<li>\'+s.trim()+\'</li>\';}).join(\'\')+\'</ul>\';\n' +
'  });\n' +
'  body = processEnv(body, \'enumerate\', function(c){\n' +
'    return \'<ol>\'+c.split(/\\\\item\\b/).filter(function(s){return s.trim();})\n' +
'      .map(function(s){return \'<li>\'+s.trim()+\'</li>\';}).join(\'\')+\'</ol>\';\n' +
'  });\n' +
'  body = processEnv(body, \'description\', function(c){\n' +
'    return \'<dl>\'+c.split(/\\\\item\\b/).filter(function(s){return s.trim();})\n' +
'      .map(function(s){\n' +
'        var m=s.match(/^\\[([^\\]]*)\\](.*)/s);\n' +
'        return m ? \'<dt><strong>\'+m[1]+\'</strong></dt><dd>\'+m[2].trim()+\'</dd>\'\n' +
'                 : \'<dd>\'+s.trim()+\'</dd>\';\n' +
'      }).join(\'\')+\'</dl>\';\n' +
'  });\n' +
'  body = processEnv(body, \'tabular\', function(c){\n' +
'    c = c.replace(/\\\\hline/g,\'\');\n' +
'    var rows = c.split(/\\\\\\\\/).map(function(r){return r.trim();}).filter(Boolean);\n' +
'    return \'<table>\'+rows.map(function(row,i){\n' +
'      var cells = row.split(\'&\').map(function(cell){return cell.trim();});\n' +
'      var tag = i===0?\'th\':\'td\';\n' +
'      return \'<tr>\'+cells.map(function(cell){\n' +
'        return \'<\'+tag+\'>\'+cell+\'</\'+tag+\'>\';\n' +
'      }).join(\'\')+\'</tr>\';\n' +
'    }).join(\'\')+\'</table>\';\n' +
'  });\n' +
'  body = processEnv(body, \'abstract\', function(c){\n' +
'    return \'<div style="margin:1.5em 3em;font-size:.95em"><strong>Abstract.</strong> \'+c.trim()+\'</div>\';\n' +
'  });\n' +
'  body = processEnv(body, \'quote\', function(c){return \'<blockquote>\'+c.trim()+\'</blockquote>\';});\n' +
'  body = processEnv(body, \'quotation\', function(c){return \'<blockquote>\'+c.trim()+\'</blockquote>\';});\n' +
'  body = body.replace(/\\\\begin\\{center}([\\s\\S]*?)\\\\end\\{center}/g,\'<div style="text-align:center">$1</div>\');\n' +
'  body = body.replace(/\\\\begin\\{flushleft}([\\s\\S]*?)\\\\end\\{flushleft}/g,\'$1\');\n' +
'  body = body.replace(/\\\\begin\\{flushright}([\\s\\S]*?)\\\\end\\{flushright}/g,\'<div style="text-align:right">$1</div>\');\n' +
'  body = body.replace(/\\\\chapter\\*?\\{([^}]*)}/g,\'<h1>$1</h1>\');\n' +
'  body = body.replace(/\\\\section\\*?\\{([^}]*)}/g,\'<h2>$1</h2>\');\n' +
'  body = body.replace(/\\\\subsection\\*?\\{([^}]*)}/g,\'<h3>$1</h3>\');\n' +
'  body = body.replace(/\\\\subsubsection\\*?\\{([^}]*)}/g,\'<h4>$1</h4>\');\n' +
'  body = body.replace(/\\\\paragraph\\*?\\{([^}]*)}/g,\'<strong>$1</strong> \');\n' +
'  body = body.replace(/\\\\textbf\\{([^}]*)}/g,\'<strong>$1</strong>\');\n' +
'  body = body.replace(/\\\\textit\\{([^}]*)}/g,\'<em>$1</em>\');\n' +
'  body = body.replace(/\\\\emph\\{([^}]*)}/g,\'<em>$1</em>\');\n' +
'  body = body.replace(/\\\\underline\\{([^}]*)}/g,\'<u>$1</u>\');\n' +
'  body = body.replace(/\\\\texttt\\{([^}]*)}/g,\'<code>$1</code>\');\n' +
'  body = body.replace(/\\\\textrm\\{([^}]*)}/g,\'$1\');\n' +
'  body = body.replace(/\\\\text\\{([^}]*)}/g,\'$1\');\n' +
'  body = body.replace(/\\{\\\\bf\\s([^}]*)}/g,\'<strong>$1</strong>\');\n' +
'  body = body.replace(/\\{\\\\it\\s([^}]*)}/g,\'<em>$1</em>\');\n' +
'  body = body.replace(/\\{\\\\tt\\s([^}]*)}/g,\'<code>$1</code>\');\n' +
'  body = body.replace(/\\\\LaTeX\\b/g,\'L<sup style="font-size:.7em">A</sup>T<sub style="font-size:.7em">E</sub>X\');\n' +
'  body = body.replace(/\\\\TeX\\b/g,\'T<sub style="font-size:.7em">E</sub>X\');\n' +
'  body = body.replace(/\\\\today\\b/g, new Date().toLocaleDateString(\'en-US\',{year:\'numeric\',month:\'long\',day:\'numeric\'}));\n' +
'  body = body.replace(/---/g,\'\\u2014\');\n' +
'  body = body.replace(/--/g,\'\\u2013\');\n' +
'  body = body.replace(/``/g,\'\\u201C\');\n' +
'  body = body.replace(/<br>/g,\'<br>\');\n' +
'  body = body.replace(/<br>/g,\'<br>\');\n' +
'  body = body.replace(/\'\'/g,\'\\u201D\');\n' +
'  body = body.replace(/\\\\qquad\\b/g,\'\\u2003\\u2003\');\n' +
'  body = body.replace(/\\\\quad\\b/g,\'\\u2003\');\n' +
'  body = body.replace(/\\\\,/g,\'\\u2009\');\n' +
'  body = body.replace(/\\\\;/g,\'\\u2004\');\n' +
'  body = body.replace(/~~/g,\'\\u00A0\');\n' +
'  body = body.replace(/~/g,\'\\u00A0\');\n' +
'  body = body.replace(/\\\\%/g,\'%\');\n' +
'  body = body.replace(/\\\\&/g,\'&amp;\');\n' +
'  body = body.replace(/\\\\\\$/g,\'$\');\n' +
'  body = body.replace(/\\\\\\{/g,\'{\');\n' +
'  body = body.replace(/\\\\\\}/g,\'}\');\n' +
'  body = body.replace(/\\\\#/g,\'#\');\n' +
'  body = body.replace(/\\\\_/g,\'_\');\n' +
'  body = body.replace(/\\\\footnote\\{([^}]*)}/g,\'<sup title="$1">\\u2020</sup>\');\n' +
'  body = body.replace(/\\\\label\\{[^}]*}/g,\'\');\n' +
'  body = body.replace(/\\\\ref\\{([^}]*)}/g,\'[ref]\');\n' +
'  body = body.replace(/\\\\cite\\{([^}]*)}/g,\'[$1]\');\n' +
'  body = body.replace(/\\\\url\\{([^}]*)}/g,\'<a href="$1">$1</a>\');\n' +
'  body = body.replace(/\\\\href\\{([^}]*)\\}\\{([^}]*)}/g,\'<a href="$1">$2</a>\');\n' +
'  body = body.replace(/\\\\\\\\(\\[.*?\\])?/g,\'<br>\');\n' +
'  body = body.replace(/\\\\newline\\b/g,\'<br>\');\n' +
'  body = body.replace(/\\\\par\\b/g,\'\\n\\n\');\n' +
'  body = body.replace(/\\\\noindent\\b/g,\'\');\n' +
'  body = body.replace(/\\\\smallskip\\b/g,\'<div style="height:.3em"></div>\');\n' +
'  body = body.replace(/\\\\bigskip\\b/g,\'<div style="height:1.2em"></div>\');\n' +
'  body = body.replace(/\\\\vspace\\{[^}]*}/g,\'<div style="height:.8em"></div>\');\n' +
'  body = body.replace(/\\\\hspace\\{[^}]*}/g,\' \');\n' +
'  body = body.replace(/\\\\clearpage\\b|\\\\newpage\\b/g,\'<hr style="border:none;margin:2em 0">\');\n' +
'  body = body.replace(/\\\\(tableofcontents|listoffigures|listoftables|bibliography|bibliographystyle|printbibliography|appendix|frontmatter|mainmatter|backmatter|centering|raggedright|raggedleft|normalsize|small|large|Large|LARGE|huge|Huge|tiny|footnotesize|normalfont|rmfamily|sffamily|ttfamily|bfseries|itshape|upshape|mdseries)\\b/g,\'\');\n' +
'  body = body.replace(/\\n\\s*\\n/g,\'</p><p>\');\n' +
'  var html = \'<p>\'+body.trim()+\'</p>\';\n' +
'  html = html.replace(/__M(\\d+)__/g, function(_,i){return math[+i];});\n' +
'  html = html.replace(/__V(\\d+)__/g, function(_,i){return verb[+i];});\n' +
'  var titleBlock = \'\';\n' +
'  if (title) titleBlock += \'<h1>\'+title+\'</h1>\';\n' +
'  if (author) titleBlock += \'<p class="author">\'+author+\'</p>\';\n' +
'  if (date)   titleBlock += \'<p class="date">\'+date+\'</p>\';\n' +
'  if (titleBlock) titleBlock += \'<hr style="border:none;border-top:1px solid #ddd;margin:1em 0">\';\n' +
'  html = html.replace(\'__TITLE__\', titleBlock);\n' +
'  html = html.replace(/<p>\\s*<\\/p>/g,\'\');\n' +
'  html = html.replace(/<p>(\\s*<(?:h[1-6]|ul|ol|dl|table|div|blockquote|hr|pre))/g,\'$1\');\n' +
'  html = html.replace(/(<\\/(?:h[1-6]|ul|ol|dl|table|div|blockquote|hr|pre)>)\\s*<\\/p>/g,\'$1\');\n' +
'  return html;\n' +
'}\n' +
'function processEnv(src, env, fn) {\n' +
'  var re = new RegExp(\'\\\\\\\\begin\\\\{\'+env+\'\\\\}(\\\\{[^}]*\\\\})?([\\\\s\\\\S]*?)\\\\\\\\end\\\\{\'+env+\'\\\\}\',\'g\');\n' +
'  return src.replace(re, function(_, args, content){ return fn(content, args||\'\'); });\n' +
'}\n' +
'function escHtml(s){\n' +
'  return s.replace(/&/g,\'&amp;\').replace(/</g,\'&lt;\').replace(/>/g,\'&gt;\');\n' +
'}\n' +
'<' + '/script>\n' +
'</body>\n' +
'</html>';

  return preamble;
}
