import { useEffect, useRef, useCallback } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useNotifications } from "@/hooks/useNotifications";
import { NotificationCard } from "./NotificationCard";

const PANEL_WIDTH = 400;

/** Resize the overlay window to fit only the notification content. */
function resizeToContent(el: HTMLElement | null) {
  if (!el) return;
  // Use the stack's actual rendered height + root padding (8px top + 8px bottom)
  const stack = el.querySelector(".notification-stack") as HTMLElement | null;
  const contentHeight = stack ? stack.offsetHeight + 16 : 0;
  getCurrentWindow()
    .setSize(new LogicalSize(PANEL_WIDTH, Math.max(contentHeight, 10)))
    .catch(() => {});
}

export function NotificationOverlay() {
  const { notifications, dismiss } = useNotifications();
  const rootRef = useRef<HTMLDivElement>(null);

  // Set transparent background on the overlay window (Cap pattern)
  useEffect(() => {
    document.documentElement.setAttribute("data-transparent-window", "true");
    document.body.style.background = "transparent";
  }, []);

  // Hide panel when empty, resize to fit content when not
  useEffect(() => {
    if (notifications.length === 0) {
      getCurrentWindow().hide();
    } else {
      // Small delay to let React render the cards before measuring
      requestAnimationFrame(() => resizeToContent(rootRef.current));
    }
  }, [notifications.length]);

  // Observe the notification stack for content size changes (progress updates, etc)
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    const stack = root.querySelector(".notification-stack");
    if (!stack) return;
    const observer = new ResizeObserver(() => {
      resizeToContent(root);
    });
    observer.observe(stack);
    return () => observer.disconnect();
  }, [notifications.length]);

  const handleAction = useCallback(
    (notificationId: string, actionId: string) => {
      invoke("action_callback", {
        notificationId,
        actionId,
      }).catch((err) =>
        console.error("[syncfu] action_callback failed:", err)
      );
    },
    []
  );

  return (
    <div data-testid="overlay-root" className="overlay-root" ref={rootRef}>
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
