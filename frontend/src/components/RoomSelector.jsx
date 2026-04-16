import React, { useState, useEffect } from 'react';
import {
  Card, Button, Input, List, Typography,
  Space, Divider, message, Spin, Badge,
} from 'antd';
import { PlusOutlined, EnterOutlined, TeamOutlined } from '@ant-design/icons';
import { createRoom, listRooms, getRoom } from '../services/api.js';
import './RoomSelector.css';

const { Title, Text } = Typography;

export default function RoomSelector({ onJoin, clientName }) {
  const [rooms, setRooms] = useState([]);
  const [loading, setLoading] = useState(true);
  const [newName, setNewName] = useState('');
  const [joinId, setJoinId] = useState('');
  const [creating, setCreating] = useState(false);
  const [joining, setJoining] = useState(false);

  useEffect(() => {
    fetchRooms();
    const timer = setInterval(fetchRooms, 6000);
    return () => clearInterval(timer);
  }, []);

  async function fetchRooms() {
    try {
      setRooms(await listRooms());
    } catch {
      // Server may not be up yet — silently retry
    } finally {
      setLoading(false);
    }
  }

  async function handleCreate() {
    if (!newName.trim()) { message.warning('Enter a room name'); return; }
    setCreating(true);
    try {
      const room = await createRoom(newName.trim());
      onJoin(room.id, room.name);
    } catch (e) {
      message.error(`Failed to create room: ${e.message}`);
    } finally {
      setCreating(false);
    }
  }

  async function handleJoinById() {
    if (!joinId.trim()) { message.warning('Enter a room ID'); return; }
    setJoining(true);
    try {
      const room = await getRoom(joinId.trim());
      onJoin(room.id, room.name);
    } catch {
      message.error('Room not found');
    } finally {
      setJoining(false);
    }
  }

  return (
    <div className="rs-root">
      <Card className="rs-card">
        <Title level={2} className="rs-title">Collaborative Whiteboard</Title>
        <Text type="secondary" className="rs-subtitle">
          Welcome, <strong>{clientName}</strong>
        </Text>

        <Space direction="vertical" style={{ width: '100%', marginTop: 20 }} size="middle">
          {/* Create */}
          <Card size="small" title={<><PlusOutlined /> Create a room</>}>
            <Space.Compact style={{ width: '100%' }}>
              <Input
                placeholder="Room name…"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                onPressEnter={handleCreate}
                maxLength={60}
              />
              <Button type="primary" onClick={handleCreate} loading={creating}>
                Create
              </Button>
            </Space.Compact>
          </Card>

          {/* Join by ID */}
          <Card size="small" title={<><EnterOutlined /> Join by room ID</>}>
            <Space.Compact style={{ width: '100%' }}>
              <Input
                placeholder="Paste room ID…"
                value={joinId}
                onChange={(e) => setJoinId(e.target.value)}
                onPressEnter={handleJoinById}
              />
              <Button onClick={handleJoinById} loading={joining}>
                Join
              </Button>
            </Space.Compact>
          </Card>

          <Divider style={{ margin: '4px 0' }}>Active rooms</Divider>

          {loading ? (
            <div className="rs-spinner"><Spin /></div>
          ) : rooms.length === 0 ? (
            <Text type="secondary" className="rs-empty">
              No active rooms — create one above!
            </Text>
          ) : (
            <List
              size="small"
              dataSource={rooms}
              renderItem={(room) => (
                <List.Item
                  actions={[
                    <Button
                      type="link"
                      icon={<TeamOutlined />}
                      onClick={() => onJoin(room.id, room.name)}
                    >
                      Join
                    </Button>,
                  ]}
                >
                  <List.Item.Meta
                    title={room.name}
                    description={
                      <>
                        <Text type="secondary" style={{ fontSize: 11 }}>
                          {room.id}
                        </Text>
                        {'  ·  '}
                        <Badge
                          status="processing"
                          text={
                            <Text type="secondary" style={{ fontSize: 11 }}>
                              {room.clientCount} online
                            </Text>
                          }
                        />
                      </>
                    }
                  />
                </List.Item>
              )}
            />
          )}
        </Space>
      </Card>
    </div>
  );
}
