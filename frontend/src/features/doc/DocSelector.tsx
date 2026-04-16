import { useState } from 'react';
import { Card, Input, Button, Typography, Empty, Spin } from 'antd';
import { PlusOutlined, SearchOutlined, FileTextOutlined, TeamOutlined } from '@ant-design/icons';
import { useDocStore } from '../../store/docStore';
import { useDocList } from './useDocList';
import { CreateDocForm } from './CreateDocForm';
import { DocInfo } from '../../types';
import './DocSelector.css';

const { Title, Text } = Typography;

export function DocSelector() {
  const { clientName, joinDoc } = useDocStore();
  const { docs, loading, error, createDoc, refresh } = useDocList();
  const [searchTerm, setSearchTerm] = useState('');
  const [showCreate, setShowCreate] = useState(false);

  const handleJoin = (doc: DocInfo) => joinDoc(doc.id, doc.title);

  const filteredDocs = docs.filter(doc => 
    doc.title.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="doc-cabinet">
      <header className="doc-cabinet__header">
        <div className="doc-cabinet__header-left">
          <div className="doc-cabinet__logo">
            <FileTextOutlined />
          </div>
          <Title level={3} className="doc-cabinet__title">My Documents</Title>
        </div>
        <div className="doc-cabinet__header-right">
          <Text type="secondary" className="doc-cabinet__user">
            <TeamOutlined /> {clientName}
          </Text>
        </div>
      </header>

      <div className="doc-cabinet__toolbar">
        <Input
          className="doc-cabinet__search"
          placeholder="Search documents..."
          prefix={<SearchOutlined />}
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
        <Button 
          type="primary" 
          icon={<PlusOutlined />} 
          onClick={() => setShowCreate(true)}
        >
          New Document
        </Button>
      </div>

      {showCreate && (
        <div className="doc-cabinet__create-modal">
          <div className="doc-cabinet__create-content">
            <Title level={4}>Create new document</Title>
            <CreateDocForm 
              createDoc={createDoc} 
              onCreated={(doc) => {
                setShowCreate(false);
                handleJoin(doc);
              }}
              onCancel={() => setShowCreate(false)}
            />
          </div>
        </div>
      )}

      <div className="doc-cabinet__content">
        {loading ? (
          <div className="doc-cabinet__loading">
            <Spin size="large" tip="Loading documents..." />
          </div>
        ) : error ? (
          <div className="doc-cabinet__error">
            <Text type="danger">{error}</Text>
            <Button onClick={refresh}>Retry</Button>
          </div>
        ) : filteredDocs.length === 0 ? (
          <Empty 
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description={searchTerm ? "No documents match your search" : "No documents yet"}
          >
            {!searchTerm && (
              <Button type="primary" onClick={() => setShowCreate(true)}>
                Create your first document
              </Button>
            )}
          </Empty>
        ) : (
          <div className="doc-cabinet__grid">
            {filteredDocs.map(doc => (
              <Card
                key={doc.id}
                className="doc-cabinet__card"
                onClick={() => handleJoin(doc)}
                hoverable
              >
                <div className="doc-card__icon">
                  <FileTextOutlined />
                </div>
                <Title level={5} className="doc-card__title">{doc.title}</Title>
                <div className="doc-card__meta">
                  <TeamOutlined /> {doc.client_count} online
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}