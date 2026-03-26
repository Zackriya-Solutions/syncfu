import { useState, useEffect } from "react";

/** Returns a short relative time string: "just now", "1m", "5m", "1h", "3h", "1d" */
function formatRelative(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  if (diff < 0) return "just now";

  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return "just now";

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;

  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;

  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

/** How often to re-render based on age */
function tickInterval(iso: string): number {
  const diff = Date.now() - new Date(iso).getTime();
  if (diff < 60_000) return 10_000;    // <1m: tick every 10s
  if (diff < 3_600_000) return 30_000; // <1h: tick every 30s
  return 60_000;                        // else: tick every 60s
}

interface RelativeTimeProps {
  readonly iso: string;
}

export function RelativeTime({ iso }: RelativeTimeProps) {
  const [text, setText] = useState(() => formatRelative(iso));

  useEffect(() => {
    setText(formatRelative(iso));
    const id = setInterval(() => {
      setText(formatRelative(iso));
    }, tickInterval(iso));
    return () => clearInterval(id);
  }, [iso]);

  return <span className="notification-time">{text}</span>;
}
