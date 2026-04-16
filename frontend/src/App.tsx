import { useState, useEffect } from 'react';
import { Card, Typography, message } from 'antd';
import { FormOutlined, FileTextOutlined } from '@ant-design/icons';
import { WhiteboardPage } from './pages/WhiteboardPage';
import { DocEditorPage } from './pages/DocEditorPage';
import { DocSelector } from './features/doc/DocSelector';
import { ErrorBoundary } from './shared/components/ErrorBoundary';
import { useDocStore } from './store/docStore';
import { docsApi } from './core/api/docsApi';

const { Title, Text } = Typography;

type AppMode = 'home' | 'room' | 'doc-cabinet' | 'doc';

export default function App() {
  const [mode, setMode] = useState<AppMode>('home');
  const [loadingDoc, setLoadingDoc] = useState(false);

  const goToCabinet = () => {
    useDocStore.getState().leaveDoc();
    setMode('doc-cabinet');
    window.history.pushState({}, '', '/');
  };

  const goHome = () => {
    useDocStore.getState().leaveDoc();
    setMode('home');
  };

  const goToDoc = async (docId: string) => {
    setLoadingDoc(true);
    try {
      const doc = await docsApi.get(docId);
      useDocStore.getState().joinDoc(doc.id, doc.title);
      setMode('doc');
      window.history.pushState({}, '', `/doc/${docId}`);
    } catch (e) {
      console.error('Failed to load document:', e);
      message.error('Document not found');
      goToCabinet();
    } finally {
      setLoadingDoc(false);
    }
  };

  useEffect(() => {
    const path = window.location.pathname;
    const match = path.match(/^\/doc\/([a-zA-Z0-9-]+)$/);
    if (match) {
      goToDoc(match[1]);
    }
  }, []);

  useEffect(() => {
    const handlePopState = () => {
      const path = window.location.pathname;
      const match = path.match(/^\/doc\/([a-zA-Z0-9-]+)$/);
      if (match) {
        goToDoc(match[1]);
      } else {
        goToCabinet();
      }
    };
    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, []);

  const docId = useDocStore((s) => s.docId);
  useEffect(() => {
    if (docId && mode === 'doc-cabinet') setMode('doc');
  }, [docId, mode]);

  if (loadingDoc) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh' }}>
        <Typography.Text>Loading document...</Typography.Text>
      </div>
    );
  }

  return (
    <ErrorBoundary>
      {mode === 'home' && (
        <HomeScreen
          onWhiteboard={() => setMode('room')}
          onDocument={goToCabinet}
        />
      )}
      {mode === 'room' && <WhiteboardPageWrapper onBack={goHome} />}
      {mode === 'doc-cabinet' && <DocCabinetWrapper onBack={goHome} />}
      {mode === 'doc' && <DocEditorPageWrapper onBack={goToCabinet} />}
    </ErrorBoundary>
  );
}

interface HomeScreenProps {
  onWhiteboard: () => void;
  onDocument: () => void;
}

function HomeScreen({ onWhiteboard, onDocument }: HomeScreenProps) {
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
        </div>
      </Card>
    </div>
  );
}

interface WrapperProps {
  onBack: () => void;
}

function WhiteboardPageWrapper({ onBack }: WrapperProps) {
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