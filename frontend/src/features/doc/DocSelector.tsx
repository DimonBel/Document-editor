import { useState } from 'react';
import { Card, Input, Button, Space, Typography, message } from 'antd';
import { FileTextOutlined } from '@ant-design/icons';
import { useDocStore } from '../../store/docStore';
import { useDocList } from './useDocList';
import { CreateDocForm } from './CreateDocForm';
import { DocList } from './DocList';
import { DocInfo } from '../../types';
import './DocSelector.css';

const { Title, Text } = Typography;

export function DocSelector() {
  const { clientName, joinDoc } = useDocStore();
  const { docs, loading, error, createDoc, getDoc, refresh } = useDocList();
  const [joinId, setJoinId] = useState('');
  const [joining, setJoining] = useState(false);

  const handleJoin = (doc: DocInfo) => joinDoc(doc.id, doc.title);

  const handleJoinById = async () => {
    const id = joinId.trim();
    if (!id) {
      message.warning('Paste a document ID');
      return;
    }
    setJoining(true);
    try {
      const doc = await getDoc(id);
      handleJoin(doc);
    } catch {
      message.error('Document not found or server is unavailable.');
    } finally {
      setJoining(false);
    }
  };

  return (
    <div className="doc-shell">
      <Card className="doc-card" bordered={false}>
        <div className="doc-header">
          <div className="doc-logo">
            <FileTextOutlined />
          </div>
          <Title level={2} className="doc-title">
            Document Editor
          </Title>
          <Text type="secondary">Welcome, <strong>{clientName}</strong></Text>
        </div>

        <div className="doc-body">
          <div className="doc-section">
            <Text className="doc-label">Create a document</Text>
            <CreateDocForm createDoc={createDoc} onCreated={handleJoin} />
          </div>

          <div className="doc-section">
            <Text className="doc-label">Join by ID</Text>
            <Space.Compact style={{ width: '100%' }}>
              <Input
                prefix={<FileTextOutlined />}
                placeholder="Paste document ID…"
                value={joinId}
                onChange={(e) => setJoinId(e.target.value)}
                onPressEnter={handleJoinById}
              />
              <Button onClick={handleJoinById} loading={joining}>
                Join
              </Button>
            </Space.Compact>
          </div>

          <div className="doc-divider">active documents</div>

          <DocList
            docs={docs}
            loading={loading}
            error={error}
            onJoin={handleJoin}
            onRefresh={refresh}
          />
        </div>
      </Card>
    </div>
  );
}