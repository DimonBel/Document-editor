import { useEffect, useRef, useCallback, useState } from 'react';
import { useLatexStore } from '../../store/latexStore';

const COLORS = ['#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F'];

function getCursorColor(clientId: string): string {
  let hash = 0;
  for (let i = 0; i < clientId.length; i++) {
    hash = clientId.charCodeAt(i) + ((hash << 5) - hash);
  }
  return COLORS[Math.abs(hash) % COLORS.length];
}

interface RemoteUser {
  clientId: string;
  name: string;
  color: string;
}

export function useLatexCollab(onSourceChange?: (source: string) => void) {
  const { roomId, clientId, clientName, setConnected, setRemoteUsers, addRemoteUser, removeRemoteUser } = useLatexStore();
  const wsRef = useRef<WebSocket | null>(null);
  const [remoteUsers, setRemoteUsersState] = useState<Record<string, RemoteUser>>({});
  const sourceDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isLocalChangeRef = useRef(false);
  const sourceRef = useRef('');

  useEffect(() => {
    if (!roomId) return;

    let isMounted = true;

    const connect = () => {
      try {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/${roomId}`;
        const ws = new WebSocket(wsUrl);
        wsRef.current = ws;

        ws.onopen = () => {
          if (!isMounted) return;
          setConnected(true);
          ws.send(JSON.stringify({
            type: 'join',
            clientId,
            name: clientName,
          }));
        };

        ws.onmessage = (evt) => {
          if (!isMounted) return;
          const data = JSON.parse(evt.data);

          switch (data.type) {
            case 'sync': {
              isLocalChangeRef.current = true;
              if (data.latexSource && onSourceChange) {
                sourceRef.current = data.latexSource;
                onSourceChange(data.latexSource);
              }
              if (data.clients) {
                const users: Record<string, RemoteUser> = {};
                data.clients.forEach((c: any) => {
                  if (c.id !== clientId) {
                    users[c.id] = {
                      clientId: c.id,
                      name: c.name,
                      color: getCursorColor(c.id),
                    };
                  }
                });
                setRemoteUsersState(users);
                setRemoteUsers(users);
              }
              isLocalChangeRef.current = false;
              break;
            }
            case 'latex_source': {
              if (data.senderId !== clientId && data.source) {
                isLocalChangeRef.current = true;
                sourceRef.current = data.source;
                onSourceChange?.(data.source);
                isLocalChangeRef.current = false;
              }
              break;
            }
            case 'user_joined': {
              if (data.client && data.client.id !== clientId) {
                const newUser = {
                  clientId: data.client.id,
                  name: data.client.name,
                  color: getCursorColor(data.client.id),
                };
                setRemoteUsersState(prev => ({
                  ...prev,
                  [data.client.id]: newUser,
                }));
                const usersObj: Record<string, { name: string; color: string }> = {};
                usersObj[data.client.id] = { name: data.client.name, color: getCursorColor(data.client.id) };
                setRemoteUsers(usersObj);
              }
              break;
            }
            case 'user_left': {
              if (data.clientId) {
                setRemoteUsersState(prev => {
                  const next = { ...prev };
                  delete next[data.clientId];
                  return next;
                });
                removeRemoteUser(data.clientId);
              }
              break;
            }
          }
        };

        ws.onclose = () => {
          if (!isMounted) return;
          setConnected(false);
          reconnectTimeoutRef.current = setTimeout(() => {
            if (isMounted && roomId) {
              connect();
            }
          }, 2000);
        };

        ws.onerror = () => {
          ws.close();
        };
      } catch (e) {
        console.error('WebSocket error:', e);
      }
    };

    connect();

    return () => {
      isMounted = false;
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (sourceDebounceRef.current) {
        clearTimeout(sourceDebounceRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [roomId, clientId, clientName, setConnected, setRemoteUsers, addRemoteUser, removeRemoteUser, onSourceChange]);

  const sendSourceUpdate = useCallback((source: string) => {
    if (sourceDebounceRef.current) {
      clearTimeout(sourceDebounceRef.current);
    }
    sourceDebounceRef.current = setTimeout(() => {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({
          type: 'latexSource',
          senderId: clientId,
          source,
        }));
      }
    }, 300);
  }, [clientId]);

  return { sendSourceUpdate, remoteUsers };
}