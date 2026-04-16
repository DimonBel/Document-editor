import { DocInfo } from '../../types';

const BASE = '/api';

export class ApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
    this.name = 'ApiError';
  }
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  });
  if (!res.ok) {
    const body = await res.text().catch(() => res.statusText);
    throw new ApiError(res.status, body);
  }
  return res.json() as Promise<T>;
}

export const docsApi = {
  list: () => request<DocInfo[]>('/documents'),
  get: (id: string) => request<DocInfo>(`/documents/${id}`),
  create: (title: string) => request<DocInfo>('/documents', { method: 'POST', body: JSON.stringify({ title }) }),
};