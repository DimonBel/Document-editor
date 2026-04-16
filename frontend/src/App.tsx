import { useState } from 'react';
import { Card, Typography } from 'antd';
import { FormOutlined, FileTextOutlined } from '@ant-design/icons';
import { WhiteboardPage } from './pages/WhiteboardPage';
import { DocEditorPage } from './pages/DocEditorPage';
import { ErrorBoundary } from './shared/components/ErrorBoundary';

const { Title, Text } = Typography;

type AppMode = 'home' | 'room' | 'doc';

export default function App() {
  const [mode, setMode] = useState<AppMode>('home');

  return (
    <ErrorBoundary>
      {mode === 'home' && (
        <HomeScreen
          onWhiteboard={() => setMode('room')}
          onDocument={() => setMode('doc')}
        />
      )}
      {mode === 'room' && <WhiteboardPageWrapper onBack={() => setMode('home')} />}
      {mode === 'doc' && <DocEditorPageWrapper onBack={() => setMode('home')} />}
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
      background: 'var(--color-neutral-100)',
    }}>
      <Card style={{
        width: '100%',
        maxWidth: '400px',
        background: 'var(--color-neutral-0)',
        borderRadius: 'var(--radius-xl)',
        boxShadow: 'var(--shadow-lg)',
        border: '1px solid var(--color-neutral-200)',
        overflow: 'hidden',
      }}>
        <div style={{
          padding: '48px 24px 32px',
          textAlign: 'center',
          borderBottom: '1px solid var(--color-neutral-100)',
        }}>
          <div style={{
            width: '48px',
            height: '48px',
            margin: '0 auto 16px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            background: 'var(--color-brand-50)',
            borderRadius: 'var(--radius-md)',
            color: 'var(--color-brand-600)',
            fontSize: '24px',
          }}>
            <FileTextOutlined />
          </div>
          <Title level={2} style={{ marginBottom: '4px' }}>
            Collaboration Hub
          </Title>
          <Text type="secondary">Choose what you want to use</Text>
        </div>

        <div style={{ padding: '24px' }}>
          <button
            onClick={onWhiteboard}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              width: '100%',
              padding: '16px',
              borderRadius: 'var(--radius-lg)',
              border: '1px solid var(--color-neutral-200)',
              marginBottom: '12px',
              background: 'var(--color-neutral-0)',
              cursor: 'pointer',
              transition: 'all 120ms ease',
              textAlign: 'left',
            }}
          >
            <div style={{
              width: '40px',
              height: '40px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: 'var(--color-brand-50)',
              borderRadius: 'var(--radius-md)',
              color: 'var(--color-brand-600)',
              fontSize: '20px',
            }}>
              <FormOutlined />
            </div>
            <div>
              <div style={{ fontWeight: 600, color: 'var(--color-neutral-900)' }}>Whiteboard</div>
              <div style={{ fontSize: '12px', color: 'var(--color-neutral-500)' }}>Draw and sketch together</div>
            </div>
          </button>

          <button
            onClick={onDocument}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              width: '100%',
              padding: '16px',
              borderRadius: 'var(--radius-lg)',
              border: '1px solid var(--color-neutral-200)',
              background: 'var(--color-neutral-0)',
              cursor: 'pointer',
              transition: 'all 120ms ease',
              textAlign: 'left',
            }}
          >
            <div style={{
              width: '40px',
              height: '40px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: 'var(--color-brand-50)',
              borderRadius: 'var(--radius-md)',
              color: 'var(--color-brand-600)',
              fontSize: '20px',
            }}>
              <FileTextOutlined />
            </div>
            <div>
              <div style={{ fontWeight: 600, color: 'var(--color-neutral-900)' }}>Documents</div>
              <div style={{ fontSize: '12px', color: 'var(--color-neutral-500)' }}>Edit text documents together</div>
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
          borderRadius: 'var(--radius-md)',
          border: '1px solid var(--color-neutral-200)',
          background: 'rgba(255,255,255,0.9)',
          backdropFilter: 'blur(8px)',
          cursor: 'pointer',
          fontSize: '13px',
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
          borderRadius: 'var(--radius-md)',
          border: '1px solid var(--color-neutral-200)',
          background: 'rgba(255,255,255,0.9)',
          backdropFilter: 'blur(8px)',
          cursor: 'pointer',
          fontSize: '13px',
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
        }}
      >
        ← Home
      </button>
      <DocEditorPage />
    </div>
  );
}