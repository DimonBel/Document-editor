import { List, Typography, Button, Spin } from 'antd';
import { FileTextOutlined, ReloadOutlined } from '@ant-design/icons';
import { DocInfo } from '../../types';

const { Text } = Typography;

interface DocListProps {
  docs: DocInfo[];
  loading: boolean;
  error: string | null;
  onJoin: (doc: DocInfo) => void;
  onRefresh: () => void;
}

export function DocList({ docs, loading, error, onJoin, onRefresh }: DocListProps) {
  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: '24px 0' }}>
        <Spin tip="Loading documents…" />
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ textAlign: 'center', padding: '16px 0' }}>
        <Text type="danger" style={{ display: 'block', marginBottom: 8 }}>
          {error}
        </Text>
        <Button size="small" icon={<ReloadOutlined />} onClick={onRefresh}>
          Retry
        </Button>
      </div>
    );
  }

  if (!docs.length) {
    return (
      <Text type="secondary" style={{ display: 'block', textAlign: 'center', padding: '16px 0' }}>
        No documents — create one above.
      </Text>
    );
  }

  return (
    <List
      size="small"
      dataSource={docs}
      renderItem={(doc) => (
        <List.Item
          key={doc.id}
          actions={[
            <Button
              type="link"
              size="small"
              icon={<FileTextOutlined />}
              onClick={() => onJoin(doc)}
            >
              Open
            </Button>,
          ]}
        >
          <List.Item.Meta
            title={<span style={{ fontWeight: 600 }}>{doc.title}</span>}
            description={
              <span style={{ fontSize: 11, color: '#a1a1aa' }}>
                {doc.id.slice(0, 8)}… · {doc.client_count} online
              </span>
            }
          />
        </List.Item>
      )}
    />
  );
}