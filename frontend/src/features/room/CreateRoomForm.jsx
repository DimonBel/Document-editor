import React, { useState } from 'react';
import { Input, Button, Space, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';

/**
 * Controlled form for creating a new room.
 * Calls `onCreated({ id, name })` on success.
 */
export function CreateRoomForm({ onCreated, createRoom }) {
  const [name,    setName]    = useState('');
  const [loading, setLoading] = useState(false);

  const submit = async () => {
    const trimmed = name.trim();
    if (!trimmed) { message.warning('Enter a room name'); return; }

    setLoading(true);
    try {
      const room = await createRoom(trimmed);
      setName('');
      onCreated(room);
    } catch (e) {
      message.error(`Could not create room: ${e.message}`);
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
