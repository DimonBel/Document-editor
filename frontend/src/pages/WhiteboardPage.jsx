import React from 'react';
import { Button, Tooltip } from 'antd';
import { LogoutOutlined } from '@ant-design/icons';
import { useRoomStore         } from '../store/roomStore.js';
import { useCollaboration     } from '../features/collaboration/useCollaboration.js';
import { WhiteboardCanvas     } from '../features/canvas/WhiteboardCanvas.jsx';
import { Toolbar              } from '../features/toolbar/Toolbar.jsx';
import { UsersList            } from '../features/collaboration/UsersList.jsx';
import { StatusBadge          } from '../shared/components/StatusBadge.jsx';
import { ErrorBoundary        } from '../shared/components/ErrorBoundary.jsx';
import './WhiteboardPage.css';

/**
 * WhiteboardPage — the "whiteboard session" screen.
 *
 * Responsibilities:
 *  1. Mount the collaboration hook (WS + CRDT).
 *  2. Compose feature components into the page layout.
 *  3. Render the app header with room name, status, and leave button.
 *
 * Intentionally thin — all business logic lives in hooks / stores / features.
 */
export function WhiteboardPage() {
  const { roomName, connected, leaveRoom } = useRoomStore();

  // Bootstrap networking; returns stable callbacks for the canvas.
  const { addElement, sendCursor } = useCollaboration();

  const connectionStatus = connected ? 'connected' : 'reconnecting';

  return (
    <div className="wb-page">
      {/* ── Header ─────────────────────────────────────────────────────── */}
      <header className="wb-header">
        <span className="wb-header__room">{roomName}</span>

        <StatusBadge status={connectionStatus} />

        <Tooltip title="Leave room" placement="bottom">
          <Button
            className="wb-header__leave"
            icon={<LogoutOutlined />}
            onClick={leaveRoom}
            aria-label="Leave room"
          />
        </Tooltip>
      </header>

      {/* ── Drawing toolbar (floating) ──────────────────────────────────── */}
      <Toolbar />

      {/* ── Online users (floating sidebar) ────────────────────────────── */}
      <UsersList />

      {/* ── Canvas ─────────────────────────────────────────────────────── */}
      <ErrorBoundary>
        <WhiteboardCanvas addElement={addElement} sendCursor={sendCursor} />
      </ErrorBoundary>
    </div>
  );
}
