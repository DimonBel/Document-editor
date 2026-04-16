import React from 'react';
import { ConfigProvider, theme } from 'antd';
import { useRoomStore    } from './store/roomStore.js';
import { RoomSelector    } from './features/room/RoomSelector.jsx';
import { WhiteboardPage  } from './pages/WhiteboardPage.jsx';
import { ErrorBoundary   } from './shared/components/ErrorBoundary.jsx';

/**
 * App — top-level router.
 *
 * Reads `roomId` from the store; if null → show RoomSelector, else → WhiteboardPage.
 * This is the only place where "which screen is active" is decided.
 *
 * Adding a real router (React Router, TanStack Router) later is a one-line
 * change here: replace the ternary with <Routes> and keep everything else.
 */
export default function App() {
  const { roomId } = useRoomStore();

  return (
    <ConfigProvider
      theme={{
        algorithm: theme.defaultAlgorithm,
        token: {
          colorPrimary:   '#6366f1',
          borderRadius:   8,
          fontFamily:     '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        },
      }}
    >
      <ErrorBoundary>
        {roomId ? <WhiteboardPage /> : <RoomSelector />}
      </ErrorBoundary>
    </ConfigProvider>
  );
}
