import { useEffect } from "react";
import { event as tauriEvent } from "@tauri-apps/api";
import { useNotificationStore } from "@/stores/notificationStore";
import { NotificationCard } from "./NotificationCard";
import type { NotificationPayload } from "@/types/notification";

export function NotificationOverlay() {
  const notifications = useNotificationStore((s) => s.notifications);
  const add = useNotificationStore((s) => s.add);
  const dismiss = useNotificationStore((s) => s.dismiss);
  const dismissAll = useNotificationStore((s) => s.dismissAll);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    const setup = async () => {
      const unAdd = await tauriEvent.listen<NotificationPayload>(
        "notification:add",
        (ev) => {
          add(ev.payload as NotificationPayload);
        }
      );
      unlisteners.push(unAdd);

      const unDismiss = await tauriEvent.listen<string>(
        "notification:dismiss",
        (ev) => {
          dismiss(ev.payload as string);
        }
      );
      unlisteners.push(unDismiss);

      const unDismissAll = await tauriEvent.listen<number>(
        "notification:dismiss-all",
        () => {
          dismissAll();
        }
      );
      unlisteners.push(unDismissAll);
    };

    setup();

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [add, dismiss, dismissAll]);

  const handleDismiss = (id: string) => {
    dismiss(id);
  };

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
              onDismiss={handleDismiss}
              onAction={handleAction}
            />
          ))}
        </div>
      )}
    </div>
  );
}
