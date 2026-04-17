import { useState, useEffect } from 'react';
import { Card, Typography, message } from 'antd';
import { FormOutlined, FileTextOutlined } from '@ant-design/icons';
import { WhiteboardPage } from './pages/WhiteboardPage';
import { DocEditorPage } from './pages/DocEditorPage';
import { LaTeXEditorPage } from './pages/LaTeXEditorPage';
import { DocSelector } from './features/doc/DocSelector';
import { RoomSelector } from './features/room/RoomSelector';
import { ErrorBoundary } from './shared/components/ErrorBoundary';
import { useDocStore } from './store/docStore';
import { useRoomStore } from './store/roomStore';
import { docsApi } from './core/api/docsApi';
import { roomsApi } from './core/api/roomsApi';

const { Title, Text } = Typography;

type AppMode = 'home' | 'room' | 'doc-cabinet' | 'doc' | 'latex';

export default function App() {
  const [mode, setMode] = useState<AppMode>('home');
  const [loadingDoc, setLoadingDoc] = useState(false);
  const [loadingRoom, setLoadingRoom] = useState(false);

  const roomId = useRoomStore((s) => s.roomId);
  const docId = useDocStore((s) => s.docId);

  const goToCabinet = () => {
    useDocStore.getState().leaveDoc();
    setMode('doc-cabinet');
    window.history.pushState({}, '', '/');
  };

  const goHome = () => {
    useDocStore.getState().leaveDoc();
    useRoomStore.getState().leaveRoom();
    setMode('home');
    window.history.pushState({}, '', '/');
  };

  const goToRoomLobby = () => {
    useRoomStore.getState().leaveRoom();
    setMode('room');
    window.history.pushState({}, '', '/');
  };

  const goToDoc = async (dId: string) => {
    setLoadingDoc(true);
    try {
      const doc = await docsApi.get(dId);
      useDocStore.getState().joinDoc(doc.id, doc.title);
      setMode('doc');
      window.history.pushState({}, '', `/doc/${dId}`);
    } catch (e) {
      console.error('Failed to load document:', e);
      message.error('Document not found');
      goToCabinet();
    } finally {
      setLoadingDoc(false);
    }
  };

  const goToRoom = async (rId: string) => {
    setLoadingRoom(true);
    try {
      const room = await roomsApi.get(rId);
      useRoomStore.getState().joinRoom({ roomId: room.id, roomName: room.name });
      setMode('room');
      window.history.pushState({}, '', `/room/${rId}`);
    } catch {
      message.error('Room not found');
      setMode('room');
    } finally {
      setLoadingRoom(false);
    }
  };

  // On mount: handle deep links /doc/:id and /room/:id
  useEffect(() => {
    const path = window.location.pathname;
    const docMatch = path.match(/^\/doc\/([a-zA-Z0-9-]+)$/);
    const roomMatch = path.match(/^\/room\/([a-zA-Z0-9-]+)$/);
    if (docMatch) {
      goToDoc(docMatch[1]);
    } else if (roomMatch) {
      setMode('room');
      goToRoom(roomMatch[1]);
    }
  }, []);

  useEffect(() => {
    const handlePopState = () => {
      const path = window.location.pathname;
      const docMatch = path.match(/^\/doc\/([a-zA-Z0-9-]+)$/);
      const roomMatch = path.match(/^\/room\/([a-zA-Z0-9-]+)$/);
      if (docMatch) {
        goToDoc(docMatch[1]);
      } else if (roomMatch) {
        setMode('room');
        goToRoom(roomMatch[1]);
      } else {
        goToCabinet();
      }
    };
    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, []);

  // Sync URL when room is joined from RoomSelector
  useEffect(() => {
    if (mode === 'room' && roomId) {
      window.history.replaceState({}, '', `/room/${roomId}`);
    }
  }, [roomId, mode]);

  useEffect(() => {
    if (docId && mode === 'doc-cabinet') setMode('doc');
  }, [docId, mode]);

  if (loadingDoc || loadingRoom) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh' }}>
        <Typography.Text>Loading...</Typography.Text>
      </div>
    );
  }

  return (
    <ErrorBoundary>
      {mode === 'home' && (
        <HomeScreen
          onWhiteboard={() => setMode('room')}
          onDocument={goToCabinet}
          onLatex={() => setMode('latex')}
        />
      )}
      {mode === 'room' && !roomId && <RoomSelectorWrapper onBack={goHome} />}
      {mode === 'room' && roomId && <WhiteboardPageWrapper onBack={goToRoomLobby} />}
      {mode === 'doc-cabinet' && <DocCabinetWrapper onBack={goHome} />}
      {mode === 'doc' && <DocEditorPageWrapper onBack={goToCabinet} />}
      {mode === 'latex' && (
        <LaTeXEditorPage onBack={() => { setMode('home'); window.history.pushState({}, '', '/'); }} />
      )}
    </ErrorBoundary>
  );
}

interface HomeScreenProps {
  onWhiteboard: () => void;
  onDocument: () => void;
  onLatex: () => void;
}

function HomeScreen({ onWhiteboard, onDocument, onLatex }: HomeScreenProps) {
  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: '100vh',
      padding: '24px',
      background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    }}>
      <Card style={{
        width: '100%',
        maxWidth: '420px',
        background: 'white',
        borderRadius: '16px',
        boxShadow: '0 20px 60px rgba(0,0,0,0.3)',
        border: 'none',
        overflow: 'hidden',
      }}>
        <div style={{
          padding: '40px 32px 28px',
          textAlign: 'center',
          borderBottom: '1px solid #eee',
        }}>
          <div style={{
            width: '64px',
            height: '64px',
            margin: '0 auto 20px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            borderRadius: '16px',
            color: 'white',
            fontSize: '28px',
          }}>
            <FileTextOutlined />
          </div>
          <Title level={2} style={{ marginBottom: '8px', color: '#1a1a2e' }}>
            Collaboration Hub
          </Title>
          <Text type="secondary" style={{ fontSize: '15px' }}>
            Choose what you want to use
          </Text>
        </div>

        <div style={{ padding: '24px' }}>
          <button
            onClick={onWhiteboard}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '16px',
              width: '100%',
              padding: '20px',
              borderRadius: '12px',
              border: '2px solid #e8e8e8',
              marginBottom: '12px',
              background: 'white',
              cursor: 'pointer',
              transition: 'all 150ms ease',
              textAlign: 'left',
            }}
            onMouseOver={(e) => {
              e.currentTarget.style.borderColor = '#667eea';
              e.currentTarget.style.boxShadow = '0 4px 12px rgba(102, 126, 234, 0.2)';
            }}
            onMouseOut={(e) => {
              e.currentTarget.style.borderColor = '#e8e8e8';
              e.currentTarget.style.boxShadow = 'none';
            }}
          >
            <div style={{
              width: '48px',
              height: '48px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: '#f0f4ff',
              borderRadius: '12px',
              color: '#667eea',
              fontSize: '22px',
            }}>
              <FormOutlined />
            </div>
            <div>
              <div style={{ fontWeight: 600, fontSize: '16px', color: '#1a1a2e' }}>Whiteboard</div>
              <div style={{ fontSize: '13px', color: '#666' }}>Draw and sketch together in real-time</div>
            </div>
          </button>

          <button
            onClick={onDocument}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '16px',
              width: '100%',
              padding: '20px',
              borderRadius: '12px',
              border: '2px solid #e8e8e8',
              background: 'white',
              cursor: 'pointer',
              transition: 'all 150ms ease',
              textAlign: 'left',
            }}
            onMouseOver={(e) => {
              e.currentTarget.style.borderColor = '#764ba2';
              e.currentTarget.style.boxShadow = '0 4px 12px rgba(118, 75, 162, 0.2)';
            }}
            onMouseOut={(e) => {
              e.currentTarget.style.borderColor = '#e8e8e8';
              e.currentTarget.style.boxShadow = 'none';
            }}
          >
            <div style={{
              width: '48px',
              height: '48px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: '#f5f0ff',
              borderRadius: '12px',
              color: '#764ba2',
              fontSize: '22px',
            }}>
              <FileTextOutlined />
            </div>
            <div>
              <div style={{ fontWeight: 600, fontSize: '16px', color: '#1a1a2e' }}>Documents</div>
              <div style={{ fontSize: '13px', color: '#666' }}>Edit text documents together like Google Docs</div>
            </div>
          </button>

          <button
            onClick={onLatex}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '16px',
              width: '100%',
              padding: '20px',
              borderRadius: '12px',
              border: '2px solid #e8e8e8',
              marginTop: '12px',
              background: 'white',
              cursor: 'pointer',
              transition: 'all 150ms ease',
              textAlign: 'left',
            }}
            onMouseOver={(e) => {
              e.currentTarget.style.borderColor = '#0d9488';
              e.currentTarget.style.boxShadow = '0 4px 12px rgba(13, 148, 136, 0.2)';
            }}
            onMouseOut={(e) => {
              e.currentTarget.style.borderColor = '#e8e8e8';
              e.currentTarget.style.boxShadow = 'none';
            }}
          >
            <div style={{
              width: '48px',
              height: '48px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: '#f0fdf9',
              borderRadius: '12px',
              color: '#0d9488',
              fontSize: '22px',
              fontWeight: 700,
              fontFamily: 'serif',
            }}>
              Σ
            </div>
            <div>
              <div style={{ fontWeight: 600, fontSize: '16px', color: '#1a1a2e' }}>LaTeX Editor</div>
              <div style={{ fontSize: '13px', color: '#666' }}>Write and compile LaTeX with live PDF preview</div>
            </div>
          </button>
        </div>
      </Card>
    </div>
  );
}

interface WrapperProps {
  onBack: () => void;
}

function RoomSelectorWrapper({ onBack }: WrapperProps) {
  return (
    <div>
      <button
        onClick={onBack}
        style={{
          position: 'fixed',
          top: '12px',
          left: '12px',
          zIndex: 500,
          padding: '8px 16px',
          borderRadius: '8px',
          border: '1px solid #ddd',
          background: 'rgba(255,255,255,0.95)',
          backdropFilter: 'blur(8px)',
          cursor: 'pointer',
          fontSize: '13px',
          fontWeight: 500,
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
        }}
      >
        ← Home
      </button>
      <RoomSelector />
    </div>
  );
}

function WhiteboardPageWrapper({ onBack }: WrapperProps) {
  const roomId = useRoomStore((s) => s.roomId);

  const shareRoom = () => {
    const url = `${window.location.origin}/room/${roomId}`;
    navigator.clipboard.writeText(url).then(() => {
      message.success('Room link copied! Share it so others can join.');
    });
  };

  return (
    <div>
      <button
        onClick={onBack}
        style={{
          position: 'fixed',
          top: '12px',
          left: '12px',
          zIndex: 500,
          padding: '8px 16px',
          borderRadius: '8px',
          border: '1px solid #ddd',
          background: 'rgba(255,255,255,0.95)',
          backdropFilter: 'blur(8px)',
          cursor: 'pointer',
          fontSize: '13px',
          fontWeight: 500,
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
        }}
      >
        ← Rooms
      </button>
      <button
        onClick={shareRoom}
        style={{
          position: 'fixed',
          top: '12px',
          right: '12px',
          zIndex: 500,
          padding: '8px 16px',
          borderRadius: '8px',
          border: '1px solid #ddd',
          background: 'rgba(255,255,255,0.95)',
          backdropFilter: 'blur(8px)',
          cursor: 'pointer',
          fontSize: '13px',
          fontWeight: 500,
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
        }}
      >
        🔗 Share room
      </button>
      <WhiteboardPage />
    </div>
  );
}

function DocCabinetWrapper({ onBack }: WrapperProps) {
  return (
    <div>
      <button
        onClick={onBack}
        style={{
          position: 'fixed',
          top: '12px',
          left: '12px',
          zIndex: 500,
          padding: '8px 16px',
          borderRadius: '8px',
          border: '1px solid #ddd',
          background: 'rgba(255,255,255,0.95)',
          backdropFilter: 'blur(8px)',
          cursor: 'pointer',
          fontSize: '13px',
          fontWeight: 500,
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
        }}
      >
        ← Home
      </button>
      <DocSelector />
    </div>
  );
}

function DocEditorPageWrapper({ onBack }: WrapperProps) {
  return (
    <div>
      <button
        onClick={onBack}
        style={{
          position: 'fixed',
          top: '12px',
          left: '12px',
          zIndex: 500,
          padding: '8px 16px',
          borderRadius: '8px',
          border: '1px solid #ddd',
          background: 'rgba(255,255,255,0.95)',
          backdropFilter: 'blur(8px)',
          cursor: 'pointer',
          fontSize: '13px',
          fontWeight: 500,
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
        }}
      >
        ← Back to Cabinet
      </button>
      <DocEditorPage />
    </div>
  );
}