import { useState } from 'react';
import { Card, Input, Button, Divider, Space, Typography, message } from 'antd';
import { EnterOutlined } from '@ant-design/icons';
import { useRoomStore } from '../../store/roomStore';
import { useRoomList } from './useRoomList';
import { CreateRoomForm } from './CreateRoomForm';
import { RoomList } from './RoomList';
import { RoomInfo } from '../../types';
import './RoomSelector.css';

const { Title, Text } = Typography;

export function RoomSelector() {
  const { clientName, joinRoom } = useRoomStore();
  const { rooms, loading, error, createRoom, getRoom, refresh } = useRoomList();
  const [joinId, setJoinId] = useState('');
  const [joining, setJoining] = useState(false);

  const handleJoin = (room: RoomInfo) => joinRoom({ roomId: room.id, roomName: room.name });

  const handleJoinById = async () => {
    const id = joinId.trim();
    if (!id) {
      message.warning('Paste a room ID');
      return;
    }
    setJoining(true);
    try {
      const room = await getRoom(id);
      handleJoin(room);
    } catch {
      message.error('Room not found or server is unavailable.');
    } finally {
      setJoining(false);
    }
  };

  return (
    <div className="rs-shell">
      <Card className="rs-card" bordered={false}>
        <header className="rs-hero">
          <div className="rs-logo" aria-hidden>✏️</div>
          <Title level={2} className="rs-title">
            Collaborative Whiteboard
          </Title>
          <Text type="secondary">Welcome, <strong>{clientName}</strong></Text>
        </header>

        <div className="rs-body">
          <section>
            <Text className="rs-section-label">Create a room</Text>
            <CreateRoomForm createRoom={createRoom} onCreated={handleJoin} />
          </section>

          <section>
            <Text className="rs-section-label">Join by ID</Text>
            <Space.Compact style={{ width: '100%' }}>
              <Input
                prefix={<EnterOutlined />}
                placeholder="Paste room ID…"
                value={joinId}
                onChange={(e) => setJoinId(e.target.value)}
                onPressEnter={handleJoinById}
              />
              <Button onClick={handleJoinById} loading={joining}>
                Join
              </Button>
            </Space.Compact>
          </section>

          <Divider plain style={{ margin: '8px 0', fontSize: 12, color: '#9ca3af' }}>
            active rooms
          </Divider>

          <RoomList
            rooms={rooms}
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