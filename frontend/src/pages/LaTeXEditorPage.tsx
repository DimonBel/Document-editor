import { useState, useCallback, useRef, useEffect } from 'react';
import { Button, Tooltip, message, Input, Modal, List, Avatar, Tag } from 'antd';
import { PlayCircleOutlined, LoadingOutlined, CopyOutlined, WarningOutlined, TeamOutlined, SaveOutlined } from '@ant-design/icons';
import CodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { stex } from '@codemirror/legacy-modes/mode/stex';
import { EditorView } from '@codemirror/view';
import { useLatexStore } from '../store/latexStore';
import { useLatexCollab } from '../features/collaboration/useLatexCollab';
import { roomsApi } from '../core/api/roomsApi';
import { buildInitPage } from './latexBuilder';
import { RoomInfo } from '../types';
import './LaTeXEditorPage.css';

const TEMPLATES: { label: string; source: string }[] = [
  {
    label: 'Article',
    source: `\\documentclass{article}
\\usepackage{amsmath}

\\title{My Document}
\\author{Author Name}
\\date{\\today}

\\begin{document}
\\maketitle

\\section{Introduction}
Hello, \\LaTeX! Press \\textbf{Ctrl+Enter} to compile.

\\section{Mathematics}
The quadratic formula:
\\begin{equation}
  x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}
\\end{equation}

Euler's identity: $e^{i\\pi} + 1 = 0$

\\section{Lists}
\\begin{itemize}
  \\item First item
  \\item Second item
  \\item Third item
\\end{itemize}

\\end{document}`,
  },
  {
    label: 'Math',
    source: `\\documentclass{article}
\\usepackage{amsmath, amssymb}

\\begin{document}

\\section*{Math Examples}

Inline: $a^2 + b^2 = c^2$

Display:
\\[ \\int_0^\\infty e^{-x^2}\\,dx = \\frac{\\sqrt{\\pi}}{2} \\]

Aligned equations:
\\begin{align}
  f(x) &= x^2 + 2x + 1 \\\\
       &= (x+1)^2
\\end{align}

\\end{document}`,
  },
  {
    label: 'Table',
    source: `\\documentclass{article}

\\begin{document}

\\section*{Results}

\\begin{tabular}{|l|c|c|}
  \\hline
  Name  & Score & Grade \\\\
  \\hline
  Alice & 95    & A \\\\
  Bob   & 82    & B \\\\
  Carol & 91    & A \\\\
  \\hline
\\end{tabular}

\\end{document}`,
  },
  {
    label: 'Theorem',
    source: `\\documentclass{article}
\\usepackage{amsmath}

\\begin{document}

\\section*{Pythagorean Theorem}

\\textbf{Theorem.}
For a right triangle with legs $a$, $b$ and hypotenuse $c$:
\\[ a^2 + b^2 = c^2 \\]

\\textbf{Proof.}
Consider the square with side $(a+b)$. \\qquad $\\square$

\\end{document}`,
  },
];

type Status = 'idle' | 'compiling' | 'success' | 'error';

const cmExtensions = [
  StreamLanguage.define(stex),
  EditorView.theme({
    '&': { height: '100%', fontSize: '14px' },
    '.cm-scroller': { overflow: 'auto', fontFamily: '"Fira Code", "Consolas", monospace' },
    '.cm-content': { padding: '8px 0' },
    '.cm-line': { padding: '0 12px' },
  }),
  EditorView.lineWrapping,
];

const INIT_PAGE = buildInitPage();

interface Props { onBack?: () => void; }

export function LaTeXEditorPage({ onBack }: Props) {
  const [source, setSource] = useState(TEMPLATES[0].source);
  const [status, setStatus] = useState<Status>('idle');
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [hasCompiled, setHasCompiled] = useState(false);
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const iframeReady = useRef(false);

  const [showRoomModal, setShowRoomModal] = useState(false);
  const [showCommitModal, setShowCommitModal] = useState(false);
  const [roomName, setRoomName] = useState('');
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [loadingRooms, setLoadingRooms] = useState(false);

  const handleSourceChange = useCallback((newSource: string) => {
    setSource(newSource);
    setStatus('idle');
    setHasCompiled(false);
  }, []);

  const { roomId, roomName: currentRoomName, clientName, elements, connected, joinLatexRoom, leaveLatexRoom, addCommit } = useLatexStore();
  const { sendSourceUpdate, remoteUsers } = useLatexCollab(handleSourceChange);

  const handleSourceUpdate = useCallback((newSource: string) => {
    setSource(newSource);
    if (roomId) {
      sendSourceUpdate(newSource);
    }
  }, [sendSourceUpdate, roomId]);

  const remoteUsersList = Object.values(remoteUsers);

  useEffect(() => {
    const handler = (e: MessageEvent) => {
      if (e.data?.type === 'ltx-ready') {
        iframeReady.current = true;
      } else if (e.data?.type === 'ltx-ok') {
        setStatus('success');
        setErrorMsg(null);
      } else if (e.data?.type === 'ltx-err') {
        setStatus('error');
        setErrorMsg(e.data.msg as string);
      }
    };
    window.addEventListener('message', handler);
    return () => window.removeEventListener('message', handler);
  }, []);

  const compile = useCallback(() => {
    if (status === 'compiling') return;
    if (!iframeReady.current) {
      message.warning('Preview not ready yet, please wait...');
      return;
    }
    setStatus('compiling');
    setErrorMsg(null);
    setHasCompiled(true);
    const iframe = iframeRef.current;
    if (iframe?.contentWindow) {
      iframe.contentWindow.postMessage({ type: 'ltx-compile', src: source }, '*');
    }
  }, [source, status]);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      compile();
    }
  }, [compile]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  const fetchRooms = async () => {
    setLoadingRooms(true);
    try {
      const list = await roomsApi.list();
      setRooms(list);
    } catch {
      message.error('Failed to load rooms');
    } finally {
      setLoadingRooms(false);
    }
  };

  const handleJoinRoom = async (room: RoomInfo) => {
    joinLatexRoom(room.id, room.name);
    setShowRoomModal(false);
    message.success(`Joined room: ${room.name}`);
  };

  const handleCreateRoom = async () => {
    if (!roomName.trim()) {
      message.warning('Enter a room name');
      return;
    }
    try {
      const room = await roomsApi.create(roomName.trim());
      joinLatexRoom(room.id, room.name);
      setRoomName('');
      setShowRoomModal(false);
      message.success(`Created and joined: ${room.name}`);
    } catch {
      message.error('Failed to create room');
    }
  };

  const handleLeaveRoom = () => {
    leaveLatexRoom();
    message.info('Left the room');
  };

  const handleCommit = () => {
    const annotationCount = elements.length;
    addCommit(clientName, source, annotationCount);
    message.success(`[${clientName}] committed ${annotationCount} annotation${annotationCount !== 1 ? 's' : ''}`);
    setShowCommitModal(false);
  };

  return (
    <div className="ltx-page">
      <header className="ltx-header">
        <div className="ltx-header__left">
          {onBack && <button className="ltx-back-btn" onClick={onBack}>← Home</button>}
          <span className="ltx-logo">Σ</span>
          <span className="ltx-title">LaTeX Editor</span>
          {roomId && (
            <Tag color="blue">{currentRoomName}</Tag>
          )}
          <span className="ltx-engine-badge">KaTeX · browser</span>
        </div>

        <div className="ltx-header__center">
          {TEMPLATES.map((t) => (
            <Tooltip key={t.label} title={`Load ${t.label} template`}>
              <button className="ltx-template-btn" onClick={() => {
                const newSource = t.source;
                setSource(newSource);
                setStatus('idle');
                setErrorMsg(null);
                setHasCompiled(false);
                if (roomId) {
                  sendSourceUpdate(newSource);
                }
              }}>{t.label}</button>
            </Tooltip>
          ))}
        </div>

        <div className="ltx-header__right">
          <Tooltip title={roomId ? "Leave room" : "Join or create room"}>
            <Button
              icon={<TeamOutlined />}
              onClick={() => {
                fetchRooms();
                setShowRoomModal(true);
              }}
            >
              {roomId ? 'Leave Room' : 'Join Room'}
            </Button>
          </Tooltip>
          {roomId && (
            <Tooltip title="Commit all changes">
              <Button
                icon={<SaveOutlined />}
                type="primary"
                onClick={() => setShowCommitModal(true)}
                disabled={elements.length === 0 && source === TEMPLATES[0].source}
              >
                Commit ({elements.length})
              </Button>
            </Tooltip>
          )}
          {connected && <Tag color="green">Connected</Tag>}
          {status === 'success'   && <span className="ltx-badge ltx-badge--success">✓ OK</span>}
          {status === 'error'     && <span className="ltx-badge ltx-badge--error">✗ Error</span>}
          {status === 'compiling' && <span className="ltx-badge ltx-badge--loading"><LoadingOutlined spin /> Compiling…</span>}
          <Button type="primary"
            icon={status === 'compiling' ? <LoadingOutlined /> : <PlayCircleOutlined />}
            disabled={status === 'compiling'} onClick={compile}>
            Compile
          </Button>
        </div>
      </header>

      {roomId && (
        <div className="ltx-collab-bar">
          <span className="ltx-collab-label">Collaborators:</span>
          <Avatar style={{ backgroundColor: '#667eea', marginRight: 4 }}>{clientName.charAt(0).toUpperCase()}</Avatar>
          <span style={{ marginRight: 8 }}>{clientName} (you)</span>
          {remoteUsersList.map((user) => (
            <span key={user.name} style={{ marginRight: 8 }}>
              <Avatar style={{ backgroundColor: user.color || '#888' }}>{user.name.charAt(0).toUpperCase()}</Avatar>
              {' '}{user.name}
            </span>
          ))}
          {remoteUsersList.length === 0 && <span style={{ color: '#888' }}>No other collaborators yet</span>}
        </div>
      )}

      <div className="ltx-workspace">
        <div className="ltx-panel ltx-panel--editor">
          <div className="ltx-panel__label">
            LaTeX Source
            <span className="ltx-panel__hint">Ctrl+Enter to compile</span>
          </div>
          <div className="ltx-editor-body ltx-editor-body--cm">
            <CodeMirror
              value={source}
              extensions={cmExtensions}
              onChange={handleSourceUpdate}
              theme="dark"
              height="100%"
              basicSetup={{
                lineNumbers: true,
                foldGutter: false,
                dropCursor: false,
                allowMultipleSelections: false,
                indentOnInput: true,
                bracketMatching: true,
                autocompletion: false,
                highlightActiveLine: true,
                highlightSelectionMatches: false,
              }}
            />
          </div>
        </div>

        <div className="ltx-divider" />

        <div className="ltx-panel ltx-panel--preview">
          <div className="ltx-panel__label">
            Preview
          </div>
          {!hasCompiled ? (
            <div className="ltx-empty">
              <div className="ltx-empty__icon">📄</div>
              <p>Press <kbd>Ctrl+Enter</kbd> or click <strong>Compile</strong></p>
              <p className="ltx-empty__sub">Math via KaTeX · no install needed</p>
            </div>
          ) : status === 'error' && errorMsg ? (
            <ErrorPanel text={errorMsg} />
          ) : null}
          <iframe
            ref={iframeRef}
            className="ltx-iframe"
            srcDoc={INIT_PAGE}
            sandbox="allow-scripts"
            title="LaTeX Preview"
            style={{ display: hasCompiled && !(status === 'error' && errorMsg) ? 'block' : 'none' }}
          />
        </div>
      </div>

      <Modal
        title="Join or Create Room"
        open={showRoomModal}
        onCancel={() => setShowRoomModal(false)}
        footer={null}
        width={500}
      >
        {roomId ? (
          <div>
            <p>You are currently in room: <strong>{currentRoomName}</strong></p>
            <Button type="primary" danger onClick={handleLeaveRoom}>Leave Room</Button>
          </div>
        ) : (
          <div>
            <div style={{ marginBottom: 16 }}>
              <Input
                placeholder="New room name..."
                value={roomName}
                onChange={(e) => setRoomName(e.target.value)}
                onPressEnter={handleCreateRoom}
              />
              <Button type="primary" onClick={handleCreateRoom} style={{ marginTop: 8 }}>
                Create Room
              </Button>
            </div>
            <div style={{ borderTop: '1px solid #eee', paddingTop: 16 }}>
              <h4>Available Rooms</h4>
              <List
                loading={loadingRooms}
                dataSource={rooms}
                renderItem={(room) => (
                  <List.Item
                    actions={[<Button key="join" onClick={() => handleJoinRoom(room)}>Join</Button>]}
                  >
                    <List.Item.Meta
                      title={room.name}
                      description={`${room.client_count} collaborators`}
                    />
                  </List.Item>
                )}
              />
            </div>
          </div>
        )}
      </Modal>

      <Modal
        title="Commit Changes"
        open={showCommitModal}
        onCancel={() => setShowCommitModal(false)}
        onOk={handleCommit}
        okText="Commit"
      >
        <p>Commit all changes from <strong>{clientName}</strong></p>
        <p>LaTeX content: {source.length} characters</p>
        <p>Annotations: {elements.length} drawings</p>
      </Modal>
    </div>
  );
}

function ErrorPanel({ text }: { text: string }) {
  const copy = () => { navigator.clipboard.writeText(text); message.success('Copied'); };
  return (
    <div className="ltx-error">
      <div className="ltx-error__bar">
        <WarningOutlined /><span>Error</span>
        <Tooltip title="Copy"><Button size="small" icon={<CopyOutlined />} onClick={copy} /></Tooltip>
      </div>
      <pre className="ltx-error__log">{text}</pre>
    </div>
  );
}

// buildInitPage is imported from ./latexBuilder