import { useState, useEffect, useCallback, useRef } from "react";
import type { NotificationPayload } from "@/types/notification";
import { NotificationIcon } from "./NotificationIcon";
import { RelativeTime } from "./RelativeTime";
import { useGoogleFont } from "@/hooks/useGoogleFont";

/** Auto-dismiss timeouts by priority (ms). Critical never auto-dismisses. */
const TIMEOUTS: Record<string, number | null> = {
  low: 6000,
  normal: 8000,
  high: 12000,
  critical: null,
};

const DISMISS_ANIM_MS = 280;

interface NotificationCardProps {
  readonly notification: NotificationPayload;
  readonly onDismiss: (id: string) => void;
  readonly onAction: (notificationId: string, actionId: string) => void;
}

export function NotificationCard({
  notification,
  onDismiss,
  onAction,
}: NotificationCardProps) {
  const { id, sender, title, body, priority, actions, progress, theme, icon, font, timeout, createdAt } =
    notification;

  useGoogleFont(font);
  const [dismissing, setDismissing] = useState(false);
  const hovering = useRef(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Resolve auto-dismiss duration (null = never)
  const autoDismissMs = resolveTimeout(timeout, priority);

  // Animated dismiss: play slide-out, then remove from store
  const animatedDismiss = useCallback(() => {
    if (dismissing) return;
    setDismissing(true);
    setTimeout(() => onDismiss(id), DISMISS_ANIM_MS);
  }, [id, onDismiss, dismissing]);

  // Auto-dismiss timer
  useEffect(() => {
    if (autoDismissMs === null) return;

    const startTimer = () => {
      timerRef.current = setTimeout(() => {
        if (!hovering.current) {
          animatedDismiss();
        }
      }, autoDismissMs);
    };

    startTimer();
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [autoDismissMs, animatedDismiss]);

  const handleMouseEnter = useCallback(() => {
    hovering.current = true;
    if (timerRef.current) clearTimeout(timerRef.current);
  }, []);

  const handleMouseLeave = useCallback(() => {
    hovering.current = false;
    // Restart timer with full duration after hover
    if (autoDismissMs !== null && !dismissing) {
      timerRef.current = setTimeout(() => {
        animatedDismiss();
      }, autoDismissMs);
    }
  }, [autoDismissMs, animatedDismiss, dismissing]);

  const classNames = [
    "notification-card",
    priority,
    theme ?? "",
    dismissing ? "dismissing" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div
      data-testid="notification-card"
      className={classNames}
      style={font ? { fontFamily: `"${font}", sans-serif` } : undefined}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <button
        className="notification-dismiss"
        onClick={() => animatedDismiss()}
        aria-label="Dismiss"
      >
        <svg width="8" height="8" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeLinecap="round" strokeWidth="2.5">
          <line x1="2" y1="2" x2="8" y2="8" />
          <line x1="8" y1="2" x2="2" y2="8" />
        </svg>
      </button>

      <div className="notification-content-row">
        {icon && (
          <div className="notification-icon">
            <NotificationIcon name={icon} size={20} strokeWidth={1.8} />
          </div>
        )}
        <div className="notification-text">
          <div className="notification-header">
            <span className="notification-sender">{sender}</span>
            <RelativeTime iso={createdAt} />
          </div>
          <div className="notification-title">{title}</div>
          <div className="notification-body">{body}</div>
        </div>
      </div>

      {progress && (
        <div className="notification-progress">
          <div
            role="progressbar"
            aria-valuenow={Math.round(progress.value * 100)}
            aria-valuemin={0}
            aria-valuemax={100}
            className={`progress-${progress.style}`}
          >
            <div
              className="progress-fill"
              style={{ width: `${progress.value * 100}%` }}
            />
          </div>
          {progress.label && (
            <span className="progress-label">{progress.label}</span>
          )}
        </div>
      )}

      {actions.length > 0 && (
        <div className="notification-actions">
          {actions.map((action) => (
            <button
              key={action.id}
              className={`action-button ${action.style}`}
              onClick={() => onAction(id, action.id)}
            >
              {action.label}
            </button>
          ))}
        </div>
      )}

      {autoDismissMs !== null && (
        <div className="notification-countdown">
          <div
            className="countdown-fill"
            style={{ "--countdown-duration": `${autoDismissMs}ms` } as React.CSSProperties}
          />
        </div>
      )}
    </div>
  );
}

function resolveTimeout(
  timeout: NotificationPayload["timeout"],
  priority: string,
): number | null {
  if (timeout === "never") return null;
  if (timeout === "default") return TIMEOUTS[priority] ?? 8000;
  if (typeof timeout === "object" && timeout.never) return null;
  if (typeof timeout === "object" && timeout.seconds) return timeout.seconds * 1000;
  return TIMEOUTS[priority] ?? 8000;
}
