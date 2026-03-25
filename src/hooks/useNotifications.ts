import { useEffect, useCallback } from "react";
import { event as tauriEvent } from "@tauri-apps/api";
import { useNotificationStore } from "@/stores/notificationStore";
import type { NotificationPayload } from "@/types/notification";

export function useNotifications() {
  const notifications = useNotificationStore((s) => s.notifications);
  const add = useNotificationStore((s) => s.add);
  const storeDismiss = useNotificationStore((s) => s.dismiss);
  const storeDismissAll = useNotificationStore((s) => s.dismissAll);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    const setup = async () => {
      const unAdd = await tauriEvent.listen<NotificationPayload>(
        "notification:add",
        (ev) => {
          console.log("[syncfu] Received notification:add", ev.payload);
          add(ev.payload as NotificationPayload);
        }
      );
      unlisteners.push(unAdd);

      const unDismiss = await tauriEvent.listen<string>(
        "notification:dismiss",
        (ev) => {
          storeDismiss(ev.payload as string);
        }
      );
      unlisteners.push(unDismiss);

      const unDismissAll = await tauriEvent.listen<number>(
        "notification:dismiss-all",
        () => {
          storeDismissAll();
        }
      );
      unlisteners.push(unDismissAll);
    };

    setup();

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [add, storeDismiss, storeDismissAll]);

  const dismiss = useCallback(
    (id: string) => {
      storeDismiss(id);
    },
    [storeDismiss]
  );

  const dismissAll = useCallback(() => {
    storeDismissAll();
  }, [storeDismissAll]);

  return {
    notifications,
    dismiss,
    dismissAll,
  } as const;
}
