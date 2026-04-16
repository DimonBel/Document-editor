const BASE = '/api';

async function request(path, options = {}) {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || `HTTP ${res.status}`);
  }
  return res.json();
}

export const createRoom = (name) =>
  request('/rooms', { method: 'POST', body: JSON.stringify({ name }) });

export const getRoom = (roomId) => request(`/rooms/${roomId}`);

export const listRooms = () => request('/rooms');
