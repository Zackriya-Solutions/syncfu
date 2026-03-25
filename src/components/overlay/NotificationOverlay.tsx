import { useEffect, useRef, useCallback } from "react";
import { window as tauriWindow } from "@tauri-apps/api";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useNotifications } from "@/hooks/useNotifications";
import { NotificationCard } from "./NotificationCard";

const PANEL_WIDTH = 400;

/** Resize the overlay window to fit only the notification content. */
function resizeToContent(el: HTMLElement | null) {
  if (!el) return;
  const height = el.scrollHeight + 4;
  getCurrentWindow()
    .setSize(new LogicalSize(PANEL_WIDTH, Math.max(height, 10)))
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

  // Also observe DOM changes for dynamic content (progress updates etc)
  useEffect(() => {
    const el = rootRef.current;
    if (!el) return;
    const observer = new ResizeObserver(() => {
      if (notifications.length > 0) {
        resizeToContent(el);
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, [notifications.length]);

  const handleAction = useCallback(
    (_notificationId: string, _actionId: string) => {
      // TODO: send action callback via Tauri command
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
