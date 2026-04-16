import { useEffect, useRef, useCallback, useState } from 'react';
import { Button, Tooltip } from 'antd';
import { LogoutOutlined } from '@ant-design/icons';
import { useDocStore } from '../store/docStore';
import './DocEditorPage.css';

interface RemoteCursor {
  clientId: string;
  name: string;
  position: number;
  color: string;
}

const COLORS = ['#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F'];

function getCursorColor(clientId: string): string {
  let hash = 0;
  for (let i = 0; i < clientId.length; i++) {
    hash = clientId.charCodeAt(i) + ((hash << 5) - hash);
  }
  return COLORS[Math.abs(hash) % COLORS.length];
}

export function DocEditorPage() {
  const { docId, docTitle, content, connected, leaveDoc, clientId, setContent } = useDocStore();
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const [remoteCursors, setRemoteCursors] = useState<Record<string, RemoteCursor>>({});
  const contentRef = useRef(content);
  const selectionRef = useRef<{ start: number; end: number }>({ start: 0, end: 0 });
  const pendingOpsRef = useRef<Array<{ type: 'insert' | 'delete'; position: number; text?: string; length?: number }>>([]);
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const versionRef = useRef(0);
  const clientName = useDocStore.getState().clientName;

  useEffect(() => {
    contentRef.current = content;
  }, [content]);

  const flushOps = useCallback(() => {
    const ws = wsRef.current;
    if (!ws || pendingOpsRef.current.length === 0 || !connected) return;

    const ops = pendingOpsRef.current;
    pendingOpsRef.current = [];

    for (const op of ops) {
      ws.send(JSON.stringify({
        type: 'DocOperation',
        op: {
          id: crypto.randomUUID(),
          client_id: clientId,
          lamport: Date.now(),
          type: op.type,
          position: op.position,
          text: op.text,
          length: op.length,
        },
      }));
    }
  }, [clientId, connected]);

  const scheduleFlush = useCallback(() => {
    if (flushTimerRef.current) clearTimeout(flushTimerRef.current);
    flushTimerRef.current = setTimeout(() => {
      flushTimerRef.current = null;
      flushOps();
    }, 16);
  }, [flushOps]);

  const applyRemoteOp = useCallback((op: any) => {
    const currentContent = contentRef.current;
    let newContent = currentContent;

    if (op.op_type === 'insert' || op.type === 'insert') {
      const text = op.text || '';
      const pos = Math.min(op.position, currentContent.length);
      newContent = currentContent.slice(0, pos) + text + currentContent.slice(pos);
    } else if (op.op_type === 'delete' || op.type === 'delete') {
      const pos = Math.min(op.position, currentContent.length);
      const len = op.length || 0;
      newContent = currentContent.slice(0, pos) + currentContent.slice(pos + len);
    }

    if (newContent !== currentContent) {
      contentRef.current = newContent;
      setContent(newContent);
      versionRef.current++;
    }
  }, [setContent]);

  useEffect(() => {
    if (!docId) return;

    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    const ws = new WebSocket(`${proto}//${host}/ws/doc/${docId}`);

    ws.onopen = () => {
      useDocStore.getState().setConnected(true);
      ws.send(JSON.stringify({
        type: 'JoinDoc',
        docId,
        clientId,
        name: clientName,
      }));
    };

    ws.onmessage = (evt) => {
      const data = JSON.parse(evt.data);
      
      switch (data.type) {
        case 'doc_sync':
          contentRef.current = data.content || '';
          setContent(data.content || '');
          versionRef.current = data.version || 0;
          
          if (data.clients) {
            const cursors: Record<string, RemoteCursor> = {};
            data.clients.forEach((c: any) => {
              if (c.id !== clientId) {
                cursors[c.id] = {
                  clientId: c.id,
                  name: c.name,
                  position: 0,
                  color: getCursorColor(c.id),
                };
              }
            });
            setRemoteCursors(cursors);
          }
          break;

        case 'doc_operation':
          if (data.op && data.senderId !== clientId) {
            applyRemoteOp(data.op);
          }
          break;

        case 'doc_cursor_update':
          if (data.clientId !== clientId) {
            setRemoteCursors(prev => ({
              ...prev,
              [data.clientId]: {
                clientId: data.clientId,
                name: data.name,
                position: data.position,
                color: getCursorColor(data.clientId),
              }
            }));
          }
          break;

        case 'doc_user_joined':
          if (data.client && data.client.id !== clientId) {
            setRemoteCursors(prev => ({
              ...prev,
              [data.client.id]: {
                clientId: data.client.id,
                name: data.client.name,
                position: 0,
                color: getCursorColor(data.client.id),
              }
            }));
          }
          break;

        case 'doc_user_left':
          if (data.clientId) {
            setRemoteCursors(prev => {
              const next = { ...prev };
              delete next[data.clientId];
              return next;
            });
          }
          break;
      }
    };

    ws.onclose = () => {
      useDocStore.getState().setConnected(false);
    };

    ws.onerror = () => {
      useDocStore.getState().setConnected(false);
    };

    wsRef.current = ws;

    return () => {
      ws.close();
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current);
    };
  }, [docId, clientId, clientName, applyRemoteOp, setContent]);

  const handleSelectionChange = useCallback(() => {
    const textarea = textareaRef.current;
    if (!textarea || !wsRef.current || !connected) return;

    const newSelection = {
      start: textarea.selectionStart,
      end: textarea.selectionEnd,
    };
    
    if (newSelection.start !== selectionRef.current.start || newSelection.end !== selectionRef.current.end) {
      selectionRef.current = newSelection;
      
      wsRef.current.send(JSON.stringify({
        type: 'DocCursorUpdate',
        clientId,
        name: clientName,
        position: newSelection.start,
        selection: newSelection,
      }));
    }
  }, [clientId, clientName, connected]);

  const handleTextChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newContent = e.target.value;
    const oldContent = contentRef.current;

    const minLen = Math.min(oldContent.length, newContent.length);
    let pos = 0;
    
    while (pos < minLen && oldContent[pos] === newContent[pos]) pos++;

    if (newContent.length > oldContent.length) {
      const inserted = newContent.slice(pos, newContent.length);
      pendingOpsRef.current.push({ type: 'insert', position: pos, text: inserted });
    } else if (newContent.length < oldContent.length) {
      const deleted = oldContent.length - newContent.length;
      pendingOpsRef.current.push({ type: 'delete', position: pos, length: deleted });
    }

    contentRef.current = newContent;
    setContent(newContent);
    handleSelectionChange();
    scheduleFlush();
  };

  useEffect(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    textarea.addEventListener('select', handleSelectionChange);
    textarea.addEventListener('click', handleSelectionChange);
    textarea.addEventListener('keyup', handleSelectionChange);

    return () => {
      textarea.removeEventListener('select', handleSelectionChange);
      textarea.removeEventListener('click', handleSelectionChange);
      textarea.removeEventListener('keyup', handleSelectionChange);
    };
  }, [handleSelectionChange]);

  return (
    <div className="doc-page">
      <header className="doc-header">
        <span className="doc-header__title">{docTitle}</span>
        <div className="doc-header__users">
          {Object.values(remoteCursors).map(c => (
            <span key={c.clientId} style={{ color: c.color, marginLeft: 8 }}>
              {c.name}
            </span>
          ))}
        </div>
        <span className={`doc-header__status ${connected ? 'doc-header__status--connected' : 'doc-header__status--disconnected'}`}>
          {connected ? 'Connected' : 'Connecting...'}
        </span>
        <Tooltip title="Leave document">
          <Button className="wb-header__leave" icon={<LogoutOutlined />} onClick={leaveDoc} />
        </Tooltip>
      </header>

      <div className="doc-editor-wrap">
        <textarea
          ref={textareaRef}
          className="doc-textarea"
          value={content}
          onChange={handleTextChange}
          onSelect={handleSelectionChange}
          onClick={handleSelectionChange}
          placeholder="Start typing... Open another browser window to collaborate!"
        />
      </div>
    </div>
  );
}