import { RoomInfo } from '../../types';

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

export const roomsApi = {
  list: () => request<RoomInfo[]>('/rooms'),
  get: (id: string) => request<RoomInfo>(`/rooms/${id}`),
  create: (name: string) => request<RoomInfo>('/rooms', { method: 'POST', body: JSON.stringify({ name }) }),
};