import { useState } from 'react';
import { Input, Button, Space, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { RoomInfo } from '../../types';

interface CreateRoomFormProps {
  onCreated: (room: RoomInfo) => void;
  createRoom: (name: string) => Promise<RoomInfo>;
}

export function CreateRoomForm({ onCreated, createRoom }: CreateRoomFormProps) {
  const [name, setName] = useState('');
  const [loading, setLoading] = useState(false);

  const submit = async () => {
    const trimmed = name.trim();
    if (!trimmed) {
      message.warning('Enter a room name');
      return;
    }

    setLoading(true);
    try {
      const room = await createRoom(trimmed);
      setName('');
      onCreated(room);
    } catch (e) {
      message.error(`Could not create room: ${e instanceof Error ? e.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Space.Compact style={{ width: '100%' }}>
      <Input
        prefix={<PlusOutlined />}
        placeholder="New room name…"
        value={name}
        maxLength={60}
        onChange={(e) => setName(e.target.value)}
        onPressEnter={submit}
      />
      <Button type="primary" onClick={submit} loading={loading}>
        Create
      </Button>
    </Space.Compact>
  );
}