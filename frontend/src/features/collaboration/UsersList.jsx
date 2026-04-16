import React from 'react';
import { Avatar, Tooltip } from 'antd';
import { useCollabStore } from '../../store/collabStore.js';
import { useRoomStore   } from '../../store/roomStore.js';
import './UsersList.css';

const PALETTE = [
  '#3b82f6','#ef4444','#10b981','#f59e0b',
  '#8b5cf6','#14b8a6','#f97316','#ec4899',
];

function avatarColor(id = '') {
  let h = 0;
  for (let i = 0; i < id.length; i++) h = (Math.imul(31, h) + id.charCodeAt(i)) | 0;
  return PALETTE[Math.abs(h) % PALETTE.length];
}

export function UsersList() {
  const { users }    = useCollabStore();
  const { clientId } = useRoomStore();

  if (!users.length) return null;

  return (
    <aside className="users-list" aria-label="Online users">
      {users.map((user) => {
        const isMe = user.id === clientId;
        return (
          <Tooltip
            key={user.id}
            title={isMe ? `${user.name} (you)` : user.name}
            placement="left"
          >
            <Avatar
              className={`user-avatar${isMe ? ' user-avatar--me' : ''}`}
              style={{ backgroundColor: avatarColor(user.id) }}
              size={34}
            >
              {user.name?.[0]?.toUpperCase() ?? '?'}
            </Avatar>
          </Tooltip>
        );
      })}
    </aside>
  );
}
