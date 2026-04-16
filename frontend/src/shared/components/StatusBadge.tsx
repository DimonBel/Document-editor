import './StatusBadge.css';

type Status = 'connected' | 'disconnected' | 'reconnecting';

interface StatusBadgeProps {
  status: Status;
}

const LABELS: Record<Status, string> = {
  connected: 'Connected',
  disconnected: 'Offline',
  reconnecting: 'Reconnecting…',
};

export function StatusBadge({ status }: StatusBadgeProps) {
  return (
    <span className={`status-badge status-badge--${status}`} role="status">
      <span className="status-badge__dot" aria-hidden />
      {LABELS[status] ?? status}
    </span>
  );
}