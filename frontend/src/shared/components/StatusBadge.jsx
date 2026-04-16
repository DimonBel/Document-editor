import React from 'react';
import './StatusBadge.css';

/**
 * Simple status pill used in the header.
 * @param {'connected'|'disconnected'|'reconnecting'} status
 */
export function StatusBadge({ status }) {
  const LABELS = {
    connected:    'Connected',
    disconnected: 'Offline',
    reconnecting: 'Reconnecting…',
  };

  return (
    <span className={`status-badge status-badge--${status}`} role="status">
      <span className="status-badge__dot" aria-hidden />
      {LABELS[status] ?? status}
    </span>
  );
}
