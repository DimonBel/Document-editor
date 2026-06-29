/**
 * Symbol palette data — split into a few small, focused categories
 * so the popover isn't a wall of 200 glyphs.
 */
export interface SymbolEntry {
  /** LaTeX command (without the leading backslash). */
  cmd: string;
  /** Visible character the palette button shows. */
  glyph: string;
  /** Short label used as the button title / a11y label. */
  label: string;
}

export interface SymbolCategory {
  id: string;
  title: string;
  symbols: SymbolEntry[];
}

export const SYMBOL_CATEGORIES: SymbolCategory[] = [
  {
    id: 'greek-lower',
    title: 'Greek (lower)',
    symbols: [
      { cmd: 'alpha',   glyph: 'α', label: 'alpha' },
      { cmd: 'beta',    glyph: 'β', label: 'beta' },
      { cmd: 'gamma',   glyph: 'γ', label: 'gamma' },
      { cmd: 'delta',   glyph: 'δ', label: 'delta' },
      { cmd: 'epsilon', glyph: 'ε', label: 'epsilon' },
      { cmd: 'zeta',    glyph: 'ζ', label: 'zeta' },
      { cmd: 'eta',     glyph: 'η', label: 'eta' },
      { cmd: 'theta',   glyph: 'θ', label: 'theta' },
      { cmd: 'lambda',  glyph: 'λ', label: 'lambda' },
      { cmd: 'mu',      glyph: 'μ', label: 'mu' },
      { cmd: 'pi',      glyph: 'π', label: 'pi' },
      { cmd: 'sigma',   glyph: 'σ', label: 'sigma' },
      { cmd: 'phi',     glyph: 'φ', label: 'phi' },
      { cmd: 'omega',   glyph: 'ω', label: 'omega' },
    ],
  },
  {
    id: 'greek-upper',
    title: 'Greek (upper)',
    symbols: [
      { cmd: 'Gamma',   glyph: 'Γ', label: 'Gamma' },
      { cmd: 'Delta',   glyph: 'Δ', label: 'Delta' },
      { cmd: 'Theta',   glyph: 'Θ', label: 'Theta' },
      { cmd: 'Lambda',  glyph: 'Λ', label: 'Lambda' },
      { cmd: 'Xi',      glyph: 'Ξ', label: 'Xi' },
      { cmd: 'Pi',      glyph: 'Π', label: 'Pi' },
      { cmd: 'Sigma',   glyph: 'Σ', label: 'Sigma' },
      { cmd: 'Phi',     glyph: 'Φ', label: 'Phi' },
      { cmd: 'Psi',     glyph: 'Ψ', label: 'Psi' },
      { cmd: 'Omega',   glyph: 'Ω', label: 'Omega' },
    ],
  },
  {
    id: 'operators',
    title: 'Operators',
    symbols: [
      { cmd: 'pm',      glyph: '±', label: 'plus or minus' },
      { cmd: 'mp',      glyph: '∓', label: 'minus or plus' },
      { cmd: 'times',   glyph: '×', label: 'times' },
      { cmd: 'div',     glyph: '÷', label: 'divide' },
      { cmd: 'cdot',    glyph: '·', label: 'center dot' },
      { cmd: 'circ',    glyph: '∘', label: 'ring' },
      { cmd: 'neq',     glyph: '≠', label: 'not equal' },
      { cmd: 'approx',  glyph: '≈', label: 'approx' },
      { cmd: 'equiv',   glyph: '≡', label: 'equivalent' },
      { cmd: 'leq',     glyph: '≤', label: 'less or equal' },
      { cmd: 'geq',     glyph: '≥', label: 'greater or equal' },
      { cmd: 'sim',     glyph: '∼', label: 'similar' },
    ],
  },
  {
    id: 'relations',
    title: 'Relations & sets',
    symbols: [
      { cmd: 'subset',    glyph: '⊂', label: 'subset' },
      { cmd: 'supset',    glyph: '⊃', label: 'superset' },
      { cmd: 'subseteq',  glyph: '⊆', label: 'subset or equal' },
      { cmd: 'supseteq',  glyph: '⊇', label: 'superset or equal' },
      { cmd: 'in',        glyph: '∈', label: 'in' },
      { cmd: 'notin',     glyph: '∉', label: 'not in' },
      { cmd: 'cup',       glyph: '∪', label: 'union' },
      { cmd: 'cap',       glyph: '∩', label: 'intersection' },
      { cmd: 'setminus',  glyph: '∖', label: 'set minus' },
      { cmd: 'emptyset',  glyph: '∅', label: 'empty set' },
      { cmd: 'forall',    glyph: '∀', label: 'for all' },
      { cmd: 'exists',    glyph: '∃', label: 'there exists' },
    ],
  },
  {
    id: 'arrows',
    title: 'Arrows',
    symbols: [
      { cmd: 'rightarrow',     glyph: '→', label: 'right arrow' },
      { cmd: 'leftarrow',      glyph: '←', label: 'left arrow' },
      { cmd: 'Rightarrow',     glyph: '⇒', label: 'double right arrow' },
      { cmd: 'Leftarrow',      glyph: '⇐', label: 'double left arrow' },
      { cmd: 'leftrightarrow', glyph: '↔', label: 'double arrow' },
      { cmd: 'Leftrightarrow', glyph: '⟺', label: 'double double arrow' },
      { cmd: 'mapsto',         glyph: '↦', label: 'maps to' },
      { cmd: 'uparrow',        glyph: '↑', label: 'up arrow' },
      { cmd: 'downarrow',      glyph: '↓', label: 'down arrow' },
    ],
  },
  {
    id: 'misc',
    title: 'Misc',
    symbols: [
      { cmd: 'infty',   glyph: '∞', label: 'infinity' },
      { cmd: 'partial', glyph: '∂', label: 'partial' },
      { cmd: 'nabla',   glyph: '∇', label: 'nabla' },
      { cmd: 'ldots',   glyph: '…', label: 'low dots' },
      { cmd: 'cdots',   glyph: '⋯', label: 'center dots' },
      { cmd: 'hbar',    glyph: 'ℏ', label: 'h-bar' },
      { cmd: 'ell',     glyph: 'ℓ', label: 'ell' },
      { cmd: 'aleph',   glyph: 'ℵ', label: 'aleph' },
      { cmd: 'angle',   glyph: '∠', label: 'angle' },
      { cmd: 'star',    glyph: '★', label: 'star' },
    ],
  },
];

/** Insert at cursor helper — wraps a LaTeX command in `$...$` when
 *  the symbol is a math glyph, otherwise inserts raw text. */
export function symbolInsert(cmd: string): string {
  const math = ['alpha','beta','gamma','delta','epsilon','zeta','eta','theta',
    'lambda','mu','pi','sigma','phi','omega',
    'Gamma','Delta','Theta','Lambda','Xi','Pi','Sigma','Phi','Psi','Omega',
    'pm','mp','times','div','cdot','circ','neq','approx','equiv','leq','geq','sim',
    'subset','supset','subseteq','supseteq','in','notin','cup','cap','setminus',
    'emptyset','forall','exists',
    'rightarrow','leftarrow','Rightarrow','Leftarrow','leftrightarrow','Leftrightarrow',
    'mapsto','uparrow','downarrow',
    'infty','partial','nabla','ldots','cdots','hbar','ell','aleph','angle','star'];
  return math.includes(cmd) ? `\\${cmd}` : `\\${cmd}`;
}