import { useNotifications } from "@/hooks/useNotifications";
import { NotificationCard } from "./NotificationCard";

export function NotificationOverlay() {
  const { notifications, dismiss } = useNotifications();

  const handleAction = (_notificationId: string, _actionId: string) => {
    // TODO: send action callback via Tauri command
  };

  return (
    <div data-testid="overlay-root" className="overlay-root">
      {notifications.length > 0 && (
        <div data-testid="notification-stack" className="notification-stack">
          {notifications.map((notification) => (
            <NotificationCard
              key={notification.id}
              notification={notification}
              onDismiss={dismiss}
              onAction={handleAction}
            />
          ))}
        </div>
      )}
    </div>
  );
}
