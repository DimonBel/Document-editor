import { useEffect, useRef, useCallback, useState } from 'react';
import { Button, Tooltip, message } from 'antd';
import { 
  LeftOutlined, 
  BoldOutlined,
  ItalicOutlined,
  UnderlineOutlined,
  StrikethroughOutlined,
  AlignLeftOutlined,
  AlignCenterOutlined,
  AlignRightOutlined,
  UnorderedListOutlined,
  OrderedListOutlined,
  ShareAltOutlined,
} from '@ant-design/icons';
import { useDocStore } from '../store/docStore';
import './DocEditorPage.css';

interface RemoteUser {
  clientId: string;
  name: string;
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
  const editorRef = useRef<HTMLDivElement>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const [remoteUsers, setRemoteUsers] = useState<Record<string, RemoteUser>>({});
  const contentRef = useRef(content);
  const pendingOpsRef = useRef<Array<{ type: 'insert' | 'delete'; position: number; text?: string; length?: number }>>([]);
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const clientName = useDocStore.getState().clientName;
  const isLocalChangeRef = useRef(false);

  useEffect(() => {
    contentRef.current = content;
  }, [content]);

  const flushOps = useCallback(() => {
    const ws = wsRef.current;
    if (!ws || pendingOpsRef.current.length === 0 || !connected) return;

    const ops = [...pendingOpsRef.current];
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
    }, 50);
  }, [flushOps]);

  const applyRemoteOp = useCallback((op: any) => {
    const editor = editorRef.current;
    if (!editor) return;

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
      if (editor.innerText !== newContent) {
        isLocalChangeRef.current = true;
        editor.innerText = newContent;
      }
    }
  }, [setContent]);

  useEffect(() => {
    if (!docId) return;

    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${proto}//${window.location.host}/ws/doc/${docId}`);

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
          
          if (editorRef.current && editorRef.current.innerText !== data.content) {
            isLocalChangeRef.current = true;
            editorRef.current.innerText = data.content || '';
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
            setRemoteUsers(users);
          }
          break;

        case 'doc_operation':
          if (data.op && data.senderId !== clientId) {
            applyRemoteOp(data.op);
          }
          break;

        case 'doc_content_update':
          if (data.senderId !== clientId && data.content !== undefined) {
            isLocalChangeRef.current = true;
            if (editorRef.current) {
              editorRef.current.innerText = data.content;
            }
            contentRef.current = data.content;
            setContent(data.content);
            isLocalChangeRef.current = false;
          }
          break;

        case 'doc_cursor_update':
          if (data.clientId !== clientId) {
            setRemoteUsers(prev => ({
              ...prev,
              [data.clientId]: {
                clientId: data.clientId,
                name: data.name,
                color: getCursorColor(data.clientId),
              }
            }));
          }
          break;

        case 'doc_user_joined':
          if (data.client && data.client.id !== clientId) {
            setRemoteUsers(prev => ({
              ...prev,
              [data.client.id]: {
                clientId: data.client.id,
                name: data.client.name,
                color: getCursorColor(data.client.id),
              }
            }));
            message.info(`${data.client.name} joined`);
          }
          break;

        case 'doc_user_left':
          if (data.clientId) {
            setRemoteUsers(prev => {
              const next = { ...prev };
              const name = next[data.clientId]?.name;
              delete next[data.clientId];
              if (name) message.info(`${name} left`);
              return next;
            });
          }
          break;
      }
    };

    ws.onclose = () => useDocStore.getState().setConnected(false);
    ws.onerror = () => useDocStore.getState().setConnected(false);

    wsRef.current = ws;

    return () => {
      ws.close();
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current);
    };
  }, [docId, clientId, clientName, applyRemoteOp, setContent]);

  const handleInput = () => {
    if (isLocalChangeRef.current) {
      isLocalChangeRef.current = false;
      return;
    }

    const editor = editorRef.current;
    if (!editor) return;

    const newContent = editor.innerText;
    const oldContent = contentRef.current;

    if (newContent === oldContent) return;

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
    scheduleFlush();

    wsRef.current?.send(JSON.stringify({
      type: 'DocCursorUpdate',
      clientId,
      name: clientName,
      position: 0,
    }));
  };

  const execCmd = (cmd: string, val?: string) => {
    document.execCommand(cmd, false, val);
    editorRef.current?.focus();
  };

  const shareDoc = () => {
    const url = `${window.location.origin}/doc/${docId}`;
    navigator.clipboard.writeText(url);
    message.success('Document link copied to clipboard!');
  };

  const userCount = Object.keys(remoteUsers).length;

  return (
    <div className="doc-page">
      <header className="doc-header">
        <div className="doc-header__left">
          <Button type="text" icon={<LeftOutlined />} onClick={leaveDoc} />
          <span className="doc-header__title">{docTitle}</span>
        </div>
        
        <div className="doc-header__toolbar">
          <Tooltip title="Bold (Ctrl+B)">
            <Button type="text" icon={<BoldOutlined />} onClick={() => execCmd('bold')} className="doc-toolbar__btn" />
          </Tooltip>
          <Tooltip title="Italic (Ctrl+I)">
            <Button type="text" icon={<ItalicOutlined />} onClick={() => execCmd('italic')} className="doc-toolbar__btn" />
          </Tooltip>
          <Tooltip title="Underline (Ctrl+U)">
            <Button type="text" icon={<UnderlineOutlined />} onClick={() => execCmd('underline')} className="doc-toolbar__btn" />
          </Tooltip>
          <Tooltip title="Strikethrough">
            <Button type="text" icon={<StrikethroughOutlined />} onClick={() => execCmd('strikeThrough')} className="doc-toolbar__btn" />
          </Tooltip>
          <div className="doc-toolbar__divider" />
          <Tooltip title="Bullet List">
            <Button type="text" icon={<UnorderedListOutlined />} onClick={() => execCmd('insertUnorderedList')} className="doc-toolbar__btn" />
          </Tooltip>
          <Tooltip title="Numbered List">
            <Button type="text" icon={<OrderedListOutlined />} onClick={() => execCmd('insertOrderedList')} className="doc-toolbar__btn" />
          </Tooltip>
          <div className="doc-toolbar__divider" />
          <Tooltip title="Align Left">
            <Button type="text" icon={<AlignLeftOutlined />} onClick={() => execCmd('justifyLeft')} className="doc-toolbar__btn" />
          </Tooltip>
          <Tooltip title="Align Center">
            <Button type="text" icon={<AlignCenterOutlined />} onClick={() => execCmd('justifyCenter')} className="doc-toolbar__btn" />
          </Tooltip>
          <Tooltip title="Align Right">
            <Button type="text" icon={<AlignRightOutlined />} onClick={() => execCmd('justifyRight')} className="doc-toolbar__btn" />
          </Tooltip>
        </div>

        <div className="doc-header__right">
          <div className="doc-header__collaborators">
            {Object.values(remoteUsers).slice(0, 3).map(user => (
              <Tooltip key={user.clientId} title={user.name}>
                <span className="doc-header__avatar" style={{ backgroundColor: user.color }}>
                  {user.name.charAt(0).toUpperCase()}
                </span>
              </Tooltip>
            ))}
            {userCount > 3 && (
              <span className="doc-header__avatar doc-header__avatar--more">+{userCount - 3}</span>
            )}
            {userCount > 0 && (
              <span className="doc-header__collab-count">{userCount} collaborator{userCount > 1 ? 's' : ''}</span>
            )}
          </div>
          <Tooltip title="Share document">
            <Button type="text" icon={<ShareAltOutlined />} onClick={shareDoc} />
          </Tooltip>
          <span className={`doc-header__status ${connected ? 'doc-header__status--connected' : ''}`}>
            {connected ? '● Connected' : '○ Connecting...'}
          </span>
        </div>
      </header>

      <div className="doc-editor-container">
        <div className="doc-editor-paper">
          <div
            ref={editorRef}
            className="doc-editor-content"
            contentEditable
            suppressContentEditableWarning
            onInput={handleInput}
            spellCheck={false}
          />
        </div>
      </div>
    </div>
  );
}