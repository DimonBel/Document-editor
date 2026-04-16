import React from 'react';
import { Avatar, Tooltip } from 'antd';
import { UserOutlined } from '@ant-design/icons';
import './UsersList.css';

const PALETTE = [
  '#3b82f6', '#ef4444', '#10b981', '#f59e0b',
  '#8b5cf6', '#14b8a6', '#f97316', '#ec4899',
];

function avatarColor(id) {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) | 0;
  }
  return PALETTE[Math.abs(hash) % PALETTE.length];
}

export default function UsersList({ users, currentUserId }) {
  return (
    <div className="users-list">
      {users.map((user) => (
        <Tooltip
          key={user.id}
          title={user.id === currentUserId ? `${user.name} (you)` : user.name}
          placement="left"
        >
          <Avatar
            style={{
              backgroundColor: avatarColor(user.id),
              outline: user.id === currentUserId ? '2px solid #1d1d1d' : 'none',
              cursor: 'default',
            }}
            icon={<UserOutlined />}
            size={32}
          >
            {user.name?.[0]?.toUpperCase()}
          </Avatar>
        </Tooltip>
      ))}
    </div>
  );
}
