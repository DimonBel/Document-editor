const BASE = '/api';

class ApiError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
    this.name = 'ApiError';
  }
}

async function request(path, init = {}) {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  });
  if (!res.ok) {
    const body = await res.text().catch(() => res.statusText);
    throw new ApiError(res.status, body);
  }
  return res.json();
}

export const roomsApi = {
  list:   ()         => request('/rooms'),
  get:    (id)       => request(`/rooms/${id}`),
  create: (name)     => request('/rooms', { method: 'POST', body: JSON.stringify({ name }) }),
};
