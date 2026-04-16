import { useState } from 'react';
import { Input, Button, Space, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { DocInfo } from '../../types';

interface CreateDocFormProps {
  onCreated: (doc: DocInfo) => void;
  createDoc: (title: string) => Promise<DocInfo>;
}

export function CreateDocForm({ onCreated, createDoc }: CreateDocFormProps) {
  const [title, setTitle] = useState('');
  const [loading, setLoading] = useState(false);

  const submit = async () => {
    const trimmed = title.trim();
    if (!trimmed) {
      message.warning('Enter a document title');
      return;
    }

    setLoading(true);
    try {
      const doc = await createDoc(trimmed);
      setTitle('');
      onCreated(doc);
    } catch (e) {
      message.error(`Could not create document: ${e instanceof Error ? e.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Space.Compact style={{ width: '100%' }}>
      <Input
        prefix={<PlusOutlined />}
        placeholder="New document title…"
        value={title}
        maxLength={60}
        onChange={(e) => setTitle(e.target.value)}
        onPressEnter={submit}
      />
      <Button type="primary" onClick={submit} loading={loading}>
        Create
      </Button>
    </Space.Compact>
  );
}