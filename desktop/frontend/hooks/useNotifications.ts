import { useEffect, useCallback } from "react";
import { event as tauriEvent, core } from "@tauri-apps/api";
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
      // Also dismiss on the backend so the panel hides when empty
      core.invoke("dismiss_notification", { id }).catch(() => {});
    },
    [storeDismiss]
  );

  const dismissAll = useCallback(() => {
    storeDismissAll();
    core.invoke("dismiss_all").catch(() => {});
  }, [storeDismissAll]);

  return {
    notifications,
    dismiss,
    dismissAll,
  } as const;
}
