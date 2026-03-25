import type { NotificationPayload } from "@/types/notification";

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
  const { id, sender, title, body, priority, actions, progress, theme } =
    notification;

  const classNames = [
    "notification-card",
    priority,
    theme ?? "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div data-testid="notification-card" className={classNames}>
      <div className="notification-header">
        <span className="notification-sender">{sender}</span>
        <button
          className="notification-dismiss"
          onClick={() => onDismiss(id)}
          aria-label="Dismiss"
        >
          ×
        </button>
      </div>

      <div className="notification-title">{title}</div>
      <div className="notification-body">{body}</div>

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
    </div>
  );
}
