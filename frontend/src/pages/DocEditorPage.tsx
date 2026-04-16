import { useEffect, useRef, useCallback } from 'react';
import { Button, Tooltip } from 'antd';
import { LogoutOutlined } from '@ant-design/icons';
import { useDocStore } from '../store/docStore';
import './DocEditorPage.css';

export function DocEditorPage() {
  const { docId, docTitle, content, connected, leaveDoc, clientId } = useDocStore();
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const pendingOpsRef = useRef<Array<{ position: number; text?: string; length?: number }>>([]);
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const flushOps = useCallback(() => {
    const ws = wsRef.current;
    if (!ws || pendingOpsRef.current.length === 0) return;

    const ops = pendingOpsRef.current;
    pendingOpsRef.current = [];

    for (const op of ops) {
      ws.send(JSON.stringify({
        type: 'DocOperation',
        op: {
          ...op,
          id: crypto.randomUUID(),
          client_id: clientId,
          lamport: Date.now(),
          type: op.text !== undefined ? 'insert' : 'delete',
        },
      }));
    }
  }, [clientId]);

  const scheduleFlush = useCallback(() => {
    if (flushTimerRef.current) return;
    flushTimerRef.current = setTimeout(() => {
      flushTimerRef.current = null;
      flushOps();
    }, 50);
  }, [flushOps]);

  useEffect(() => {
    if (!docId) return;

    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${proto}//${window.location.host}/ws/doc/${docId}`);

    ws.onopen = () => {
      useDocStore.getState().setConnected(true);
      ws.send(JSON.stringify({
        type: 'JoinDoc',
        docId,
        clientId: useDocStore.getState().clientId,
        name: useDocStore.getState().clientName,
      }));
    };

    ws.onmessage = (evt) => {
      const data = JSON.parse(evt.data);
      switch (data.type) {
        case 'doc_sync':
          useDocStore.getState().setContent(data.content);
          useDocStore.getState().setVersion(data.version);
          break;
      }
    };

    ws.onclose = () => {
      useDocStore.getState().setConnected(false);
    };

    wsRef.current = ws;

    return () => {
      ws.close();
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current);
    };
  }, [docId]);

  const handleTextChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newContent = e.target.value;
    const oldContent = useDocStore.getState().content;

    const minLen = Math.min(oldContent.length, newContent.length);
    let pos = 0;
    while (pos < minLen && oldContent[pos] === newContent[pos]) pos++;

    if (newContent.length > oldContent.length) {
      const inserted = newContent.slice(pos, newContent.length);
      pendingOpsRef.current.push({ position: pos, text: inserted });
    } else if (newContent.length < oldContent.length) {
      const deleted = oldContent.length - newContent.length;
      pendingOpsRef.current.push({ position: pos, length: deleted });
    }

    useDocStore.getState().setContent(newContent);
    scheduleFlush();
  };

  return (
    <div className="doc-page">
      <header className="doc-header">
        <span className="doc-header__title">{docTitle}</span>
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
          placeholder="Start typing..."
        />
      </div>
    </div>
  );
}