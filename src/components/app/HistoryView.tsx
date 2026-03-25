import { useHistoryStore } from "@/stores/historyStore";
import type { HistoryEntry } from "@/types/notification";

function formatRelativeTime(isoDate: string): string {
  const now = Date.now();
  const then = new Date(isoDate).getTime();
  const diffMs = now - then;
  const diffMin = Math.floor(diffMs / 60_000);

  if (diffMin < 1) return "just now";
  if (diffMin < 60) return `${diffMin}m ago`;

  const diffHours = Math.floor(diffMin / 60);
  if (diffHours < 24) return `${diffHours}h ago`;

  const diffDays = Math.floor(diffHours / 24);
  if (diffDays === 1) return "yesterday";
  return `${diffDays}d ago`;
}

function HistoryRow({ entry }: { readonly entry: HistoryEntry }) {
  return (
    <div data-testid="history-row" className="history-row">
      <span
        className="priority-dot"
        data-priority={entry.priority}
      />
      <span className="history-sender">{entry.sender}</span>
      <span className="history-title">{entry.title}</span>
      <span className="history-time">
        {formatRelativeTime(entry.createdAt)}
      </span>
    </div>
  );
}

export function HistoryView() {
  const entries = useHistoryStore((s) => s.entries);

  if (entries.length === 0) {
    return (
      <div data-testid="history-empty" className="history-empty">
        <p>No notifications yet</p>
        <p className="history-empty-sub">
          Notifications sent to syncfu will appear here.
        </p>
      </div>
    );
  }

  return (
    <div data-testid="history-view" className="history-view">
      <div className="history-list">
        {entries.map((entry) => (
          <HistoryRow key={entry.id} entry={entry} />
        ))}
      </div>
    </div>
  );
}
