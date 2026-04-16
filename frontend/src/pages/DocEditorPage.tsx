import { useEffect, useRef, useState } from 'react';
import { Button, Tooltip, message } from 'antd';
import { 
  LeftOutlined, 
  BoldOutlined,
  ItalicOutlined,
  UnderlineOutlined,
  AlignLeftOutlined,
  AlignCenterOutlined,
  AlignRightOutlined,
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
  const clientName = useDocStore.getState().clientName;
  const isLocalChangeRef = useRef(false);
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    contentRef.current = content;
  }, [content]);

  useEffect(() => {
    if (!docId) return;

    let ws: WebSocket;
    let isMounted = true;

    const connect = () => {
      const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const host = window.location.host;
      
      try {
        ws = new WebSocket(`${proto}//${host}/ws/doc/${docId}`);
        wsRef.current = ws;

        ws.onopen = () => {
          if (!isMounted) return;
          useDocStore.getState().setConnected(true);
          ws.send(JSON.stringify({
            type: 'JoinDoc',
            docId,
            clientId,
            name: clientName,
          }));
        };

        ws.onmessage = (evt) => {
          if (!isMounted) return;
          const data = JSON.parse(evt.data);
          
          switch (data.type) {
            case 'doc_sync':
              isLocalChangeRef.current = true;
              if (editorRef.current) {
                editorRef.current.innerText = data.content || '';
              }
              contentRef.current = data.content || '';
              setContent(data.content || '');
              isLocalChangeRef.current = false;
              
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

            case 'doc_content_update':
              if (data.senderId !== clientId && data.content !== undefined) {
                isLocalChangeRef.current = true;
                if (editorRef.current) {
                  editorRef.current.innerText = data.content;
                }
                contentRef.current = data.content;
                setContent(data.content);
                isLocalChangeRef.current = false;
                message.info('Document updated by another user');
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

        ws.onclose = () => {
          if (!isMounted) return;
          useDocStore.getState().setConnected(false);
          reconnectTimeoutRef.current = setTimeout(() => {
            if (isMounted && docId) {
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
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
      if (ws) {
        ws.close();
      }
    };
  }, [docId, clientId, clientName, setContent]);

  const saveContent = () => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      const currentContent = editorRef.current?.innerText || '';
      wsRef.current.send(JSON.stringify({
        type: 'DocContentUpdate',
        content: currentContent,
      }));
    }
  };

  const scheduleSave = () => {
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
    }
    saveTimeoutRef.current = setTimeout(() => {
      saveContent();
    }, 500);
  };

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

    contentRef.current = newContent;
    setContent(newContent);
    scheduleSave();
  };

  const execCommand = (command: string, value?: string) => {
    document.execCommand(command, false, value);
    editorRef.current?.focus();
    handleInput();
  };

  const shareDoc = () => {
    const url = `${window.location.origin}/doc/${docId}`;
    navigator.clipboard.writeText(url);
    message.success('Document link copied!');
  };

  const userCount = Object.keys(remoteUsers).length;

  return (
    <div className="doc-page">
      <header className="doc-header">
        <div className="doc-header__left">
          <Tooltip title="Back to documents">
            <Button 
              className="doc-header__back" 
              icon={<LeftOutlined />} 
              onClick={() => {
                saveContent();
                leaveDoc();
              }}
            />
          </Tooltip>
          <div className="doc-header__title-wrap">
            <span className="doc-header__title">{docTitle}</span>
            <span className={`doc-header__status ${connected ? 'doc-header__status--connected' : 'doc-header__status--disconnected'}`}>
              {connected ? 'Saved' : 'Connecting...'}
            </span>
          </div>
        </div>
        
        <div className="doc-header__center">
          <Tooltip title="Bold (Ctrl+B)">
            <Button className="doc-toolbar__btn" icon={<BoldOutlined />} onClick={() => execCommand('bold')} />
          </Tooltip>
          <Tooltip title="Italic (Ctrl+I)">
            <Button className="doc-toolbar__btn" icon={<ItalicOutlined />} onClick={() => execCommand('italic')} />
          </Tooltip>
          <Tooltip title="Underline (Ctrl+U)">
            <Button className="doc-toolbar__btn" icon={<UnderlineOutlined />} onClick={() => execCommand('underline')} />
          </Tooltip>
          <div className="doc-toolbar__divider" />
          <Tooltip title="Align Left">
            <Button className="doc-toolbar__btn" icon={<AlignLeftOutlined />} onClick={() => execCommand('justifyLeft')} />
          </Tooltip>
          <Tooltip title="Align Center">
            <Button className="doc-toolbar__btn" icon={<AlignCenterOutlined />} onClick={() => execCommand('justifyCenter')} />
          </Tooltip>
          <Tooltip title="Align Right">
            <Button className="doc-toolbar__btn" icon={<AlignRightOutlined />} onClick={() => execCommand('justifyRight')} />
          </Tooltip>
        </div>

        <div className="doc-header__right">
          <div className="doc-header__users">
            {Object.values(remoteUsers).map(user => (
              <Tooltip key={user.clientId} title={user.name}>
                <span 
                  className="doc-header__avatar" 
                  style={{ backgroundColor: user.color }}
                >
                  {user.name.charAt(0).toUpperCase()}
                </span>
              </Tooltip>
            ))}
            {userCount > 0 && (
              <span className="doc-header__user-count">
                {userCount} collaborator{userCount > 1 ? 's' : ''}
              </span>
            )}
          </div>
          <Tooltip title="Share document">
            <Button className="doc-toolbar__btn" icon={<ShareAltOutlined />} onClick={shareDoc} />
          </Tooltip>
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
            data-placeholder="Start typing your document..."
          />
        </div>
      </div>
    </div>
  );
}