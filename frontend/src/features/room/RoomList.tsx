import { List, Badge, Typography, Button, Spin } from 'antd';
import { TeamOutlined, ReloadOutlined } from '@ant-design/icons';
import { RoomInfo } from '../../types';

const { Text } = Typography;

interface RoomListProps {
  rooms: RoomInfo[];
  loading: boolean;
  error: string | null;
  onJoin: (room: RoomInfo) => void;
  onRefresh: () => void;
}

export function RoomList({ rooms, loading, error, onJoin, onRefresh }: RoomListProps) {
  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: '24px 0' }}>
        <Spin tip="Loading rooms…" />
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

  if (!rooms.length) {
    return (
      <Text type="secondary" style={{ display: 'block', textAlign: 'center', padding: '16px 0' }}>
        No active rooms — create one above.
      </Text>
    );
  }

  return (
    <List
      size="small"
      dataSource={rooms}
      renderItem={(room) => (
        <List.Item
          key={room.id}
          actions={[
            <Button
              type="link"
              size="small"
              icon={<TeamOutlined />}
              onClick={() => onJoin(room)}
            >
              Join
            </Button>,
          ]}
        >
          <List.Item.Meta
            title={<span style={{ fontWeight: 600 }}>{room.name}</span>}
            description={
              <span style={{ fontSize: 11, color: '#9ca3af' }}>
                {room.id}
                &nbsp;·&nbsp;
                <Badge status="processing" />
                {room.client_count ?? 0} online
              </span>
            }
          />
        </List.Item>
      )}
    />
  );
}