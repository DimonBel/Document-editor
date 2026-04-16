import { ConfigProvider, theme } from 'antd';
import { useRoomStore } from './store/roomStore';
import { RoomSelector } from './features/room/RoomSelector';
import { WhiteboardPage } from './pages/WhiteboardPage';
import { ErrorBoundary } from './shared/components/ErrorBoundary';

export default function App() {
  const { roomId } = useRoomStore();

  return (
    <ConfigProvider
      theme={{
        algorithm: theme.defaultAlgorithm,
        token: {
          colorPrimary: '#6366f1',
          borderRadius: 8,
          fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        },
      }}
    >
      <ErrorBoundary>
        {roomId ? <WhiteboardPage /> : <RoomSelector />}
      </ErrorBoundary>
    </ConfigProvider>
  );
}