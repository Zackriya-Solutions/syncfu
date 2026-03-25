import { useEffect } from "react";
import { window as tauriWindow } from "@tauri-apps/api";
import { useNotifications } from "@/hooks/useNotifications";
import { NotificationCard } from "./NotificationCard";

export function NotificationOverlay() {
  const { notifications, dismiss } = useNotifications();

  // Set transparent background on the overlay window (Cap pattern)
  useEffect(() => {
    document.documentElement.setAttribute("data-transparent-window", "true");
    document.body.style.background = "transparent";
  }, []);

  // Hide the panel window when all notifications are dismissed
  useEffect(() => {
    if (notifications.length === 0) {
      tauriWindow.getCurrentWindow().hide();
    }
  }, [notifications.length]);

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
