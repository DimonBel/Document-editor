export interface CompileError {
  error: string;
  log: string;
}

export type CompileResult =
  | { ok: true; pdfBlob: Blob }
  | { ok: false; error: string; log: string };

export async function compileLaTeX(source: string): Promise<CompileResult> {
  const res = await fetch('/api/latex/compile', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ source }),
  });

  if (res.ok) {
    const pdfBlob = await res.blob();
    return { ok: true, pdfBlob };
  }

  try {
    const data: CompileError = await res.json();
    return { ok: false, error: data.error, log: data.log };
  } catch {
    return { ok: false, error: `Server error ${res.status}`, log: '' };
  }
}
