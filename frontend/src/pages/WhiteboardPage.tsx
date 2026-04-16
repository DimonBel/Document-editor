import { Button, Tooltip } from 'antd';
import { LogoutOutlined } from '@ant-design/icons';
import { useRoomStore } from '../store/roomStore';
import { useCollaboration } from '../features/collaboration/useCollaboration';
import { WhiteboardCanvas } from '../features/canvas/WhiteboardCanvas';
import { Toolbar } from '../features/toolbar/Toolbar';
import { UsersList } from '../features/collaboration/UsersList';
import { StatusBadge } from '../shared/components/StatusBadge';
import { ErrorBoundary } from '../shared/components/ErrorBoundary';
import './WhiteboardPage.css';

export function WhiteboardPage() {
  const { roomName, connected, leaveRoom } = useRoomStore();

  const { addElement, sendCursor, sendPreview } = useCollaboration();

  const connectionStatus = connected ? 'connected' : 'reconnecting';

  return (
    <div className="wb-page">
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

      <Toolbar />

      <UsersList />

      <ErrorBoundary>
        <WhiteboardCanvas addElement={addElement} sendCursor={sendCursor} sendPreview={sendPreview} />
      </ErrorBoundary>
    </div>
  );
}